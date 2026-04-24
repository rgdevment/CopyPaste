import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:path/path.dart' as p;

import '../models/card_color.dart';
import '../models/clipboard_content_type.dart';
import '../models/clipboard_item.dart';
import '../repository/i_clipboard_repository.dart';
import 'app_logger.dart';
import 'image_processing_queue.dart';
import 'native_thumbnail_provider.dart';
import 'text_classifier.dart';
import 'thumbnail_queue.dart';
import 'thumbnail_service.dart';

class ClipboardService {
  ClipboardService(
    this._repository, {
    String? imagesPath,
    NativeThumbnailProvider? nativeThumbnailProvider,
  }) : _imagesPath = imagesPath,
       _thumbnailService = (imagesPath != null && imagesPath.isNotEmpty)
           ? ThumbnailService(
               imagesPath: imagesPath,
               nativeProvider: nativeThumbnailProvider,
             )
           : null {
    _imageQueue = ImageProcessingQueue(
      repository: _repository,
      onItemUpdated: _onImageItemUpdated,
    );
    final service = _thumbnailService;
    _thumbQueue = service == null
        ? null
        : ThumbnailQueue(
            repository: _repository,
            service: service,
            onItemUpdated: _onThumbItemUpdated,
          );
  }

  final IClipboardRepository _repository;
  final String? _imagesPath;
  late final ImageProcessingQueue _imageQueue;
  final ThumbnailService? _thumbnailService;
  late final ThumbnailQueue? _thumbQueue;
  final _itemAdded = StreamController<ClipboardItem>.broadcast();
  final _itemReactivated = StreamController<ClipboardItem>.broadcast();
  bool _disposed = false;

  void _onImageItemUpdated(ClipboardItem item) {
    if (!_disposed) {
      try {
        _itemReactivated.add(item);
      } on StateError catch (_) {}
    }
  }

  void _onThumbItemUpdated(ClipboardItem item) {
    if (_disposed) return;
    try {
      _itemReactivated.add(item);
    } on StateError catch (_) {}
  }

  /// Requests background regeneration of [item]'s thumbnail if the source
  /// file's `mtime` no longer matches the recorded `sourceModifiedAt`.
  /// No-op when no `imagesPath` was configured. Safe to call from `build()`
  /// — work is enqueued asynchronously.
  void requestThumbnailIfStale(ClipboardItem item) {
    _thumbQueue?.enqueueIfStale(item);
  }

  /// Forces an enqueue regardless of staleness (e.g. the user explicitly
  /// asked to refresh the thumb).
  void requestThumbnailRefresh(ClipboardItem item) {
    _thumbQueue?.enqueue(item, reason: ThumbnailJobReason.manualRefresh);
  }

  Stream<ClipboardItem> get onItemAdded => _itemAdded.stream;
  Stream<ClipboardItem> get onItemReactivated => _itemReactivated.stream;

  int pasteIgnoreWindowMs = 450;

  Stopwatch? _pasteStopwatch;
  String? _lastPastedContent;

  Future<void> notifyPasteInitiated(String itemId) async {
    _pasteStopwatch = Stopwatch()..start();
    final item = await _repository.getById(itemId);
    _lastPastedContent = item?.content;
  }

  bool _shouldIgnore(String? content) {
    final sw = _pasteStopwatch;
    if (sw == null) return false;
    final elapsed = sw.elapsedMilliseconds;
    if (elapsed < pasteIgnoreWindowMs) return true;
    if (content != null &&
        content == _lastPastedContent &&
        elapsed < pasteIgnoreWindowMs * 2) {
      return true;
    }
    return false;
  }

