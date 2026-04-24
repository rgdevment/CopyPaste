import 'dart:io';

import 'package:drift/drift.dart' hide isNotNull, isNull;
import 'package:drift/native.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

import 'package:core/core.dart';

// ---------------------------------------------------------------------------
// Minimal test-only drift database used to create an old-schema SQLite file.
// It has schemaVersion=1 and creates only the original columns so that when
// SqliteRepository.fromPath opens the file it must run onUpgrade.
// ---------------------------------------------------------------------------
class _V1Database extends GeneratedDatabase {
  _V1Database(super.e);

  @override
  int get schemaVersion => 1;

  @override
  Iterable<TableInfo<Table, dynamic>> get allTables => const [];

  @override
  MigrationStrategy get migration => MigrationStrategy(
    onCreate: (m) async {
      // Create only the v1 schema — no thumbPath, sourceModifiedAt, brokenSince.
      await customStatement('''
        CREATE TABLE clipboard_items (
          id TEXT NOT NULL PRIMARY KEY,
          content TEXT NOT NULL,
          type INTEGER NOT NULL,
          created_at INTEGER NOT NULL,
          modified_at INTEGER NOT NULL,
          app_source TEXT,
          is_pinned INTEGER NOT NULL DEFAULT 0,
          label TEXT,
          card_color INTEGER NOT NULL DEFAULT 0,
          metadata TEXT,
          paste_count INTEGER NOT NULL DEFAULT 0,
          content_hash TEXT
        )
      ''');
    },
  );
}

// ---------------------------------------------------------------------------
// Same but with schemaVersion=3 (has thumbPath + sourceModifiedAt, not
// brokenSince). Tests the from < 4 migration branch in isolation.
// ---------------------------------------------------------------------------
class _V3Database extends GeneratedDatabase {
  _V3Database(super.e);

  @override
  int get schemaVersion => 3;

  @override
  Iterable<TableInfo<Table, dynamic>> get allTables => const [];

  @override
  MigrationStrategy get migration => MigrationStrategy(
    onCreate: (m) async {
      await customStatement('''
        CREATE TABLE clipboard_items (
          id TEXT NOT NULL PRIMARY KEY,
          content TEXT NOT NULL,
          type INTEGER NOT NULL,
          created_at INTEGER NOT NULL,
          modified_at INTEGER NOT NULL,
          app_source TEXT,
          is_pinned INTEGER NOT NULL DEFAULT 0,
          label TEXT,
          card_color INTEGER NOT NULL DEFAULT 0,
          metadata TEXT,
          paste_count INTEGER NOT NULL DEFAULT 0,
          content_hash TEXT,
          thumb_path TEXT,
          source_modified_at INTEGER
        )
      ''');
    },
  );
}

void main() {
  group('SqliteRepository schema migration', () {
    test('migrates v1 → v4 and repository is fully functional', () async {
      final dir = Directory.systemTemp.createTempSync('repo_migrate_v1_');
      try {
        final dbPath = p.join(dir.path, 'v1.db');

        // --- Step 1: create a v1 database file ---
        final v1 = _V1Database(NativeDatabase(File(dbPath)));
        // Force the DB to open by running a no-op query.
        await v1.customStatement('SELECT 1');
        await v1.close();

        // --- Step 2: open with the current SqliteRepository ---
        final repo = SqliteRepository.fromPath(dbPath);

        // A simple query forces the LazyDatabase to open + run migration.
        final count = await repo.count();
        expect(count, equals(0));

        // Save an item that uses the new columns (v3 thumbPath, v4 brokenSince).
        await repo.save(
          ClipboardItem(
            id: 'migrated',
            content: 'hello after migration',
            type: ClipboardContentType.text,
            thumbPath: '/tmp/thumb.png',
            brokenSince: null,
          ),
        );

        final found = await repo.getById('migrated');
        expect(found, isNotNull);
        expect(found!.content, equals('hello after migration'));
        expect(found.thumbPath, equals('/tmp/thumb.png'));

        await repo.close();
      } finally {
        dir.deleteSync(recursive: true);
      }
    });

    test('migrates v3 → v4 and brokenSince column is accessible', () async {
      final dir = Directory.systemTemp.createTempSync('repo_migrate_v3_');
      try {
        final dbPath = p.join(dir.path, 'v3.db');

        // --- Step 1: create a v3 database file ---
        final v3 = _V3Database(NativeDatabase(File(dbPath)));
        await v3.customStatement('SELECT 1');
        await v3.close();

        // --- Step 2: open with the current SqliteRepository ---
        final repo = SqliteRepository.fromPath(dbPath);
        await repo.count(); // triggers migration

        final now = DateTime.now().toUtc();
        await repo.save(
          ClipboardItem(
            id: 'v3migrated',
            content: 'from v3',
            type: ClipboardContentType.text,
            brokenSince: now,
          ),
        );

        final found = await repo.getById('v3migrated');
        expect(found?.brokenSince, isNotNull);

        await repo.close();
      } finally {
        dir.deleteSync(recursive: true);
      }
    });
  });

  group('SqliteRepository.getThumbPaths', () {
    test('returns only non-null non-empty thumbPaths', () async {
      final repo = SqliteRepository.inMemory();
      try {
        await repo.save(
          ClipboardItem(
            id: 'with-thumb',
            content: '/img/photo.png',
            type: ClipboardContentType.image,
            thumbPath: '/thumbs/photo_thumb.png',
          ),
        );
        await repo.save(
          ClipboardItem(
            id: 'no-thumb',
            content: 'hello',
            type: ClipboardContentType.text,
          ),
        );

        final paths = await repo.getThumbPaths();
        expect(paths.length, equals(1));
        expect(paths.first, equals('/thumbs/photo_thumb.png'));
      } finally {
        await repo.close();
      }
    });

    test('returns empty list when no items have thumbPath', () async {
      final repo = SqliteRepository.inMemory();
      try {
        await repo.save(
          ClipboardItem(content: 'text', type: ClipboardContentType.text),
        );
        final paths = await repo.getThumbPaths();
        expect(paths, isEmpty);
      } finally {
        await repo.close();
      }
    });
  });
}
