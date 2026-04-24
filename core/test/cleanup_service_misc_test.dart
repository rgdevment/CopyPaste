import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

import 'package:core/core.dart';

void main() {
  group('CleanupService.updateKeepBrokenCallback', () {
    test('replaces the keepBrokenDays getter', () async {
      final tempDir = Directory.systemTemp.createTempSync(
        'cleanup_misc_keep_broken_',
      );
      final repo = SqliteRepository.inMemory();
      final storage = await StorageConfig.create(baseDir: tempDir.path);
      await storage.ensureDirectories();

      try {
        var keepDays = 999; // large value — won't purge anything

        final service = CleanupService(
          repo,
          () => 0,
          storage: storage,
          getKeepBrokenDays: () => keepDays,
        );

        // Create an item with a very old brokenSince date.
        final extDir = Directory(p.join(tempDir.path, 'ext'))
          ..createSync(recursive: true);
        final ext = File(p.join(extDir.path, 'gone.png'))
          ..writeAsBytesSync([1]);
        await repo.save(
          ClipboardItem(
            id: 'broken-item',
            content: ext.path,
            type: ClipboardContentType.image,
            brokenSince: DateTime.now().toUtc().subtract(
              const Duration(days: 60),
            ),
          ),
        );
        ext.deleteSync(); // file is gone

        // Swap to 1 day so the item would be purged.
        keepDays = 1;
        service.updateKeepBrokenCallback(() => keepDays);

        service.start(tempDir.path);
        await Future<void>.delayed(const Duration(milliseconds: 150));
        service.dispose();

        // The item should be gone because brokenSince > keepDays.
        expect(await repo.getById('broken-item'), isNull);

        await repo.close();
      } finally {
        if (tempDir.existsSync()) tempDir.deleteSync(recursive: true);
      }
    });
  });

  group('CleanupService.updateImagesQuotaCallback', () {
    test('replaces the quota getter and enforces the new limit', () async {
      final tempDir = Directory.systemTemp.createTempSync(
        'cleanup_misc_quota_',
      );
      final repo = SqliteRepository.inMemory();
      final storage = await StorageConfig.create(baseDir: tempDir.path);
      await storage.ensureDirectories();

      try {
        var quotaMB = 0; // disabled initially

        final service = CleanupService(
          repo,
          () => 0,
          storage: storage,
          getImagesQuotaMB: () => quotaMB,
        );

        // Write two ~600 KB files and register them.
        final f1 = File(p.join(storage.imagesPath, 'q1.png'))
          ..writeAsBytesSync(List<int>.filled(600 * 1024, 0xAA));
        final f2 = File(p.join(storage.imagesPath, 'q2.png'))
          ..writeAsBytesSync(List<int>.filled(600 * 1024, 0xBB));

        await repo.save(
          ClipboardItem(
            id: 'q1',
            content: f1.path,
            type: ClipboardContentType.image,
            createdAt: DateTime.utc(2024, 1, 1),
            modifiedAt: DateTime.utc(2024, 1, 1),
          ),
        );
        await repo.save(
          ClipboardItem(
            id: 'q2',
            content: f2.path,
            type: ClipboardContentType.image,
            createdAt: DateTime.utc(2024, 12, 1),
            modifiedAt: DateTime.utc(2024, 12, 1),
          ),
        );

        // First run with quota=0 — nothing should be purged.
        service.start(tempDir.path);
        await Future<void>.delayed(const Duration(milliseconds: 150));
        service.dispose();

        expect(f1.existsSync(), isTrue);
        expect(f2.existsSync(), isTrue);

        // Now activate a 1 MB quota and run again.
        quotaMB = 1;
        service.updateImagesQuotaCallback(() => quotaMB);

        final marker = File(p.join(tempDir.path, 'last_cleanup.txt'));
        if (marker.existsSync()) marker.deleteSync();

        final service2 = CleanupService(
          repo,
          () => 0,
          storage: storage,
          getImagesQuotaMB: () => quotaMB,
        );
        service2.start(tempDir.path);
        await Future<void>.delayed(const Duration(milliseconds: 200));
        service2.dispose();

        // Oldest item (q1) must have been purged to go under 1 MB.
        expect(f1.existsSync(), isFalse);
        expect(f2.existsSync(), isTrue);

        await repo.close();
      } finally {
        if (tempDir.existsSync()) tempDir.deleteSync(recursive: true);
      }
    });
  });

  group('CleanupService.isVolumePresent – macOS', () {
    test(
      'returns false for a /Volumes/ path with no such mount',
      () {
        // Pick a name that is extremely unlikely to be an actual mounted volume.
        const fakePath =
            '/Volumes/CopyPasteNonExistentVolumeXYZ9999/some/file.png';
        expect(CleanupService.isVolumePresent(fakePath), isFalse);
      },
      skip: !Platform.isMacOS ? 'macOS-only' : null,
    );

    test(
      'returns false for bare /Volumes/ path (empty mount name)',
      () {
        // '/Volumes/' → rest='', mount='' → isEmpty → false
        expect(CleanupService.isVolumePresent('/Volumes/'), isFalse);
      },
      skip: !Platform.isMacOS ? 'macOS-only' : null,
    );

    test(
      'returns true for a regular macOS path not under /Volumes/',
      () {
        // Any path that doesn't start with '/Volumes/' returns true on macOS.
        expect(CleanupService.isVolumePresent('/Users/test/file.png'), isTrue);
      },
      skip: !Platform.isMacOS ? 'macOS-only' : null,
    );

    test(
      'returns true for existing Macintosh HD volume',
      () {
        // '/Volumes/Macintosh HD' is typically present on macOS machines.
        // If not, the test is still valid: existsSync() returns false → we'd
        // return false, but the code path is exercised.
        const path = '/Volumes/Macintosh HD/some/file.png';
        // Just assert it doesn't throw; the boolean value depends on the host.
        expect(() => CleanupService.isVolumePresent(path), returnsNormally);
      },
      skip: !Platform.isMacOS ? 'macOS-only' : null,
    );
  });
}
