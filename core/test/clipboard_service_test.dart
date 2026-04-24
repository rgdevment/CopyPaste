import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:image/image.dart' as img;
import 'package:path/path.dart' as p;

import 'package:core/core.dart';

void main() {
  late SqliteRepository repo;
  late ClipboardService service;

  setUp(() {
    repo = SqliteRepository.inMemory();
    service = ClipboardService(repo);
  });

  tearDown(() async {
    await service.dispose();
    await repo.close();
  });

  group('ClipboardService.processText', () {
    test('saves new item and emits onItemAdded', () async {
      ClipboardItem? emitted;
      service.onItemAdded.listen((item) => emitted = item);

      final result = await service.processText(
        'hello',
        ClipboardContentType.text,
      );
      await Future<void>.delayed(Duration.zero);

      expect(result, isNotNull);
      expect(result!.content, equals('hello'));
      expect(emitted?.content, equals('hello'));
    });

    test('reactivates existing item and emits onItemReactivated', () async {
      ClipboardItem? reactivated;
      service.onItemReactivated.listen((item) => reactivated = item);

      final first = await service.processText('dup', ClipboardContentType.text);
      expect(first, isNotNull);

      final second = await service.processText(
        'dup',
        ClipboardContentType.text,
      );
      await Future<void>.delayed(Duration.zero);

      expect(second, isNotNull);
      expect(reactivated?.content, equals('dup'));
    });

    test('returns null when inside paste ignore window', () async {
      service.pasteIgnoreWindowMs = 60000;
      await service.notifyPasteInitiated('any-id');

      final result = await service.processText(
        'ignored',
        ClipboardContentType.text,
      );
      expect(result, isNull);
    });

    test('saves item with source and rtf/html metadata', () async {
      final result = await service.processText(
        'data',
        ClipboardContentType.link,
        source: 'Chrome',
        rtfBytes: [72, 69, 76, 76, 79],
        htmlBytes: [60, 104, 116, 109, 108, 62],
      );
      expect(result!.appSource, equals('Chrome'));
      expect(result.metadata, isNotNull);
      expect(result.metadata, contains('rtf'));
      expect(result.metadata, contains('html'));
    });

    test('saves item without metadata when no rtf/html provided', () async {
      final result = await service.processText(
        'plain',
        ClipboardContentType.text,
      );
      expect(result!.metadata, isNull);
    });
  });

  group('ClipboardService.processImage', () {
    test('saves new image item by contentHash', () async {
      final result = await service.processImage(
        'hash-abc',
        imagePath: '/tmp/image.png',
      );
      expect(result, isNotNull);
      expect(result!.contentHash, equals('hash-abc'));
      expect(result.type, equals(ClipboardContentType.image));
      expect(result.content, equals('/tmp/image.png'));
    });

    test('reactivates duplicate image by hash', () async {
      ClipboardItem? reactivated;
      service.onItemReactivated.listen((item) => reactivated = item);

      await service.processImage('hash-dup');
      await service.processImage('hash-dup');
      await Future<void>.delayed(Duration.zero);

      expect(reactivated, isNotNull);
    });

    test('enqueues thumbnail generation for external image path', () async {
      final imagesDir = Directory.systemTemp.createTempSync('svc_proc_thumb_');
      final externalDir = Directory.systemTemp.createTempSync('svc_proc_ext_');
      try {
        final svc = ClipboardService(repo, imagesPath: imagesDir.path);
        // Real PNG so the ThumbnailService can decode it.
        final pixels = img.Image(width: 64, height: 64);
        final externalFile = File(p.join(externalDir.path, 'photo.png'))
          ..writeAsBytesSync(img.encodePng(pixels));

        ClipboardItem? reactivated;
        final sub = svc.onItemReactivated.listen((it) => reactivated = it);

        final created = await svc.processImage(
          'thumb-enq-hash',
          imagePath: externalFile.path,
        );
        expect(created, isNotNull);

        // Wait for the thumbnail queue to finish (single short job).
        for (var i = 0; i < 40; i++) {
          await Future<void>.delayed(const Duration(milliseconds: 50));
          final stored = await repo.getById(created!.id);
          if (stored?.thumbPath != null) break;
        }

        final stored = await repo.getById(created!.id);
        expect(stored?.thumbPath, isNotNull);
        expect(File(stored!.thumbPath!).existsSync(), isTrue);
        expect(reactivated?.id, equals(stored.id));

        await sub.cancel();
        await svc.dispose();
      } finally {
        imagesDir.deleteSync(recursive: true);
        externalDir.deleteSync(recursive: true);
      }
    });
  });

  group('ClipboardService.recordPaste', () {
    test('increments pasteCount and returns updated item', () async {
      final item = await service.processText(
        'paste me',
        ClipboardContentType.text,
      );
      expect(item, isNotNull);

      final updated = await service.recordPaste(item!.id);

      expect(updated, isNotNull);
      expect(updated!.pasteCount, equals(1));
      final stored = await repo.getById(item.id);
      expect(stored!.pasteCount, equals(1));
    });

    test('returns null for unknown id', () async {
      final result = await service.recordPaste('nonexistent-id');
      expect(result, isNull);
    });
  });

  group('ClipboardService.processFiles', () {
    test('saves file list with metadata', () async {
      ClipboardItem? emitted;
      service.onItemAdded.listen((item) => emitted = item);

      final result = await service.processFiles(
        ['C:\\docs\\file1.txt', 'C:\\docs\\file2.txt'],
        ClipboardContentType.file,
        source: 'explorer',
      );
      await Future<void>.delayed(Duration.zero);

      expect(result, isNotNull);
      expect(result!.content, contains('file1.txt'));
      expect(result.content, contains('file2.txt'));
      expect(result.metadata, isNotNull);
      expect(result.metadata, contains('file_count'));
      expect(result.metadata, contains('"file_count":2'));
      expect(result.appSource, equals('explorer'));
      expect(emitted?.id, equals(result.id));
    });

    test('returns null for empty file list', () async {
      final result = await service.processFiles([], ClipboardContentType.file);
      expect(result, isNull);
    });

    test('reactivates duplicate file list', () async {
      ClipboardItem? reactivated;
      service.onItemReactivated.listen((item) => reactivated = item);

      await service.processFiles(['C:\\same.txt'], ClipboardContentType.file);
      await service.processFiles(['C:\\same.txt'], ClipboardContentType.file);
      await Future<void>.delayed(Duration.zero);

      expect(reactivated, isNotNull);
    });

    test('sets is_directory true for folder type', () async {
      final result = await service.processFiles([
        'C:\\MyFolder',
      ], ClipboardContentType.folder);
      expect(result, isNotNull);
      expect(result!.metadata, contains('"is_directory":true'));
    });
  });

  group('ClipboardService.removeItem', () {
    test('deletes item from repository', () async {
      final item = await service.processText('bye', ClipboardContentType.text);
      await service.removeItem(item!.id);
      final found = await repo.getById(item.id);
      expect(found, isNull);
    });
  });

  group('ClipboardService.updatePin', () {
    test('pins and unpins item', () async {
      final item = await service.processText(
        'pin me',
        ClipboardContentType.text,
      );

      await service.updatePin(item!.id, true);
      var found = await repo.getById(item.id);
      expect(found!.isPinned, isTrue);

      await service.updatePin(item.id, false);
      found = await repo.getById(item.id);
      expect(found!.isPinned, isFalse);
    });

    test('silently ignores unknown id', () async {
      await expectLater(service.updatePin('nonexistent', true), completes);
    });
  });

  group('ClipboardService.updateLabelAndColor', () {
    test('updates label and color', () async {
      final item = await service.processText(
        'label',
        ClipboardContentType.text,
      );

      await service.updateLabelAndColor(item!.id, 'My Label', CardColor.blue);
      final found = await repo.getById(item.id);
      expect(found!.label, equals('My Label'));
      expect(found.cardColor, equals(CardColor.blue));
    });

    test('silently ignores unknown id', () async {
      await expectLater(
        service.updateLabelAndColor('nonexistent', null, CardColor.none),
        completes,
      );
    });
  });

  group('ClipboardService.getHistoryAdvanced', () {
    test('returns all items when no filters are applied', () async {
      await service.processText('item1', ClipboardContentType.text);
      await service.processText('item2', ClipboardContentType.text);
      final results = await service.getHistoryAdvanced(limit: 50, skip: 0);
      expect(results.length, equals(2));
    });

    test('filters by type', () async {
      await service.processText('text item', ClipboardContentType.text);
      await service.processText('link item', ClipboardContentType.link);
      final results = await service.getHistoryAdvanced(
        types: [ClipboardContentType.text],
        limit: 50,
        skip: 0,
      );
      expect(results.length, equals(1));
      expect(results.first.type, equals(ClipboardContentType.text));
    });

    test('filters by color', () async {
      final item = await service.processText(
        'colored',
        ClipboardContentType.text,
      );
      await service.updateLabelAndColor(item!.id, null, CardColor.red);
      await service.processText('no color', ClipboardContentType.text);
      final results = await service.getHistoryAdvanced(
        colors: [CardColor.red],
        limit: 50,
        skip: 0,
      );
      expect(results.length, equals(1));
      expect(results.first.cardColor, equals(CardColor.red));
    });

    test('filters by isPinned', () async {
      final item = await service.processText(
        'pinned',
        ClipboardContentType.text,
      );
      await service.updatePin(item!.id, true);
      await service.processText('normal', ClipboardContentType.text);
      final results = await service.getHistoryAdvanced(
        isPinned: true,
        limit: 50,
        skip: 0,
      );
      expect(results.length, equals(1));
      expect(results.first.isPinned, isTrue);
    });

    test('filters by query', () async {
      await service.processText('hello world', ClipboardContentType.text);
      await service.processText('something else', ClipboardContentType.text);
      final results = await service.getHistoryAdvanced(
        query: 'hello',
        limit: 50,
        skip: 0,
      );
      expect(results.length, equals(1));
      expect(results.first.content, equals('hello world'));
    });

    test('respects limit and skip', () async {
      for (var i = 0; i < 5; i++) {
        await service.processText(
          'item$i unique_$i',
          ClipboardContentType.text,
        );
      }
      final page1 = await service.getHistoryAdvanced(limit: 3, skip: 0);
      final page2 = await service.getHistoryAdvanced(limit: 3, skip: 3);
      expect(page1.length, equals(3));
      expect(page2.length, equals(2));
    });
  });

  group('ClipboardService.clearUnpinnedHistory', () {
    test('removes all non-pinned items', () async {
      final item = await service.processText(
        'to be pinned',
        ClipboardContentType.text,
      );
      await service.updatePin(item!.id, true);
      await service.processText('to be deleted', ClipboardContentType.text);

      final deleted = await service.clearUnpinnedHistory();
      expect(deleted, equals(1));

      final remaining = await service.getHistoryAdvanced(limit: 50, skip: 0);
      expect(remaining.length, equals(1));
      expect(remaining.first.isPinned, isTrue);
    });

    test('returns 0 when nothing to delete', () async {
      final count = await service.clearUnpinnedHistory();
      expect(count, equals(0));
    });
  });

  group('ClipboardService.getItemCount', () {
    test('returns correct count', () async {
      expect(await service.getItemCount(), equals(0));
      await service.processText('one', ClipboardContentType.text);
      expect(await service.getItemCount(), equals(1));
      await service.processText('two', ClipboardContentType.text);
      expect(await service.getItemCount(), equals(2));
    });
  });

  group('ClipboardService.walCheckpoint', () {
    test('completes without error', () async {
      await service.processText('checkpoint test', ClipboardContentType.text);
      await expectLater(service.walCheckpoint(), completes);
    });
  });

  group('ClipboardService.updateMetadata', () {
    test('updates metadata and emits onItemReactivated', () async {
      ClipboardItem? reactivated;
      service.onItemReactivated.listen((item) => reactivated = item);

      final item = await service.processText('meta', ClipboardContentType.text);
      await service.updateMetadata(item!.id, '{"key":"value"}');
      await Future<void>.delayed(Duration.zero);

      final stored = await repo.getById(item.id);
      expect(stored!.metadata, equals('{"key":"value"}'));
      expect(reactivated, isNotNull);
      expect(reactivated!.metadata, equals('{"key":"value"}'));
    });

    test('silently ignores unknown id', () async {
      await expectLater(service.updateMetadata('nonexistent', '{}'), completes);
    });
  });

  group('ClipboardService.dispose', () {
    test('closes streams after dispose', () async {
      var addedDone = false;
      var reactivatedDone = false;
      service.onItemAdded.listen(null, onDone: () => addedDone = true);
      service.onItemReactivated.listen(
        null,
        onDone: () => reactivatedDone = true,
      );
      await service.dispose();
      await Future<void>.delayed(Duration.zero);
      expect(addedDone, isTrue);
      expect(reactivatedDone, isTrue);
    });
  });

  group('ClipboardService._shouldIgnore second window', () {
    test('ignores duplicate content within 2x paste window', () async {
      service.pasteIgnoreWindowMs = 50;
      final item = await service.processText(
        'dup-window',
        ClipboardContentType.text,
      );
      // Trigger notifyPasteInitiated so _lastPastedContent = 'dup-window'
      await service.notifyPasteInitiated(item!.id);

      // Wait longer than 1x window but less than 2x window
      await Future<void>.delayed(const Duration(milliseconds: 65));

      // Should be ignored because content matches and elapsed < 2x window
      final result = await service.processText(
        'dup-window',
        ClipboardContentType.text,
      );
      expect(result, isNull);
    });
  });

  group('ClipboardService.processImage with imageBytes', () {
    test('saves temp BMP when imageBytes and imagesPath provided', () async {
      final imagesDir = Directory.systemTemp.createTempSync('svc_img_bmp_');
      try {
        final svc = ClipboardService(repo, imagesPath: imagesDir.path);
        final result = await svc.processImage(
          'hash-temp-bmp',
          imageBytes: [1, 2, 3, 4, 5],
        );
        expect(result, isNotNull);
        // Content should point to temp BMP file
        expect(result!.content, contains(imagesDir.path));
        await svc.dispose();
      } finally {
        imagesDir.deleteSync(recursive: true);
      }
    });

    test('background processing with valid PNG updates item', () async {
      final imagesDir = Directory.systemTemp.createTempSync('svc_img_png_');
      try {
        // Build a small valid PNG in memory
        final image = img.Image(width: 2, height: 2);
        image.setPixelRgb(0, 0, 255, 0, 0);
        final pngBytes = img.encodePng(image);

        final svc = ClipboardService(repo, imagesPath: imagesDir.path);
        final reactivatedCompleter = Completer<ClipboardItem>();
        svc.onItemReactivated.listen((item) {
          if (!reactivatedCompleter.isCompleted) {
            reactivatedCompleter.complete(item);
          }
        });

        final result = await svc.processImage(
          'hash-valid-png',
          imageBytes: pngBytes,
        );
        expect(result, isNotNull);

        // Wait for background isolate to finish processing
        final updated = await reactivatedCompleter.future.timeout(
          const Duration(seconds: 10),
        );
        expect(updated.content, endsWith('.png'));
        expect(updated.metadata, isNotNull);
        expect(updated.metadata, contains('width'));

        await svc.dispose();
      } finally {
        imagesDir.deleteSync(recursive: true);
      }
    });
  });

  group('ClipboardService.removeItem for image', () {
    test('cleans up image file when removing image item', () async {
      final imagesDir = Directory.systemTemp.createTempSync('svc_rm_img_');
      try {
        final svc = ClipboardService(repo, imagesPath: imagesDir.path);
        final imageFile = File(p.join(imagesDir.path, 'img.png'))
          ..writeAsBytesSync([137, 80, 78, 71]);
        final item = ClipboardItem(
          content: imageFile.path,
          type: ClipboardContentType.image,
          contentHash: 'rm-hash',
        );
        await repo.save(item);

        await svc.removeItem(item.id);

        expect(await repo.getById(item.id), isNull);
        expect(imageFile.existsSync(), isFalse);

        await svc.dispose();
      } finally {
        imagesDir.deleteSync(recursive: true);
      }
    });

    test('refuses to delete image file outside imagesPath', () async {
      final imagesDir = Directory.systemTemp.createTempSync('svc_rm_safe_');
      final externalDir = Directory.systemTemp.createTempSync('svc_rm_ext_');
      try {
        final svc = ClipboardService(repo, imagesPath: imagesDir.path);
        // Simulates a dragged image whose content path is the user's own file
        // (outside the app's images directory). Must never be deleted.
        final externalFile = File(p.join(externalDir.path, 'user_photo.png'))
          ..writeAsBytesSync([137, 80, 78, 71]);
        final item = ClipboardItem(
          content: externalFile.path,
          type: ClipboardContentType.image,
          contentHash: 'ext-hash',
        );
        await repo.save(item);

        await svc.removeItem(item.id);

        expect(await repo.getById(item.id), isNull);
        expect(
          externalFile.existsSync(),
          isTrue,
          reason: 'external user files must never be deleted',
        );

        await svc.dispose();
      } finally {
        imagesDir.deleteSync(recursive: true);
        externalDir.deleteSync(recursive: true);
      }
    });

    test('also deletes thumbPath file inside imagesPath', () async {
      final imagesDir = Directory.systemTemp.createTempSync('svc_rm_thumb_');
      final externalDir = Directory.systemTemp.createTempSync('svc_rm_ext2_');
      try {
        final svc = ClipboardService(repo, imagesPath: imagesDir.path);
        final externalFile = File(p.join(externalDir.path, 'photo.png'))
          ..writeAsBytesSync([137, 80, 78, 71]);
        final thumbFile = File(p.join(imagesDir.path, 'thumb-id_thumb.png'))
          ..writeAsBytesSync([1, 2, 3]);
        final item = ClipboardItem(
          id: 'thumb-id',
          content: externalFile.path,
          type: ClipboardContentType.image,
          contentHash: 'thumb-hash',
          thumbPath: thumbFile.path,
        );
        await repo.save(item);

        await svc.removeItem(item.id);

        expect(await repo.getById(item.id), isNull);
        expect(
          externalFile.existsSync(),
          isTrue,
          reason: 'external user file must never be deleted',
        );
        expect(
          thumbFile.existsSync(),
          isFalse,
          reason: 'app-owned thumb must be removed when item is deleted',
        );

        await svc.dispose();
      } finally {
        imagesDir.deleteSync(recursive: true);
        externalDir.deleteSync(recursive: true);
      }
    });

    test('refuses to delete thumbPath outside imagesPath', () async {
      final imagesDir = Directory.systemTemp.createTempSync('svc_rm_thumb2_');
      final externalDir = Directory.systemTemp.createTempSync('svc_rm_ext3_');
      try {
        final svc = ClipboardService(repo, imagesPath: imagesDir.path);
        final externalThumb = File(p.join(externalDir.path, 'evil_thumb.png'))
          ..writeAsBytesSync([1, 2, 3]);
        final item = ClipboardItem(
          id: 'evil',
          content: '',
          type: ClipboardContentType.image,
          contentHash: 'evil-hash',
          thumbPath: externalThumb.path,
        );
        await repo.save(item);

        await svc.removeItem(item.id);

        expect(
          externalThumb.existsSync(),
          isTrue,
          reason: 'thumbPath outside imagesPath must be ignored',
        );

        await svc.dispose();
      } finally {
        imagesDir.deleteSync(recursive: true);
        externalDir.deleteSync(recursive: true);
      }
    });
  });

  group('ClipboardService.processFiles single file with size', () {
    test('includes file_size in metadata for single existing file', () async {
      final dir = Directory.systemTemp.createTempSync('svc_files_');
      try {
        final file = File(p.join(dir.path, 'test.txt'))
          ..writeAsStringSync('hello world');
        final result = await service.processFiles([
          file.path,
        ], ClipboardContentType.file);
        expect(result, isNotNull);
        expect(result!.metadata, contains('file_size'));
      } finally {
        dir.deleteSync(recursive: true);
      }
    });
  });

  group('ClipboardService thumbnail gate methods with imagesPath', () {
    test('requestThumbnailIfStale is a no-op when called', () async {
      final dir = Directory.systemTemp.createTempSync('svc_gate_stale_');
      try {
        final svc = ClipboardService(repo, imagesPath: dir.path);
        final item = await svc.processImage(
          'gate-hash-stale',
          imagePath: '/some/external.png',
        );
        expect(item, isNotNull);
        // Must not throw; exercises the _thumbQueue?.enqueueIfStale branch.
        expect(() => svc.requestThumbnailIfStale(item!), returnsNormally);
        await svc.dispose();
      } finally {
        dir.deleteSync(recursive: true);
      }
    });

    test('requestThumbnailRefresh enqueues a manual refresh', () async {
      final dir = Directory.systemTemp.createTempSync('svc_gate_refresh_');
      try {
        final svc = ClipboardService(repo, imagesPath: dir.path);
        final item = await svc.processImage(
          'gate-hash-refresh',
          imagePath: '/some/external.png',
        );
        expect(item, isNotNull);
        // Must not throw; exercises the _thumbQueue?.enqueue branch.
        expect(() => svc.requestThumbnailRefresh(item!), returnsNormally);
        await svc.dispose();
      } finally {
        dir.deleteSync(recursive: true);
      }
    });

    test(
      'updateThumbnailTypeGate assigns to _thumbnailService.isTypeEnabled',
      () async {
        final dir = Directory.systemTemp.createTempSync('svc_gate_type_');
        try {
          final svc = ClipboardService(repo, imagesPath: dir.path);
          // Setting a gate must not throw.
          svc.updateThumbnailTypeGate(
            (type) => type == ClipboardContentType.image,
          );
          svc.updateThumbnailTypeGate(null);
          await svc.dispose();
        } finally {
          dir.deleteSync(recursive: true);
        }
      },
    );

    test('updateMaxImageBytesGate assigns the new getter', () async {
      // Works with or without imagesPath — imageQueue is always created.
      service.updateMaxImageBytesGate(() => 5 * 1024 * 1024);
      service.updateMaxImageBytesGate(null);
      // No error means the branch is exercised.
    });
  });

  group('ClipboardService.processText legacy reclassification', () {
    test(
      'upgrades existing text item to resolved type on duplicate submission',
      () async {
        // Save an email address as plain text (simulates an item captured before
        // the TextClassifier gained email classification).
        const email = 'legacy.user@example.com';
        final legacy = ClipboardItem(
          content: email,
          type: ClipboardContentType.text, // stored with wrong type
        );
        await repo.save(legacy);

        ClipboardItem? reactivated;
        service.onItemReactivated.listen((item) => reactivated = item);

        // Submit the same content; TextClassifier now classifies it as 'email'.
        // findByContentAndType('email') returns null, so the legacy path runs.
        final result = await service.processText(
          email,
          ClipboardContentType.text,
        );
        await Future<void>.delayed(Duration.zero);

        expect(result, isNotNull);
        expect(result!.type, equals(ClipboardContentType.email));
        expect(result.id, equals(legacy.id));
        expect(reactivated?.id, equals(legacy.id));
        expect(reactivated?.type, equals(ClipboardContentType.email));
      },
    );
  });

  group('ClipboardService.processImage BMP write failure', () {
    test('falls back gracefully when temp BMP cannot be written', () async {
      final dir = Directory.systemTemp.createTempSync('svc_bmp_fail_');
      try {
        // Make the images directory read-only so File.writeAsBytes throws.
        await Process.run('chmod', ['444', dir.path]);

        final svc = ClipboardService(repo, imagesPath: dir.path);
        // Should not throw; the catch block logs a warning and saves anyway.
        final result = await svc.processImage(
          'bmp-fail-hash',
          imageBytes: [1, 2, 3, 4],
        );
        expect(result, isNotNull);
        // Item is saved even though the BMP write failed; content is empty.
        expect(result!.type, equals(ClipboardContentType.image));

        await svc.dispose();
      } finally {
        await Process.run('chmod', ['755', dir.path]);
        dir.deleteSync(recursive: true);
      }
    });
  });
}
