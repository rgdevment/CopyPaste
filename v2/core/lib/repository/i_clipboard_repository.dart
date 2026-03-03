import '../models/card_color.dart';
import '../models/clipboard_content_type.dart';
import '../models/clipboard_item.dart';

abstract interface class IClipboardRepository {
  Future<void> save(ClipboardItem item);
  Future<void> update(ClipboardItem item);
  Future<ClipboardItem?> getById(String id);
  Future<ClipboardItem?> getLatest();
  Future<ClipboardItem?> findByContentAndType(
    String content,
    ClipboardContentType type,
  );
  Future<ClipboardItem?> findByContentHash(String contentHash);
  Future<List<ClipboardItem>> getAll();
  Future<void> delete(String id);
  Future<int> clearOldItems(int days, {bool excludePinned = true});
  Future<List<ClipboardItem>> search(
    String query, {
    int limit = 50,
    int skip = 0,
  });
  Future<List<ClipboardItem>> searchAdvanced({
    String? query,
    List<ClipboardContentType>? types,
    List<CardColor>? colors,
    bool? isPinned,
    required int limit,
    required int skip,
  });
  Future<void> close();
}
