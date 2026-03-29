import 'package:flutter_test/flutter_test.dart';

import 'package:core/core.dart';

void main() {
  late SqliteRepository repo;
  late ClipboardService service;

  setUp(() {
    repo = SqliteRepository.inMemory();
    service = ClipboardService(repo);
  });

  tearDown(() async {
    service.dispose();
    await repo.close();
  });

  group('ClipboardService.reclassifyLegacyTextItems', () {
    test('reclassifies legacy text item to email', () async {
      // Insert a ClipboardItem already stored as plain text with email content
      final item = ClipboardItem(
        content: 'user@example.com',
        type: ClipboardContentType.text,
      );
      await repo.save(item);

      await service.reclassifyLegacyTextItems();

      final updated = await repo.getById(item.id);
      expect(updated, isNotNull);
      expect(updated!.type, equals(ClipboardContentType.email));
    });

    test('reclassifies legacy text item to uuid', () async {
      final item = ClipboardItem(
        content: '550e8400-e29b-41d4-a716-446655440000',
        type: ClipboardContentType.text,
      );
      await repo.save(item);

      await service.reclassifyLegacyTextItems();

      final updated = await repo.getById(item.id);
      expect(updated!.type, equals(ClipboardContentType.uuid));
    });

    test('reclassifies legacy text item to ip', () async {
      final item = ClipboardItem(
        content: '192.168.1.1',
        type: ClipboardContentType.text,
      );
      await repo.save(item);

      await service.reclassifyLegacyTextItems();

      final updated = await repo.getById(item.id);
      expect(updated!.type, equals(ClipboardContentType.ip));
    });

    test('reclassifies legacy text item to color', () async {
      final item = ClipboardItem(
        content: '#FF5733',
        type: ClipboardContentType.text,
      );
      await repo.save(item);

      await service.reclassifyLegacyTextItems();

      final updated = await repo.getById(item.id);
      expect(updated!.type, equals(ClipboardContentType.color));
    });

    test('reclassifies legacy text item to json', () async {
      final item = ClipboardItem(
        content: '{"key": "value"}',
        type: ClipboardContentType.text,
      );
      await repo.save(item);

      await service.reclassifyLegacyTextItems();

      final updated = await repo.getById(item.id);
      expect(updated!.type, equals(ClipboardContentType.json));
    });

    test('does not reclassify plain text items', () async {
      final item = ClipboardItem(
        content: 'just some plain text here',
        type: ClipboardContentType.text,
      );
      await repo.save(item);

      await service.reclassifyLegacyTextItems();

      final unchanged = await repo.getById(item.id);
      expect(unchanged!.type, equals(ClipboardContentType.text));
    });

    test('does not touch non-text typed items', () async {
      final item = ClipboardItem(
        content: 'already-email@domain.com',
        type: ClipboardContentType.email,
      );
      await repo.save(item);

      await service.reclassifyLegacyTextItems();

      final unchanged = await repo.getById(item.id);
      expect(unchanged!.type, equals(ClipboardContentType.email));
    });

    test('processes batch of items spanning multiple pages', () async {
      // Insert more than one batch (batchSize = 50) of text items
      for (var i = 0; i < 55; i++) {
        await repo.save(
          ClipboardItem(
            content: 'plain text item number $i',
            type: ClipboardContentType.text,
          ),
        );
      }
      // Add a few that should be reclassified
      final emailItem = ClipboardItem(
        content: 'batch@test.com',
        type: ClipboardContentType.text,
      );
      final ipItem = ClipboardItem(
        content: '10.0.0.1',
        type: ClipboardContentType.text,
      );
      await repo.save(emailItem);
      await repo.save(ipItem);

      await service.reclassifyLegacyTextItems();

      final updatedEmail = await repo.getById(emailItem.id);
      final updatedIp = await repo.getById(ipItem.id);
      expect(updatedEmail!.type, equals(ClipboardContentType.email));
      expect(updatedIp!.type, equals(ClipboardContentType.ip));
    });

    test('completes gracefully when repository is empty', () async {
      await expectLater(service.reclassifyLegacyTextItems(), completes);
    });

    test('stops reclassifying when disposed mid-batch', () async {
      for (var i = 0; i < 10; i++) {
        await repo.save(
          ClipboardItem(
            content: 'item$i@example.com',
            type: ClipboardContentType.text,
          ),
        );
      }
      // Dispose immediately — should not throw
      service.dispose();
      await expectLater(service.reclassifyLegacyTextItems(), completes);
    });

    test('reclassifies phone number stored as text', () async {
      final item = ClipboardItem(
        content: '+56 9 1234 5678',
        type: ClipboardContentType.text,
      );
      await repo.save(item);

      await service.reclassifyLegacyTextItems();

      final updated = await repo.getById(item.id);
      expect(updated!.type, equals(ClipboardContentType.phone));
    });
  });
}
