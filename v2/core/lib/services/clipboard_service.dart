import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:path/path.dart' as p;

import '../models/card_color.dart';
import '../models/clipboard_content_type.dart';
import '../models/clipboard_item.dart';
import '../repository/i_clipboard_repository.dart';

class ClipboardService {
  ClipboardService(this._repository, {String? imagesPath})
      : _imagesPath = imagesPath;

  final IClipboardRepository _repository;
  final String? _imagesPath;
  final _itemAdded = StreamController<ClipboardItem>.broadcast();
  final _itemReactivated = StreamController<ClipboardItem>.broadcast();

  Stream<ClipboardItem> get onItemAdded => _itemAdded.stream;
  Stream<ClipboardItem> get onItemReactivated => _itemReactivated.stream;

  String? _thumbnailsPath;
  int pasteIgnoreWindowMs = 450;

  void setThumbnailsPath(String path) => _thumbnailsPath = path;

  DateTime _lastPasteTime =
      DateTime.fromMillisecondsSinceEpoch(0, isUtc: true);
  String? _lastPastedContent;

  Future<void> notifyPasteInitiated(String itemId) async {
    _lastPasteTime = DateTime.now().toUtc();
    final item = await _repository.getById(itemId);
    _lastPastedContent = item?.content;
  }

  bool _shouldIgnore(String? content) {
    final elapsed = DateTime.now().toUtc().difference(_lastPasteTime);
    if (elapsed.inMilliseconds < pasteIgnoreWindowMs) {
      return true;
    }
    if (content != null &&
        content == _lastPastedContent &&
        elapsed.inSeconds < 2) {
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

    final existing =
        await _repository.findByContentAndType(content, type);
    if (existing != null) {
      final updated =
          existing.copyWith(modifiedAt: DateTime.now().toUtc());
      await _repository.update(updated);
      _itemReactivated.add(updated);
      return updated;
    }

    final meta = <String, Object>{};
    if (rtfBytes != null) meta['rtf'] = base64Encode(rtfBytes);
    if (htmlBytes != null) meta['html'] = base64Encode(htmlBytes);

    final item = ClipboardItem(
      content: content,
      type: type,
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
      final updated =
          existing.copyWith(modifiedAt: DateTime.now().toUtc());
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
        final savedPath = p.join(_imagesPath, '${item.id}.bmp');
        await File(savedPath).writeAsBytes(imageBytes);
        savedItem = item.copyWith(content: savedPath);
      } catch (_) {}
    }

    await _repository.save(savedItem);
    _itemAdded.add(savedItem);
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
    final existing =
        await _repository.findByContentAndType(content, type);
    if (existing != null) {
      final updated =
          existing.copyWith(modifiedAt: DateTime.now().toUtc());
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
      } catch (_) {}
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
    final item = await _repository.getById(itemId);
    if (item == null) return null;
    final updated = item.copyWith(
      pasteCount: item.pasteCount + 1,
      modifiedAt: DateTime.now().toUtc(),
    );
    await _repository.update(updated);
    await notifyPasteInitiated(itemId);
    return updated;
  }

  Future<void> removeItem(String id) async {
    final item = await _repository.getById(id);
    if (item != null) {
      _cleanupItemFiles(item);
    }
    await _repository.delete(id);
  }

  void _cleanupItemFiles(ClipboardItem item) {
    if (item.type == ClipboardContentType.image &&
        item.content.isNotEmpty) {
      try {
        final file = File(item.content);
        if (file.existsSync()) file.deleteSync();
      } catch (_) {}
    }

    if (_thumbnailsPath != null) {
      const thumbExtensions = ['.png', '.jpg', '.jpeg', '.webp'];
      for (final ext in thumbExtensions) {
        try {
          final thumbPath = p.join(_thumbnailsPath!, '${item.id}_t$ext');
          final thumbFile = File(thumbPath);
          if (thumbFile.existsSync()) thumbFile.deleteSync();
        } catch (_) {}
      }
    }
  }

  Future<List<ClipboardItem>> getHistory({int limit = 50, int skip = 0}) =>
      _repository.search('', limit: limit, skip: skip);

  Future<List<ClipboardItem>> getHistoryAdvanced({
    String? query,
    List<ClipboardContentType>? types,
    List<CardColor>? colors,
    bool? isPinned,
    int limit = 50,
    int skip = 0,
  }) =>
      _repository.searchAdvanced(
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

  void dispose() {
    _itemAdded.close();
    _itemReactivated.close();
  }
}
