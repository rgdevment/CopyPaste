import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'package:core/core.dart';

void main() {
  late SqliteRepository repo;
  late Directory tempDir;

  setUp(() {
    repo = SqliteRepository.inMemory();
    tempDir = Directory.systemTemp.createTempSync('cleanup_test_');
  });

  tearDown(() async {
    await repo.close();
    if (tempDir.existsSync()) tempDir.deleteSync(recursive: true);
  });

  group('CleanupService', () {
    test('runCleanupIfNeeded via start() clears old items', () async {
      await repo.save(ClipboardItem(
        content: 'old',
        type: ClipboardContentType.text,
        createdAt:
            DateTime.now().toUtc().subtract(const Duration(days: 40)),
        modifiedAt:
            DateTime.now().toUtc().subtract(const Duration(days: 40)),
      ));

      final service = CleanupService(repo, () => 30);
      service.start(tempDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 50));
      service.dispose();

      final count = await repo.count();
      expect(count, equals(0));
    });

    test('skips cleanup when same day as last run', () async {
      final file = File('${tempDir.path}/last_cleanup.txt');
      file.writeAsStringSync(DateTime.now().toUtc().toIso8601String());

      await repo.save(ClipboardItem(
        content: 'old',
        type: ClipboardContentType.text,
        createdAt:
            DateTime.now().toUtc().subtract(const Duration(days: 40)),
        modifiedAt:
            DateTime.now().toUtc().subtract(const Duration(days: 40)),
      ));

      final service = CleanupService(repo, () => 30);
      service.start(tempDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 50));
      service.dispose();

      // Item should still be there because cleanup was skipped
      final count = await repo.count();
      expect(count, equals(1));
    });

    test('skips cleanup when retentionDays is 0', () async {
      await repo.save(ClipboardItem(
        content: 'old',
        type: ClipboardContentType.text,
        createdAt:
            DateTime.now().toUtc().subtract(const Duration(days: 40)),
        modifiedAt:
            DateTime.now().toUtc().subtract(const Duration(days: 40)),
      ));

      final service = CleanupService(repo, () => 0);
      service.start(tempDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 50));
      service.dispose();

      final count = await repo.count();
      expect(count, equals(1));
    });

    test('skips cleanup when retentionDays is negative', () async {
      await repo.save(ClipboardItem(
        content: 'item',
        type: ClipboardContentType.text,
        createdAt:
            DateTime.now().toUtc().subtract(const Duration(days: 100)),
        modifiedAt:
            DateTime.now().toUtc().subtract(const Duration(days: 100)),
      ));

      final service = CleanupService(repo, () => -1);
      service.start(tempDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 50));
      service.dispose();

      final count = await repo.count();
      expect(count, equals(1));
    });

    test('preserves pinned items during cleanup', () async {
      final pinned = ClipboardItem(
        content: 'pinned old',
        type: ClipboardContentType.text,
        isPinned: true,
        createdAt:
            DateTime.now().toUtc().subtract(const Duration(days: 40)),
        modifiedAt:
            DateTime.now().toUtc().subtract(const Duration(days: 40)),
      );
      await repo.save(pinned);

      final service = CleanupService(repo, () => 30);
      service.start(tempDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 50));
      service.dispose();

      final found = await repo.getById(pinned.id);
      expect(found, isNotNull);
    });

    test('writes last cleanup date to file after running', () async {
      final service = CleanupService(repo, () => 30);
      service.start(tempDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 50));
      service.dispose();

      final file = File('${tempDir.path}/last_cleanup.txt');
      expect(file.existsSync(), isTrue);

      final content = file.readAsStringSync().trim();
      final parsed = DateTime.tryParse(content);
      expect(parsed, isNotNull);
      // Should be today's date
      final now = DateTime.now().toUtc();
      expect(parsed!.year, equals(now.year));
      expect(parsed.month, equals(now.month));
      expect(parsed.day, equals(now.day));
    });

    test('runs cleanup with previous-day date file', () async {
      final yesterday =
          DateTime.now().toUtc().subtract(const Duration(days: 1));
      final file = File('${tempDir.path}/last_cleanup.txt');
      file.writeAsStringSync(yesterday.toIso8601String());

      await repo.save(ClipboardItem(
        content: 'old',
        type: ClipboardContentType.text,
        createdAt:
            DateTime.now().toUtc().subtract(const Duration(days: 40)),
        modifiedAt:
            DateTime.now().toUtc().subtract(const Duration(days: 40)),
      ));

      final service = CleanupService(repo, () => 30);
      service.start(tempDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 50));
      service.dispose();

      final count = await repo.count();
      expect(count, equals(0));
    });

    test('dispose prevents further cleanup', () async {
      final service = CleanupService(repo, () => 30);
      service.dispose();
      // After dispose, runCleanupIfNeeded is a no-op

      await repo.save(ClipboardItem(
        content: 'old',
        type: ClipboardContentType.text,
        createdAt:
            DateTime.now().toUtc().subtract(const Duration(days: 40)),
        modifiedAt:
            DateTime.now().toUtc().subtract(const Duration(days: 40)),
      ));

      await service.runCleanupIfNeeded();

      final count = await repo.count();
      expect(count, equals(1)); // item was NOT deleted
    });

    test('does not crash on missing base dir', () async {
      final service = CleanupService(repo, () => 30);
      // Do NOT call start(), baseDirPath is null
      // runCleanupIfNeeded uses '' as basePath — may fail file write gracefully
      await expectLater(service.runCleanupIfNeeded(), completes);
      service.dispose();
    });
  });
}
