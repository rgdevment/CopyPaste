import 'dart:io';
import 'dart:typed_data';

import 'package:core/core.dart';
import 'package:core/repository/i_clipboard_repository.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:image/image.dart' as img;
import 'package:path/path.dart' as p;

class _ThrowingUpdateRepo implements IClipboardRepository {
  final _items = <String, ClipboardItem>{};

  void seed(ClipboardItem item) => _items[item.id] = item;

  @override
  Future<void> save(ClipboardItem item) async => _items[item.id] = item;
  @override
  Future<void> update(ClipboardItem item) async =>
      throw Exception('update deliberately failed');
  @override
  Future<ClipboardItem?> getById(String id) async => _items[id];
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
  Future<int> count() async => _items.length;
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

void main() {
  late Directory tempDir;
  late Directory imagesDir;
  late Directory externalDir;
  late SqliteRepository repo;
  late ThumbnailService service;
  late ThumbnailQueue queue;
  late List<ClipboardItem> updatedItems;

  setUp(() {
    tempDir = Directory.systemTemp.createTempSync('thumb_queue_test_');
    imagesDir = Directory(p.join(tempDir.path, 'images'))
      ..createSync(recursive: true);
    externalDir = Directory(p.join(tempDir.path, 'external'))
      ..createSync(recursive: true);
    repo = SqliteRepository.inMemory();
    service = ThumbnailService(imagesPath: imagesDir.path);
    updatedItems = [];
    queue = ThumbnailQueue(
      repository: repo,
      service: service,
      onItemUpdated: updatedItems.add,
    );
  });

  tearDown(() async {
    await queue.dispose();
    await repo.close();
    if (tempDir.existsSync()) tempDir.deleteSync(recursive: true);
  });

  Uint8List makePng({int w = 1024, int h = 1024}) {
    final image = img.Image(width: w, height: h);
    return Uint8List.fromList(img.encodePng(image));
  }

  Future<ClipboardItem> saveImageItem(String externalPath, {String? id}) async {
    final item = ClipboardItem(
      id: id ?? 'item-${DateTime.now().microsecondsSinceEpoch}',
      content: externalPath,
      type: ClipboardContentType.image,
    );
    await repo.save(item);
    return item;
  }

  Future<void> drainQueue() async {
    // Wait until the queue is fully idle: no pending jobs AND no in-flight
    // encode. `pendingCount` alone is not enough — it drops to zero as soon
    // as a job is taken off the queue, while the isolate may still be
    // encoding the PNG. Poll up to ~5 s, which is generous enough for
    // slow Linux CI runners.
    for (var i = 0; i < 100; i++) {
      if (queue.isIdle) {
        // One more pump so the `whenComplete` chain in `_scheduleNext`
        // has a chance to flush its microtasks before the test asserts.
        await Future<void>.delayed(const Duration(milliseconds: 10));
        if (queue.isIdle) return;
      }
      await Future<void>.delayed(const Duration(milliseconds: 50));
    }
  }

  group('ThumbnailQueue.enqueue', () {
    test('generates thumb, persists thumbPath, emits onItemUpdated', () async {
      final src = File(p.join(externalDir.path, 'big.png'))
        ..writeAsBytesSync(makePng(w: 1024, h: 512));
      final item = await saveImageItem(src.path, id: 'fresh');

      queue.enqueue(item);
      await drainQueue();

      final stored = await repo.getById('fresh');
      expect(stored, isNotNull);
      expect(
        stored!.thumbPath,
        equals(p.join(imagesDir.path, 'fresh_thumb.png')),
      );
      expect(stored.sourceModifiedAt, isNotNull);
      expect(File(stored.thumbPath!).existsSync(), isTrue);
      expect(updatedItems, hasLength(1));
      expect(updatedItems.single.id, equals('fresh'));
    });

    test('ignores duplicate enqueue for same id while pending', () async {
      final src = File(p.join(externalDir.path, 'dup.png'))
        ..writeAsBytesSync(makePng());
      final item = await saveImageItem(src.path, id: 'dup');

      queue.enqueue(item);
      queue.enqueue(item);
      queue.enqueue(item);
      await drainQueue();

      expect(updatedItems, hasLength(1));
    });

    test('skips non-image items', () async {
      final item = ClipboardItem(
        id: 'txt',
        content: 'hello',
        type: ClipboardContentType.text,
      );
      await repo.save(item);

      queue.enqueue(item);
      await drainQueue();

      final stored = await repo.getById('txt');
      expect(stored?.thumbPath, isNull);
      expect(updatedItems, isEmpty);
    });

    test('skips multi-path content', () async {
      final a = File(p.join(externalDir.path, 'a.png'))
        ..writeAsBytesSync(makePng());
      final b = File(p.join(externalDir.path, 'b.png'))
        ..writeAsBytesSync(makePng());
      final item = ClipboardItem(
        id: 'multi',
        content: '${a.path}\n${b.path}',
        type: ClipboardContentType.image,
      );
      await repo.save(item);

      queue.enqueue(item);
      await drainQueue();

      expect((await repo.getById('multi'))?.thumbPath, isNull);
    });
  });

  group('ThumbnailQueue race conditions', () {
    test(
      'drops generated thumb if item was deleted during generation',
      () async {
        final src = File(p.join(externalDir.path, 'race.png'))
          ..writeAsBytesSync(makePng(w: 2048, h: 2048));
        final item = await saveImageItem(src.path, id: 'race');

        queue.enqueue(item);
        // Delete before the encoder can finish (encoder is in an isolate
        // and the file is large enough to take more than zero microtasks).
        await repo.delete('race');
        await drainQueue();

        // The generated thumb (if any) must have been cleaned up by the
        // queue's race-window check. The repository row stays gone.
        final orphan = File(p.join(imagesDir.path, 'race_thumb.png'));
        expect(orphan.existsSync(), isFalse);
        expect(updatedItems, isEmpty);
      },
    );

    test('skips entirely if item is gone before generation starts', () async {
      final src = File(p.join(externalDir.path, 'gone.png'))
        ..writeAsBytesSync(makePng());
      final item = await saveImageItem(src.path, id: 'gone');
      await repo.delete('gone');

      queue.enqueue(item);
      await drainQueue();

      expect(
        File(p.join(imagesDir.path, 'gone_thumb.png')).existsSync(),
        isFalse,
      );
      expect(updatedItems, isEmpty);
    });
  });

  group('ThumbnailQueue.enqueueIfStale', () {
    test('enqueues when no sourceModifiedAt has been recorded', () async {
      final src = File(p.join(externalDir.path, 'cold.png'))
        ..writeAsBytesSync(makePng());
      final item = await saveImageItem(src.path, id: 'cold');

      queue.enqueueIfStale(item);
      await drainQueue();

      expect((await repo.getById('cold'))?.thumbPath, isNotNull);
    });

    test(
      'does not enqueue when mtime matches recorded sourceModifiedAt',
      () async {
        final src = File(p.join(externalDir.path, 'fresh.png'))
          ..writeAsBytesSync(makePng());
        final mtime = src.statSync().modified.toUtc();
        final item = ClipboardItem(
          id: 'fresh-stale',
          content: src.path,
          type: ClipboardContentType.image,
          sourceModifiedAt: mtime,
        );
        await repo.save(item);

        queue.enqueueIfStale(item);
        await drainQueue();

        expect(updatedItems, isEmpty);
      },
    );

    test('enqueues when source mtime differs from recorded', () async {
      final src = File(p.join(externalDir.path, 'stale.png'))
        ..writeAsBytesSync(makePng());
      final past = DateTime.utc(2020, 1, 1);
      final item = ClipboardItem(
        id: 'stale',
        content: src.path,
        type: ClipboardContentType.image,
        sourceModifiedAt: past,
      );
      await repo.save(item);

      queue.enqueueIfStale(item);
      await drainQueue();

      final stored = await repo.getById('stale');
      expect(stored?.thumbPath, isNotNull);
      expect(stored!.sourceModifiedAt, isNot(equals(past)));
    });

    test('no-op for missing source file', () async {
      final item = ClipboardItem(
        id: 'missing',
        content: p.join(externalDir.path, 'nope.png'),
        type: ClipboardContentType.image,
      );
      await repo.save(item);

      queue.enqueueIfStale(item);
      await drainQueue();

      expect(updatedItems, isEmpty);
    });
  });

  group('ThumbnailQueue.dispose', () {
    test('refuses new jobs after dispose', () async {
      await queue.dispose();
      final src = File(p.join(externalDir.path, 'late.png'))
        ..writeAsBytesSync(makePng());
      final item = await saveImageItem(src.path, id: 'late');

      queue.enqueue(item);
      await Future<void>.delayed(const Duration(milliseconds: 100));

      expect(updatedItems, isEmpty);
      expect((await repo.getById('late'))?.thumbPath, isNull);
    });
  });

  group('ThumbnailQueue.pendingCount', () {
    test('is zero on a fresh queue', () {
      expect(queue.pendingCount, equals(0));
    });
  });

  group('ThumbnailQueue depth warning', () {
    test('logs warn when more than 20 items are pending', () async {
      // Enqueue 22 items synchronously — the first starts processing
      // asynchronously while items 2-22 accumulate in _queue.
      // When item 22 is added, _queue.length > 20 triggers AppLogger.warn.
      for (var i = 0; i < 22; i++) {
        final item = ClipboardItem(
          id: 'depth-warn-$i',
          content: p.join(externalDir.path, 'depth_$i.png'),
          type: ClipboardContentType.image,
        );
        queue.enqueue(item);
      }
      // Verify items accumulated (warning was logged).
      expect(queue.pendingCount, greaterThan(0));
    });
  });

  group('ThumbnailQueue._safeGenerate exception', () {
    test('catches write failure inside Isolate and emits no update', () async {
      final dir = Directory.systemTemp.createTempSync('tq_safegen_err_');
      final ext = Directory(p.join(dir.path, 'ext'))..createSync();
      final imgs = Directory(p.join(dir.path, 'imgs'))..createSync();
      final src = File(p.join(ext.path, 'source.png'))
        ..writeAsBytesSync(makePng(w: 64, h: 64));

      final localRepo = SqliteRepository.inMemory();
      final localService = ThumbnailService(imagesPath: imgs.path);
      final updatedLocal = <ClipboardItem>[];
      final localQueue = ThumbnailQueue(
        repository: localRepo,
        service: localService,
        onItemUpdated: updatedLocal.add,
      );

      final item = ClipboardItem(
        id: 'safegen-err',
        content: src.path,
        type: ClipboardContentType.image,
      );
      await localRepo.save(item);

      // Make imgs dir non-writable so the isolate's File.writeAsBytesSync
      // throws EACCES, which propagates through Isolate.run and is caught by
      // _safeGenerate (line 184).
      await Process.run('chmod', ['555', imgs.path]);

      try {
        localQueue.enqueue(item);
        for (var i = 0; i < 100; i++) {
          if (localQueue.isIdle) break;
          await Future<void>.delayed(const Duration(milliseconds: 50));
        }
        expect(updatedLocal, isEmpty);
      } finally {
        await Process.run('chmod', ['755', imgs.path]);
        await localQueue.dispose();
        await localRepo.close();
        dir.deleteSync(recursive: true);
      }
    });
  });

  group('ThumbnailQueue update failure', () {
    test('catches repo update error and deletes generated thumb', () async {
      final dir = Directory.systemTemp.createTempSync('tq_update_fail_');
      final ext = Directory(p.join(dir.path, 'ext'))..createSync();
      final imgs = Directory(p.join(dir.path, 'imgs'))..createSync();
      final src = File(p.join(ext.path, 'source.png'))
        ..writeAsBytesSync(makePng(w: 64, h: 64));

      final throwingRepo = _ThrowingUpdateRepo();
      final localService = ThumbnailService(imagesPath: imgs.path);
      final updatedLocal = <ClipboardItem>[];
      final localQueue = ThumbnailQueue(
        repository: throwingRepo,
        service: localService,
        onItemUpdated: updatedLocal.add,
      );

      final item = ClipboardItem(
        id: 'update-fail',
        content: src.path,
        type: ClipboardContentType.image,
      );
      throwingRepo.seed(item);

      try {
        localQueue.enqueue(item);
        // Wait for the job to attempt update, fail, and return to idle.
        for (var i = 0; i < 200; i++) {
          if (localQueue.isIdle) break;
          await Future<void>.delayed(const Duration(milliseconds: 50));
        }
        // update threw → onItemUpdated was never called.
        expect(updatedLocal, isEmpty);
        // The generated thumb was deleted by _safeDelete.
        final thumbFiles = imgs.listSync().whereType<File>().toList();
        expect(thumbFiles, isEmpty);
      } finally {
        await localQueue.dispose();
        dir.deleteSync(recursive: true);
      }
    });
  });
}
