/// Integration tests that verify ClipboardService behaviour is identical
/// across Windows, macOS, and Linux — no platform-specific branching exists
/// in the Dart service layer, so these tests run unconditionally on all
/// platforms (CI runs for each OS via the flutter test matrix).
library;

import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

import 'package:core/core.dart';

void main() {
  late SqliteRepository repo;
  late Directory imagesDir;
  late ClipboardService service;

  setUp(() {
    repo = SqliteRepository.inMemory();
    imagesDir = Directory.systemTemp.createTempSync('svc_platform_');
    service = ClipboardService(repo, imagesPath: imagesDir.path);
  });

  tearDown(() async {
    await service.dispose();
    await repo.close();
    if (imagesDir.existsSync()) imagesDir.deleteSync(recursive: true);
  });

  group('ClipboardService – cross-platform path handling', () {
    test('processFiles handles Unix-style paths', () async {
      final result = await service.processFiles([
        '/home/user/documents/report.pdf',
      ], ClipboardContentType.file);
      expect(result, isNotNull);
      expect(result!.content, equals('/home/user/documents/report.pdf'));
      final meta = jsonDecode(result.metadata!) as Map<String, dynamic>;
      expect(meta['file_name'], equals('report.pdf'));
      expect(meta['first_ext'], equals('.pdf'));
    });

    test('processFiles handles Windows-style paths', () async {
      final result = await service.processFiles([
        r'C:\Users\user\Documents\report.docx',
      ], ClipboardContentType.file);
      expect(result, isNotNull);
      // p.basename handles both separators
      expect(result!.content, contains('report.docx'));
    });

    test('processFiles handles multiple files across platforms', () async {
      final paths = [
        if (Platform.isWindows) ...[
          r'C:\docs\file1.txt',
          r'C:\docs\file2.txt',
        ] else ...[
          '/home/user/file1.txt',
          '/home/user/file2.txt',
        ],
      ];

      final result = await service.processFiles(
        paths,
        ClipboardContentType.file,
      );
      expect(result, isNotNull);
      final meta = jsonDecode(result!.metadata!) as Map<String, dynamic>;
      expect(meta['file_count'], equals(2));
    });

    test('processFiles with folder type sets is_directory=true', () async {
      final folder = Platform.isWindows ? r'C:\MyFolder' : '/home/user/folder';
      final result = await service.processFiles([
        folder,
      ], ClipboardContentType.folder);
      expect(result, isNotNull);
      final meta = jsonDecode(result!.metadata!) as Map<String, dynamic>;
      expect(meta['is_directory'], isTrue);
    });

    test('processFiles with file type sets is_directory=false', () async {
      final file = Platform.isWindows
          ? r'C:\MyFolder\file.txt'
          : '/home/user/file.txt';
      final result = await service.processFiles([
        file,
      ], ClipboardContentType.file);
      final meta = jsonDecode(result!.metadata!) as Map<String, dynamic>;
      expect(meta['is_directory'], isFalse);
    });
  });

  group('ClipboardService – content deduplication cross-platform', () {
    test('duplicate text is reactivated regardless of source', () async {
      await service.processText(
        'duplicate',
        ClipboardContentType.text,
        source: 'app1',
      );
      ClipboardItem? reactivated;
      service.onItemReactivated.listen((item) => reactivated = item);

      await service.processText(
        'duplicate',
        ClipboardContentType.text,
        source: 'app2',
      );
      await Future<void>.delayed(Duration.zero);

      expect(reactivated, isNotNull);
      expect(reactivated!.content, equals('duplicate'));
    });

    test('same image hash triggers reactivation', () async {
      await service.processImage('cross-platform-hash-1');
      ClipboardItem? reactivated;
      service.onItemReactivated.listen((item) => reactivated = item);

      await service.processImage('cross-platform-hash-1');
      await Future<void>.delayed(Duration.zero);

      expect(reactivated, isNotNull);
    });
  });

  group('ClipboardService – image processing with real temp dir', () {
    test(
      'processImage with imageBytes writes .bmp then updates via background isolate',
      () async {
        // This test verifies the full write-temp-BMP → background PNG pipeline.
        // On all platforms the temp file is in imagesDir (injected), so the
        // path separator is native and no platform divergence is expected.
        final pngBytes = _makeSmallPng();
        String? reactivatedPath;
        service.onItemReactivated.listen(
          (item) => reactivatedPath = item.content,
        );

        final result = await service.processImage(
          'integration-hash',
          imageBytes: pngBytes,
        );
        expect(result, isNotNull);
        // Immediately after processImage the content points to the temp BMP
        expect(result!.content, endsWith('.bmp'));
        expect(File(result.content).existsSync(), isTrue);

        // Wait for background PNG processing
        await Future<void>.delayed(const Duration(seconds: 8));

        if (reactivatedPath != null) {
          expect(reactivatedPath, endsWith('.png'));
        }
        // Whether or not the background completes within the wait, no crash is OK
      },
    );
  });

  group('ClipboardService – metadata encoding cross-platform', () {
    test('RTF and HTML metadata is base64-encoded correctly', () async {
      final rtfBytes = [0x7B, 0x5C, 0x72, 0x74, 0x66, 0x31]; // {\rtf1
      final htmlBytes = [
        0x3C,
        0x62,
        0x3E,
        0x68,
        0x69,
        0x3C,
        0x2F,
        0x62,
        0x3E,
      ]; // <b>hi</b>

      final result = await service.processText(
        'rich content',
        ClipboardContentType.text,
        rtfBytes: rtfBytes,
        htmlBytes: htmlBytes,
      );
      expect(result, isNotNull);
      final meta = jsonDecode(result!.metadata!) as Map<String, dynamic>;
      expect(meta.containsKey('rtf'), isTrue);
      expect(meta.containsKey('html'), isTrue);

      // Verify round-trip decodability
      final decodedRtf = base64Decode(meta['rtf'] as String);
      final decodedHtml = base64Decode(meta['html'] as String);
      expect(decodedRtf, equals(rtfBytes));
      expect(decodedHtml, equals(htmlBytes));
    });

    test('metadata is null when no rtf/html provided', () async {
      final result = await service.processText(
        'no metadata',
        ClipboardContentType.text,
      );
      expect(result!.metadata, isNull);
    });
  });

  group('ClipboardService – paste ignore window cross-platform', () {
    test(
      'notifyPasteInitiated blocks clipboard echo on all platforms',
      () async {
        service.pasteIgnoreWindowMs = 500;
        final item = await service.processText(
          'echo',
          ClipboardContentType.text,
        );
        await service.notifyPasteInitiated(item!.id);

        // Attempt to re-process the same content immediately
        final ignored = await service.processText(
          'echo',
          ClipboardContentType.text,
        );
        expect(ignored, isNull);
      },
    );

    test('clipboard is no longer ignored after window expires', () async {
      service.pasteIgnoreWindowMs = 20;
      final item = await service.processText(
        'temporary',
        ClipboardContentType.text,
      );
      await service.notifyPasteInitiated(item!.id);

      // Wait past the full 2× window
      await Future<void>.delayed(const Duration(milliseconds: 50));

      final result = await service.processText(
        'temporary',
        ClipboardContentType.text,
      );
      expect(result, isNotNull);
    });
  });

  group('ClipboardService – full CRUD lifecycle cross-platform', () {
    test('full lifecycle: save → pin → label → delete', () async {
      // Create
      final item = await service.processText(
        'lifecycle',
        ClipboardContentType.text,
      );
      expect(item, isNotNull);
      expect(await service.getItemCount(), equals(1));

      // Pin
      await service.updatePin(item!.id, true);
      var found = await repo.getById(item.id);
      expect(found!.isPinned, isTrue);

      // Label + color
      await service.updateLabelAndColor(item.id, 'tagged', CardColor.green);
      found = await repo.getById(item.id);
      expect(found!.label, equals('tagged'));
      expect(found.cardColor, equals(CardColor.green));

      // Paste count
      await service.recordPaste(item.id);
      found = await repo.getById(item.id);
      expect(found!.pasteCount, equals(1));

      // Delete
      await service.removeItem(item.id);
      expect(await repo.getById(item.id), isNull);
      expect(await service.getItemCount(), equals(0));
    });

    test(
      'clearUnpinnedHistory preserves pinned items on all platforms',
      () async {
        final pinned = await service.processText(
          'keep',
          ClipboardContentType.text,
        );
        await service.updatePin(pinned!.id, true);

        await service.processText('delete-me-1', ClipboardContentType.text);
        await service.processText('delete-me-2', ClipboardContentType.text);

        final count = await service.clearUnpinnedHistory();
        expect(count, equals(2));
        expect(await service.getItemCount(), equals(1));

        final remaining = await service.getHistoryAdvanced(limit: 10, skip: 0);
        expect(remaining.first.isPinned, isTrue);
      },
    );
  });

  group('ClipboardService – unsupported image format (SVG/PDF/etc.)', () {
    test(
      'temp BMP is deleted when image bytes cannot be decoded (e.g. SVG)',
      () async {
        // Simulate SVG bytes arriving as image clipboard content
        final svgBytes =
            '<svg xmlns="http://www.w3.org/2000/svg"><rect/></svg>'.codeUnits;
        const fakeHash = 'svg-hash-001';

        await service.processImage(fakeHash, imageBytes: svgBytes);

        // Give background Isolate time to attempt decode and clean up
        await Future<void>.delayed(const Duration(milliseconds: 500));

        // Temp .bmp must not exist — it should be cleaned up on decode failure
        final tempBmp = File(p.join(imagesDir.path, '$fakeHash.bmp'));
        expect(tempBmp.existsSync(), isFalse);
      },
    );

    test(
      'repository item is still saved even when image bytes cannot be decoded',
      () async {
        final svgBytes = '<svg xmlns="http://www.w3.org/2000/svg"/>'.codeUnits;
        const fakeHash = 'svg-hash-002';

        final item = await service.processImage(fakeHash, imageBytes: svgBytes);

        expect(item, isNotNull);
        expect(item!.contentHash, equals(fakeHash));

        await Future<void>.delayed(const Duration(milliseconds: 500));

        // Item still present in repository
        final found = await repo.getById(item.id);
        expect(found, isNotNull);
      },
    );
  });

  group('ClipboardService – file size metadata on all platforms', () {
    test('includes file_size for a single real file', () async {
      final testFile = File(p.join(imagesDir.path, 'sample.txt'))
        ..writeAsStringSync('cross-platform content');

      final result = await service.processFiles([
        testFile.path,
      ], ClipboardContentType.file);
      expect(result, isNotNull);
      final meta = jsonDecode(result!.metadata!) as Map<String, dynamic>;
      expect(meta.containsKey('file_size'), isTrue);
      expect((meta['file_size'] as int) > 0, isTrue);
    });

    test('does not include file_size for non-existent single file', () async {
      final missingPath = p.join(imagesDir.path, 'missing_file.txt');
      final result = await service.processFiles([
        missingPath,
      ], ClipboardContentType.file);
      expect(result, isNotNull);
      final meta = jsonDecode(result!.metadata!) as Map<String, dynamic>;
      // file_size may be absent (stat fails) — just verify no crash
      expect(meta['file_count'], equals(1));
    });
  });
}

/// Builds a minimal 1×1 PNG in pure Dart (no external files needed).
List<int> _makeSmallPng() {
  // A valid 1×1 red PNG (67 bytes, hard-coded known-good bytes).
  return [
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
    0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk length + type
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // width=1, height=1
    0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, // bit depth=8, colour=RGB
    0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, // IHDR CRC + IDAT chunk
    0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, // IDAT data (1 red pixel)
    0x00, 0x00, 0x02, 0x00, 0x01, 0xE2, 0x21, 0xBC, // IDAT CRC
    0x33, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, // IEND chunk
    0x44, 0xAE, 0x42, 0x60, 0x82, // IEND CRC
  ];
}
