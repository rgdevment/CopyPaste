import 'dart:io';
import 'dart:typed_data';

import 'package:core/core.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:image/image.dart' as img;
import 'package:path/path.dart' as p;

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
    // The queue runs its jobs via Future.whenComplete; pumping a few
    // microtasks is enough for jobs that don't block on I/O beyond what
    // the OS resolves synchronously.
    for (var i = 0; i < 50; i++) {
      if (queue.pendingCount == 0) {
        await Future<void>.delayed(const Duration(milliseconds: 50));
        if (queue.pendingCount == 0) return;
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
}