  Future<ClipboardItem?> processText(
    String content,
    ClipboardContentType type, {
    String? source,
    List<int>? rtfBytes,
    List<int>? htmlBytes,
  }) async {
    if (_shouldIgnore(content)) return null;

    final resolvedType = type == ClipboardContentType.text
        ? TextClassifier.classify(content)
        : type;

    final existing = await _repository.findByContentAndType(
      content,
      resolvedType,
    );
    if (existing != null) {
      final updated = existing.copyWith(modifiedAt: DateTime.now().toUtc());
      await _repository.update(updated);
      _itemReactivated.add(updated);
      return updated;
    }

    if (resolvedType != ClipboardContentType.text) {
      final legacy = await _repository.findByContentAndType(
        content,
        ClipboardContentType.text,
      );
      if (legacy != null) {
        final updated = legacy.copyWith(
          type: resolvedType,
          modifiedAt: DateTime.now().toUtc(),
        );
        await _repository.update(updated);
        _itemReactivated.add(updated);
        return updated;
      }
    }

    final meta = <String, Object>{};
    if (rtfBytes != null) meta['rtf'] = base64Encode(rtfBytes);
    if (htmlBytes != null) meta['html'] = base64Encode(htmlBytes);

    final item = ClipboardItem(
      content: content,
      type: resolvedType,
      appSource: source,
      metadata: meta.isNotEmpty ? jsonEncode(meta) : null,
    );
    await _repository.save(item);
    _itemAdded.add(item);
    return item;
  }

  Future<ClipboardItem?> processImage(
    String contentHash, {
    String? source,
    String? imagePath,
    List<int>? imageBytes,
  }) async {
    if (_shouldIgnore(null)) return null;

    final existing = await _repository.findByContentHash(contentHash);
    if (existing != null) {
      final updated = existing.copyWith(modifiedAt: DateTime.now().toUtc());
      await _repository.update(updated);
      _itemReactivated.add(updated);
      return updated;
    }

    final item = ClipboardItem(
      content: imagePath ?? '',
      type: ClipboardContentType.image,
      appSource: source,
      contentHash: contentHash,
    );

    var savedItem = item;
    if (imageBytes != null && imageBytes.isNotEmpty && _imagesPath != null) {
      try {
        final tempPath = p.join(_imagesPath, '${item.id}.bmp');
        await File(tempPath).writeAsBytes(imageBytes);
        savedItem = item.copyWith(content: tempPath);
      } catch (e) {
        AppLogger.warn(
          'processImage: could not write temp BMP for ${item.id}: $e',
        );
      }
    }

    await _repository.save(savedItem);
    _itemAdded.add(savedItem);

    if (imageBytes != null && imageBytes.isNotEmpty && _imagesPath != null) {
      _imageQueue.enqueue(
        item: savedItem,
        imageBytes: imageBytes,
        imagesPath: _imagesPath,
      );
    } else {
      // External image referenced by path: schedule thumb generation.
      // (When imageBytes is non-empty the result will land inside
      // imagesPath and ThumbnailService skips it by design.)
      _thumbQueue?.enqueue(savedItem);
    }

    return savedItem;
  }

  Future<ClipboardItem?> processFiles(
    List<String> files,
    ClipboardContentType type, {
    String? source,
  }) async {
    if (files.isEmpty) return null;
    if (_shouldIgnore(null)) return null;

    final content = files.join('\n');
    final existing = await _repository.findByContentAndType(content, type);
    if (existing != null) {
      final updated = existing.copyWith(modifiedAt: DateTime.now().toUtc());
      await _repository.update(updated);
      _itemReactivated.add(updated);
      return updated;
    }

    final firstFile = files.first;
    final meta = <String, Object>{
      'file_count': files.length,
      'file_name': p.basename(firstFile),
      'first_ext': p.extension(firstFile),
      'is_directory': type == ClipboardContentType.folder,
    };

    if (files.length == 1) {
      try {
        final fileSize = File(firstFile).lengthSync();
        meta['file_size'] = fileSize;
      } catch (e) {
        AppLogger.warn('processFiles: could not read size of $firstFile: $e');
      }
    }

    final item = ClipboardItem(
      content: content,
      type: type,
      appSource: source,
      metadata: jsonEncode(meta),
    );
    await _repository.save(item);
    _itemAdded.add(item);

    // Native-backed thumbs cover video/audio (and image when the path is
    // external). The queue ignores types it cannot handle, so this is a
    // safe fire-and-forget call.
    if (files.length == 1) {
      _thumbQueue?.enqueue(item);
    }

    return item;
  }

  Future<ClipboardItem?> recordPaste(String itemId) async {
    final now = DateTime.now().toUtc();
    final item = await _repository.getById(itemId);
    if (item == null) return null;
    final updated = item.copyWith(
      pasteCount: item.pasteCount + 1,
      modifiedAt: now,
    );
    await _repository.update(updated);
    return updated;
  }

