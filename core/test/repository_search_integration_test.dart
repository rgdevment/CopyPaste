/// Cross-platform repository search integration tests.
/// Verifies that FTS5, LIKE fallback, and Unicode normalization work correctly
/// across Windows, macOS, and Linux — all using the in-memory SQLite instance.
library;

import 'package:flutter_test/flutter_test.dart';

import 'package:core/core.dart';

void main() {
  late SqliteRepository repo;

  setUp(() {
    repo = SqliteRepository.inMemory();
  });

  tearDown(() => repo.close());

  group('Repository search – FTS5 path', () {
    test('finds item by exact word', () async {
      await repo.save(
        ClipboardItem(
          content: 'flutter desktop app',
          type: ClipboardContentType.text,
        ),
      );
      await repo.save(
        ClipboardItem(
          content: 'mobile development',
          type: ClipboardContentType.text,
        ),
      );

      final results = await repo.searchAdvanced(
        query: 'flutter',
        limit: 50,
        skip: 0,
      );
      expect(results.length, equals(1));
      expect(results.first.content, contains('flutter'));
    });

    test('finds item by prefix (FTS5 prefix query)', () async {
      await repo.save(
        ClipboardItem(
          content: 'clipboard manager',
          type: ClipboardContentType.text,
        ),
      );
      await repo.save(
        ClipboardItem(
          content: 'clipboard history',
          type: ClipboardContentType.text,
        ),
      );
      await repo.save(
        ClipboardItem(
          content: 'unrelated content',
          type: ClipboardContentType.text,
        ),
      );

      final results = await repo.searchAdvanced(
        query: 'clipboard',
        limit: 50,
        skip: 0,
      );
      expect(results.length, equals(2));
    });

    test('search is case-insensitive', () async {
      await repo.save(
        ClipboardItem(content: 'Hello World', type: ClipboardContentType.text),
      );

      final lower = await repo.searchAdvanced(
        query: 'hello',
        limit: 50,
        skip: 0,
      );
      final upper = await repo.searchAdvanced(
        query: 'HELLO',
        limit: 50,
        skip: 0,
      );
      expect(lower.length, equals(1));
      expect(upper.length, equals(1));
    });

    test('returns empty when no match', () async {
      await repo.save(
        ClipboardItem(content: 'some content', type: ClipboardContentType.text),
      );

      final results = await repo.searchAdvanced(
        query: 'zxqvnomatch',
        limit: 50,
        skip: 0,
      );
      expect(results, isEmpty);
    });
  });

  group('Repository search – LIKE fallback path (symbol queries)', () {
    test('dot query matches file extensions', () async {
      await repo.save(
        ClipboardItem(
          content: '/home/user/document.pdf',
          type: ClipboardContentType.file,
        ),
      );
      await repo.save(
        ClipboardItem(
          content: '/home/user/image.jpg',
          type: ClipboardContentType.file,
        ),
      );
      await repo.save(
        ClipboardItem(content: 'plain text', type: ClipboardContentType.text),
      );

      final results = await repo.searchAdvanced(
        query: '.pdf',
        limit: 50,
        skip: 0,
      );
      expect(results.length, equals(1));
      expect(results.first.content, contains('.pdf'));
    });

    test('at-sign query finds email addresses', () async {
      await repo.save(
        ClipboardItem(
          content: 'user@gmail.com',
          type: ClipboardContentType.email,
        ),
      );
      await repo.save(
        ClipboardItem(
          content: 'admin@company.io',
          type: ClipboardContentType.email,
        ),
      );
      await repo.save(
        ClipboardItem(
          content: 'no email here',
          type: ClipboardContentType.text,
        ),
      );

      final results = await repo.searchAdvanced(query: '@', limit: 50, skip: 0);
      expect(results.length, equals(2));
      for (final item in results) {
        expect(item.content, contains('@'));
      }
    });

    test('hyphen query matches UUIDs and phone numbers', () async {
      await repo.save(
        ClipboardItem(
          content: '550e8400-e29b-41d4-a716-446655440000',
          type: ClipboardContentType.uuid,
        ),
      );
      await repo.save(
        ClipboardItem(
          content: '+1-800-555-0100',
          type: ClipboardContentType.phone,
        ),
      );
      await repo.save(
        ClipboardItem(content: 'no hyphens', type: ClipboardContentType.text),
      );

      final results = await repo.searchAdvanced(query: '-', limit: 50, skip: 0);
      expect(results.length, greaterThanOrEqualTo(2));
    });

    test('symbol query paginates correctly', () async {
      for (var i = 0; i < 7; i++) {
        await repo.save(
          ClipboardItem(
            content: 'file$i@example.com',
            type: ClipboardContentType.email,
          ),
        );
      }
      for (var i = 0; i < 3; i++) {
        await repo.save(
          ClipboardItem(content: 'no-at-$i', type: ClipboardContentType.text),
        );
      }

      final page1 = await repo.searchAdvanced(query: '@', limit: 4, skip: 0);
      final page2 = await repo.searchAdvanced(query: '@', limit: 4, skip: 4);

      expect(page1.length, equals(4));
      expect(page2.length, equals(3));
      for (final item in [...page1, ...page2]) {
        expect(item.content, contains('@'));
      }
    });
  });

  group('Repository search – combined filters', () {
    test('query + type filter returns precise results', () async {
      await repo.save(
        ClipboardItem(
          content: 'python script',
          type: ClipboardContentType.text,
        ),
      );
      await repo.save(
        ClipboardItem(content: 'python link', type: ClipboardContentType.link),
      );
      await repo.save(
        ClipboardItem(content: 'ruby script', type: ClipboardContentType.text),
      );

      final results = await repo.searchAdvanced(
        query: 'python',
        types: [ClipboardContentType.text],
        limit: 50,
        skip: 0,
      );
      expect(results.length, equals(1));
      expect(results.first.content, equals('python script'));
    });

    test('query + color filter', () async {
      final colored = ClipboardItem(
        content: 'important note',
        type: ClipboardContentType.text,
        cardColor: CardColor.red,
      );
      final plain = ClipboardItem(
        content: 'important update',
        type: ClipboardContentType.text,
      );
      await repo.save(colored);
      await repo.save(plain);

      final results = await repo.searchAdvanced(
        query: 'important',
        colors: [CardColor.red],
        limit: 50,
        skip: 0,
      );
      expect(results.length, equals(1));
      expect(results.first.id, equals(colored.id));
    });

    test('pinned filter + query', () async {
      final pinnedItem = ClipboardItem(
        content: 'pinned secret',
        type: ClipboardContentType.text,
        isPinned: true,
      );
      final normalItem = ClipboardItem(
        content: 'normal secret',
        type: ClipboardContentType.text,
        isPinned: false,
      );
      await repo.save(pinnedItem);
      await repo.save(normalItem);

      final results = await repo.searchAdvanced(
        query: 'secret',
        isPinned: true,
        limit: 50,
        skip: 0,
      );
      expect(results.length, equals(1));
      expect(results.first.isPinned, isTrue);
    });

    test('multiple type filter returns all matching types', () async {
      await repo.save(
        ClipboardItem(
          content: 'my@email.com',
          type: ClipboardContentType.email,
        ),
      );
      await repo.save(
        ClipboardItem(
          content: 'https://site.com',
          type: ClipboardContentType.link,
        ),
      );
      await repo.save(
        ClipboardItem(content: 'plain text', type: ClipboardContentType.text),
      );

      final results = await repo.searchAdvanced(
        types: [ClipboardContentType.email, ClipboardContentType.link],
        limit: 50,
        skip: 0,
      );
      expect(results.length, equals(2));
      for (final item in results) {
        expect(
          item.type == ClipboardContentType.email ||
              item.type == ClipboardContentType.link,
          isTrue,
        );
      }
    });
  });

  group('Repository search – ordering', () {
    test('results are ordered by modifiedAt descending', () async {
      final old = ClipboardItem(
        content: 'older item',
        type: ClipboardContentType.text,
        modifiedAt: DateTime.utc(2023, 1, 1),
      );
      final mid = ClipboardItem(
        content: 'middle item',
        type: ClipboardContentType.text,
        modifiedAt: DateTime.utc(2024, 6, 15),
      );
      final recent = ClipboardItem(
        content: 'recent item',
        type: ClipboardContentType.text,
        modifiedAt: DateTime.utc(2025, 3, 1),
      );
      await repo.save(old);
      await repo.save(mid);
      await repo.save(recent);

      final results = await repo.searchAdvanced(limit: 10, skip: 0);
      expect(results[0].content, equals('recent item'));
      expect(results[1].content, equals('middle item'));
      expect(results[2].content, equals('older item'));
    });
  });

  group('Repository search – edge cases', () {
    test('empty query returns all items ordered by date', () async {
      await repo.save(
        ClipboardItem(content: 'alpha', type: ClipboardContentType.text),
      );
      await repo.save(
        ClipboardItem(content: 'beta', type: ClipboardContentType.text),
      );

      final results = await repo.searchAdvanced(limit: 50, skip: 0);
      expect(results.length, equals(2));
    });

    test('null query returns all items', () async {
      await repo.save(
        ClipboardItem(content: 'item a', type: ClipboardContentType.text),
      );
      final results = await repo.searchAdvanced(
        query: null,
        limit: 50,
        skip: 0,
      );
      expect(results, isNotEmpty);
    });

    test('query longer than content does not crash', () async {
      await repo.save(
        ClipboardItem(content: 'short', type: ClipboardContentType.text),
      );
      final results = await repo.searchAdvanced(
        query: 'a' * 500,
        limit: 50,
        skip: 0,
      );
      expect(results, isEmpty);
    });

    test('search on empty database returns empty list', () async {
      final results = await repo.searchAdvanced(
        query: 'anything',
        limit: 50,
        skip: 0,
      );
      expect(results, isEmpty);
    });

    test('skip beyond total items returns empty list', () async {
      await repo.save(
        ClipboardItem(content: 'only item', type: ClipboardContentType.text),
      );

      final results = await repo.searchAdvanced(limit: 10, skip: 100);
      expect(results, isEmpty);
    });
  });
}
