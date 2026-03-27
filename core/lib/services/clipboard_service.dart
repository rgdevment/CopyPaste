import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:path/path.dart' as p;

import '../models/card_color.dart';
import '../models/clipboard_content_type.dart';
import '../models/clipboard_item.dart';
import '../repository/i_clipboard_repository.dart';
import 'app_logger.dart';
import 'image_processor.dart';
import 'text_classifier.dart';

class ClipboardService {
  ClipboardService(this._repository, {String? imagesPath})
    : _imagesPath = imagesPath;

  final IClipboardRepository _repository;
  final String? _imagesPath;
  final _itemAdded = StreamController<ClipboardItem>.broadcast();
  final _itemReactivated = StreamController<ClipboardItem>.broadcast();
  bool _disposed = false;

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
      unawaited(_processImageBackground(savedItem, imageBytes));
    }

    return savedItem;
  }

  Future<void> _processImageBackground(
    ClipboardItem item,
    List<int> imageBytes,
  ) async {
    try {
      final bytes = imageBytes is Uint8List
          ? imageBytes
          : Uint8List.fromList(imageBytes);
      final result = await ImageProcessor.processAndSave(
        imageBytes: bytes,
        id: item.id,
        imagesDir: _imagesPath!,
      );
      if (result == null) {
        AppLogger.warn(
          '_processImageBackground: ImageProcessor returned null for ${item.id}',
        );
        return;
      }
      if (_disposed) return;

      final bmpPath = p.join(_imagesPath, '${item.id}.bmp');
      try {
        final bmp = File(bmpPath);
        if (bmp.existsSync()) bmp.deleteSync();
      } catch (_) {}

      final meta = <String, Object>{
        'width': result.width,
        'height': result.height,
        'size': result.fileSize,
      };
      final updated = item.copyWith(
        content: result.imagePath,
        metadata: jsonEncode(meta),
      );
      await _repository.update(updated);
      if (!_disposed) {
        try {
          _itemReactivated.add(updated);
        } on StateError catch (_) {}
      }
    } catch (e, s) {
      AppLogger.error('Image processing failed for ${item.id}: $e\n$s');
    }
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

  void _cleanupItemFiles(ClipboardItem item) {
    if (item.type == ClipboardContentType.image && item.content.isNotEmpty) {
      try {
        final file = File(item.content);
        if (file.existsSync()) file.deleteSync();
      } catch (e) {
        AppLogger.warn(
          '_cleanupItemFiles: could not delete image for ${item.id}: $e',
        );
      }
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

  void dispose() {
    _disposed = true;
    _itemAdded.close();
    _itemReactivated.close();
  }
}