  Future<void> removeItem(String id) async {
    final item = await _repository.getById(id);
    await _repository.delete(id);
    if (item != null) {
      _cleanupItemFiles(item);
    }
  }

  /// Deletes a file only if [path] is canonically contained inside the app's
  /// own images directory. Any path outside is refused and logged.
  ///
  /// This is the single entry point for file deletion in this service. Never
  /// call `File.delete*` directly on a path that comes from user input, item
  /// content, or any source outside the app's own path builder.
  bool _deleteAppFile(String path) {
    final imagesPath = _imagesPath;
    if (imagesPath == null || imagesPath.isEmpty) return false;
    final String canonicalBase;
    final String canonicalTarget;
    try {
      canonicalBase = p.canonicalize(imagesPath);
      canonicalTarget = p.canonicalize(path);
    } catch (e) {
      AppLogger.warn('_deleteAppFile: canonicalize failed for "$path": $e');
      return false;
    }
    final baseWithSep = canonicalBase.endsWith(p.separator)
        ? canonicalBase
        : '$canonicalBase${p.separator}';
    if (!canonicalTarget.startsWith(baseWithSep)) {
      AppLogger.error(
        '_deleteAppFile: refused to delete out-of-scope path '
        '"$canonicalTarget" (base="$canonicalBase")',
      );
      return false;
    }
    try {
      final file = File(canonicalTarget);
      if (file.existsSync()) file.deleteSync();
      return true;
    } catch (e) {
      AppLogger.warn('_deleteAppFile: delete failed for "$path": $e');
      return false;
    }
  }

  void _cleanupItemFiles(ClipboardItem item) {
    if (item.type == ClipboardContentType.image && item.content.isNotEmpty) {
      _deleteAppFile(item.content);
    }
    final thumb = item.thumbPath;
    if (thumb != null && thumb.isNotEmpty) {
      _deleteAppFile(thumb);
    }
  }

  Future<List<ClipboardItem>> getHistoryAdvanced({
    String? query,
    List<ClipboardContentType>? types,
    List<CardColor>? colors,
    bool? isPinned,
    int limit = 50,
    int skip = 0,
  }) => _repository.searchAdvanced(
    query: query,
    types: types,
    colors: colors,
    isPinned: isPinned,
    limit: limit,
    skip: skip,
  );

  Future<void> updatePin(String id, bool isPinned) async {
    final item = await _repository.getById(id);
    if (item == null) return;
    await _repository.update(
      item.copyWith(isPinned: isPinned, modifiedAt: DateTime.now().toUtc()),
    );
  }

  Future<void> updateLabelAndColor(
    String id,
    String? label,
    CardColor color,
  ) async {
    final item = await _repository.getById(id);
    if (item == null) return;
    await _repository.update(
      item.copyWith(
        label: label,
        cardColor: color,
        modifiedAt: DateTime.now().toUtc(),
      ),
    );
  }

  Future<int> clearUnpinnedHistory() => _repository.deleteAllUnpinned();

  Future<int> getItemCount() => _repository.count();

  Future<void> reclassifyLegacyTextItems() async {
    const batchSize = 50;
    var skip = 0;
    while (true) {
      if (_disposed) return;
      final batch = await _repository.searchAdvanced(
        types: [ClipboardContentType.text],
        limit: batchSize,
        skip: skip,
      );
      if (batch.isEmpty) return;
      for (final item in batch) {
        if (_disposed) return;
        final resolved = TextClassifier.classify(item.content);
        if (resolved != ClipboardContentType.text) {
          await _repository.update(item.copyWith(type: resolved));
        }
      }
      if (batch.length < batchSize) return;
      skip += batchSize;
    }
  }

  Future<void> walCheckpoint() => _repository.walCheckpoint();

  Future<void> updateMetadata(String id, String metadata) async {
    final item = await _repository.getById(id);
    if (item == null) return;
    final updated = item.copyWith(metadata: metadata);
    await _repository.update(updated);
    if (!_disposed) _itemReactivated.add(updated);
  }

  Future<void> dispose() async {
    _disposed = true;
    await _imageQueue.dispose();
    await _thumbQueue?.dispose();
    await _itemAdded.close();
    await _itemReactivated.close();
  }
}
