import 'dart:io';
import 'dart:typed_data';

import 'package:core/models/card_color.dart';
import 'package:core/models/clipboard_content_type.dart';
import 'package:core/models/clipboard_item.dart';
import 'package:core/repository/i_clipboard_repository.dart';
import 'package:core/services/image_processing_queue.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:image/image.dart' as img;
import 'package:path/path.dart' as p;

class _RecordingRepo implements IClipboardRepository {
  final updates = <ClipboardItem>[];

  @override
  Future<void> update(ClipboardItem item) async {
    updates.add(item);
  }

  // Unused in these tests.
  @override
  Future<void> save(ClipboardItem item) async {}
  @override
  Future<ClipboardItem?> getById(String id) async => null;
  @override
  Future<ClipboardItem?> getLatest() async => null;
  @override
  Future<ClipboardItem?> findByContentAndType(
    String content,
    ClipboardContentType type,
  ) async => null;
  @override
  Future<ClipboardItem?> findByContentHash(String contentHash) async => null;
  @override
  Future<List<ClipboardItem>> getAll() async => const [];
  @override
  Future<void> delete(String id) async {}
  @override
  Future<int> clearOldItems(int days, {bool excludePinned = true}) async => 0;
  @override
  Future<int> deleteAllUnpinned() async => 0;
  @override
  Future<int> count() async => 0;
  @override
  Future<List<ClipboardItem>> search(
    String query, {
    int limit = 50,
    int skip = 0,
  }) async => const [];
  @override
  Future<List<ClipboardItem>> searchAdvanced({
    String? query,
    List<ClipboardContentType>? types,
    List<CardColor>? colors,
    bool? isPinned,
    required int limit,
    required int skip,
  }) async => const [];
  @override
  Future<List<String>> getImagePaths() async => const [];
  @override
  Future<List<String>> getThumbPaths() async => const [];
  @override
  Future<void> walCheckpoint() async {}
  @override
  Future<void> close() async {}
}

Uint8List _smallPng() {
  final image = img.Image(width: 32, height: 32);
  for (var x = 0; x < 32; x++) {
    for (var y = 0; y < 32; y++) {
      image.setPixelRgb(x, y, x * 8, y * 8, 64);
    }
  }
  return Uint8List.fromList(img.encodePng(image));
}

ClipboardItem _item(String id) =>
    ClipboardItem(id: id, content: '$id.bmp', type: ClipboardContentType.image);

void main() {
  late Directory tempDir;
  late _RecordingRepo repo;
  late ImageProcessingQueue queue;

  setUp(() {
    tempDir = Directory.systemTemp.createTempSync('img_queue_test_');
    repo = _RecordingRepo();
    queue = ImageProcessingQueue(repository: repo);
  });

  tearDown(() async {
    await queue.dispose();
    if (tempDir.existsSync()) tempDir.deleteSync(recursive: true);
  });

  group('ImageProcessingQueue.getMaxImageBytes (PR #10)', () {
    test('drops job when input exceeds cap', () async {
      final bytes = _smallPng();
      queue.getMaxImageBytes = () => bytes.length - 1; // strict cap

      queue.enqueue(
        item: _item('drop-me'),
        imageBytes: bytes,
        imagesPath: tempDir.path,
      );

      // Give the isolate time to (not) run.
      await Future<void>.delayed(const Duration(milliseconds: 200));
      expect(
        repo.updates,
        isEmpty,
        reason: 'queue must skip oversized buffers without invoking the repo',
      );
    });

    test('processes job when input is under cap', () async {
      final bytes = _smallPng();
      queue.getMaxImageBytes = () => bytes.length + 1024;

      queue.enqueue(
        item: _item('keep-me'),
        imageBytes: bytes,
        imagesPath: tempDir.path,
      );

      // The isolate writes a real PNG; wait for it to complete.
      await Future<void>.delayed(const Duration(seconds: 3));
      expect(repo.updates, isNotEmpty);
      final pngPath = p.join(tempDir.path, 'keep-me.png');
      expect(File(pngPath).existsSync(), isTrue);
    });

    test('cap of 0 disables the gate (bytes flow through)', () async {
      final bytes = _smallPng();
      queue.getMaxImageBytes = () => 0;

      queue.enqueue(
        item: _item('zero-cap'),
        imageBytes: bytes,
        imagesPath: tempDir.path,
      );

      await Future<void>.delayed(const Duration(seconds: 3));
      expect(repo.updates, isNotEmpty);
    });
  });

  group('ImageProcessingQueue depth warning', () {
    test('logs warn when more than 10 items are pending', () async {
      // Enqueue 12 items synchronously — first starts asynchronously while
      // items 2-12 accumulate. When item 12 is added, _queue.length > 10
      // triggers AppLogger.warn (line 81).
      for (var i = 0; i < 12; i++) {
        queue.enqueue(
          item: _item('depth-warn-$i'),
          imageBytes: _smallPng(),
          imagesPath: tempDir.path,
        );
      }
      // Just verify no exception was thrown and items were accepted.
      await Future<void>.delayed(const Duration(seconds: 4));
      expect(repo.updates.length, greaterThanOrEqualTo(12));
    });
  });

  group('ImageProcessingQueue timeout', () {
    test('TimeoutException is handled and no update is emitted', () async {
      final slowRepo = _RecordingRepo();
      final slowQueue = ImageProcessingQueue(
        repository: slowRepo,
        jobTimeout: const Duration(milliseconds: 100),
      );

      // Pass valid PNG bytes but a non-existent imagesPath.
      // ImageProcessor.processSync decodes OK, then throws FileSystemException
      // on File.writeAsBytesSync — the isolate exits without sending a result.
      // resultCompleter never completes → timeout fires after 100ms.
      slowQueue.enqueue(
        item: _item('slow'),
        imageBytes: _smallPng(),
        imagesPath: '/nonexistent_copypaste_timeout_test_path',
      );

      // Wait enough for the 100ms timeout to fire.
      await Future<void>.delayed(const Duration(milliseconds: 500));
      // Timeout fired → no update recorded for the item.
      expect(slowRepo.updates.where((u) => u.id == 'slow'), isEmpty);
    });
  });
}
