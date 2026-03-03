import 'dart:async';

import '../models/clipboard_content_type.dart';
import '../models/clipboard_item.dart';
import '../repository/i_clipboard_repository.dart';

class ClipboardService {
  ClipboardService(this._repository);

  final IClipboardRepository _repository;
  final _itemAdded = StreamController<ClipboardItem>.broadcast();
  final _itemReactivated = StreamController<ClipboardItem>.broadcast();

  Stream<ClipboardItem> get onItemAdded => _itemAdded.stream;
  Stream<ClipboardItem> get onItemReactivated => _itemReactivated.stream;

  int pasteIgnoreWindowMs = 450;
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
    String? metadata,
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

    final item = ClipboardItem(
      content: content,
      type: type,
      appSource: source,
      metadata: metadata,
    );
    await _repository.save(item);
    _itemAdded.add(item);
    return item;
  }

  Future<ClipboardItem?> processImage(
    String contentHash, {
    String? source,
    String? imagePath,
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
    await _repository.save(item);
    _itemAdded.add(item);
    return item;
  }

  Future<void> recordPaste(String itemId) async {
    final item = await _repository.getById(itemId);
    if (item == null) return;
    await _repository.update(
      item.copyWith(
        pasteCount: item.pasteCount + 1,
        modifiedAt: DateTime.now().toUtc(),
      ),
    );
    await notifyPasteInitiated(itemId);
  }

  void dispose() {
    _itemAdded.close();
    _itemReactivated.close();
  }
}
