import 'dart:io';

import 'package:drift/drift.dart';
import 'package:drift/native.dart';

import '../models/card_color.dart';
import '../models/clipboard_content_type.dart';
import '../models/clipboard_item.dart';
import '../search/search_helper.dart';
import 'i_clipboard_repository.dart';

part 'sqlite_repository.g.dart';

@DataClassName('ClipboardRow')
class ClipboardItems extends Table {
  TextColumn get id => text()();
  TextColumn get content => text()();
  IntColumn get type => integer()();
  DateTimeColumn get createdAt => dateTime()();
  DateTimeColumn get modifiedAt => dateTime()();
  TextColumn get appSource => text().nullable()();
  BoolColumn get isPinned =>
      boolean().withDefault(const Constant(false))();
  TextColumn get label => text().nullable()();
  IntColumn get cardColor =>
      integer().withDefault(const Constant(0))();
  TextColumn get metadata => text().nullable()();
  IntColumn get pasteCount =>
      integer().withDefault(const Constant(0))();
  TextColumn get contentHash => text().nullable()();

  @override
  Set<Column> get primaryKey => {id};
}

@DriftDatabase(tables: [ClipboardItems])
class _AppDatabase extends _$_AppDatabase {
  _AppDatabase(super.e);

  @override
  int get schemaVersion => 1;

  @override
  MigrationStrategy get migration => MigrationStrategy(
        beforeOpen: (details) async {
          await customStatement('''
            CREATE VIRTUAL TABLE IF NOT EXISTS ClipboardItems_fts USING fts5(
              content,
              app_source,
              label,
              content='clipboard_items',
              content_rowid='rowid'
            )
          ''');

          await customStatement('''
            CREATE TRIGGER IF NOT EXISTS clipboard_items_ai AFTER INSERT ON clipboard_items BEGIN
              INSERT INTO ClipboardItems_fts(rowid, content, app_source, label)
              VALUES (NEW.rowid, NEW.content, NEW.app_source, NEW.label);
            END
          ''');

          await customStatement('''
            CREATE TRIGGER IF NOT EXISTS clipboard_items_ad AFTER DELETE ON clipboard_items BEGIN
              INSERT INTO ClipboardItems_fts(ClipboardItems_fts, rowid, content, app_source, label)
              VALUES ('delete', OLD.rowid, OLD.content, OLD.app_source, OLD.label);
            END
          ''');

          await customStatement('''
            CREATE TRIGGER IF NOT EXISTS clipboard_items_au AFTER UPDATE ON clipboard_items BEGIN
              INSERT INTO ClipboardItems_fts(ClipboardItems_fts, rowid, content, app_source, label)
              VALUES ('delete', OLD.rowid, OLD.content, OLD.app_source, OLD.label);
              INSERT INTO ClipboardItems_fts(rowid, content, app_source, label)
              VALUES (NEW.rowid, NEW.content, NEW.app_source, NEW.label);
            END
          ''');
        },
      );
}

class SqliteRepository implements IClipboardRepository {
  SqliteRepository._(this._db);

  factory SqliteRepository.fromPath(String dbPath) {
    try {
      final db = _AppDatabase(
        LazyDatabase(() async => NativeDatabase(File(dbPath))),
      );
      return SqliteRepository._(db);
    } catch (_) {
      _handleCorruptDatabase(dbPath);
      return SqliteRepository._(
        _AppDatabase(
          LazyDatabase(() async => NativeDatabase(File(dbPath))),
        ),
      );
    }
  }

  factory SqliteRepository.inMemory() =>
      SqliteRepository._(_AppDatabase(NativeDatabase.memory()));

  final _AppDatabase _db;

  static void _handleCorruptDatabase(String dbPath) {
    final file = File(dbPath);
    if (!file.existsSync()) return;
    try {
      final timestamp = DateTime.now().toUtc().millisecondsSinceEpoch;
      file.renameSync('$dbPath.backup.$timestamp');
      File('$dbPath-wal').deleteSync();
    } catch (_) {}
    try {
      File('$dbPath-shm').deleteSync();
    } catch (_) {}
  }

  ClipboardItem _fromRow(ClipboardRow row) => ClipboardItem(
        id: row.id,
        content: row.content,
        type: ClipboardContentType.fromValue(row.type),
        createdAt: row.createdAt,
        modifiedAt: row.modifiedAt,
        appSource: row.appSource,
        isPinned: row.isPinned,
        label: row.label,
        cardColor: CardColor.fromValue(row.cardColor),
        metadata: row.metadata,
        pasteCount: row.pasteCount,
        contentHash: row.contentHash,
      );

  ClipboardItemsCompanion _toCompanion(ClipboardItem item) =>
      ClipboardItemsCompanion(
        id: Value(item.id),
        content: Value(item.content),
        type: Value(item.type.value),
        createdAt: Value(item.createdAt),
        modifiedAt: Value(item.modifiedAt),
        appSource: Value(item.appSource),
        isPinned: Value(item.isPinned),
        label: Value(item.label),
        cardColor: Value(item.cardColor.value),
        metadata: Value(item.metadata),
        pasteCount: Value(item.pasteCount),
        contentHash: Value(item.contentHash),
      );

  ClipboardItem _fromQueryRow(QueryRow row) => ClipboardItem(
        id: row.read<String>('id'),
        content: row.read<String>('content'),
        type: ClipboardContentType.fromValue(row.read<int>('type')),
        createdAt: row.read<DateTime>('created_at'),
        modifiedAt: row.read<DateTime>('modified_at'),
        appSource: row.readNullable<String>('app_source'),
        isPinned: row.read<bool>('is_pinned'),
        label: row.readNullable<String>('label'),
        cardColor: CardColor.fromValue(row.read<int>('card_color')),
        metadata: row.readNullable<String>('metadata'),
        pasteCount: row.read<int>('paste_count'),
        contentHash: row.readNullable<String>('content_hash'),
      );

  @override
  Future<void> save(ClipboardItem item) =>
      _db.into(_db.clipboardItems).insert(_toCompanion(item));

  @override
  Future<void> update(ClipboardItem item) async {
    await (_db.update(_db.clipboardItems)
          ..where((t) => t.id.equals(item.id)))
        .write(_toCompanion(item));
  }

  @override
  Future<ClipboardItem?> getById(String id) async {
    final row = await (_db.select(_db.clipboardItems)
          ..where((t) => t.id.equals(id)))
        .getSingleOrNull();
    return row == null ? null : _fromRow(row);
  }

  @override
  Future<ClipboardItem?> getLatest() async {
    final row = await (_db.select(_db.clipboardItems)
          ..orderBy([(t) => OrderingTerm.desc(t.modifiedAt)])
          ..limit(1))
        .getSingleOrNull();
    return row == null ? null : _fromRow(row);
  }

  @override
  Future<ClipboardItem?> findByContentAndType(
    String content,
    ClipboardContentType type,
  ) async {
    final row = await (_db.select(_db.clipboardItems)
          ..where(
            (t) => t.content.equals(content) & t.type.equals(type.value),
          ))
        .getSingleOrNull();
    return row == null ? null : _fromRow(row);
  }

  @override
  Future<ClipboardItem?> findByContentHash(String contentHash) async {
    final row = await (_db.select(_db.clipboardItems)
          ..where((t) => t.contentHash.equals(contentHash)))
        .getSingleOrNull();
    return row == null ? null : _fromRow(row);
  }

  @override
  Future<List<ClipboardItem>> getAll() async {
    final rows = await (_db.select(_db.clipboardItems)
          ..orderBy([(t) => OrderingTerm.desc(t.modifiedAt)]))
        .get();
    return rows.map(_fromRow).toList();
  }

  @override
  Future<void> delete(String id) =>
      (_db.delete(_db.clipboardItems)..where((t) => t.id.equals(id))).go();

  @override
  Future<int> clearOldItems(int days, {bool excludePinned = true}) async {
    final cutoff = DateTime.now().toUtc().subtract(Duration(days: days));
    final deleted = await (_db.delete(_db.clipboardItems)
          ..where((t) {
            final isOld = t.modifiedAt.isSmallerThanValue(cutoff);
            return excludePinned ? isOld & t.isPinned.equals(false) : isOld;
          }))
        .go();

    if (deleted > 50) {
      try {
        await _db.customStatement('VACUUM');
      } catch (_) {}
    }

    return deleted;
  }

  @override
  Future<List<ClipboardItem>> search(
    String query, {
    int limit = 50,
    int skip = 0,
  }) async {
    final normalized = SearchHelper.normalize(query);
    if (normalized.isEmpty) {
      final rows = await (_db.select(_db.clipboardItems)
            ..orderBy([
              (t) => OrderingTerm.desc(t.isPinned),
              (t) => OrderingTerm.desc(t.modifiedAt),
            ])
            ..limit(limit, offset: skip))
          .get();
      return rows.map(_fromRow).toList();
    }

    final ftsQuery =
        '${normalized.replaceAll('"', '""')}*';
    final likePattern = '%$normalized%';

    final results = await _db.customSelect(
      '''
      WITH fts_results AS (
        SELECT c.*, bm25(ClipboardItems_fts) AS rank, 1 AS source
        FROM clipboard_items c
        INNER JOIN ClipboardItems_fts fts ON c.rowid = fts.rowid
        WHERE ClipboardItems_fts MATCH ?
      ),
      like_results AS (
        SELECT c.*, 0.0 AS rank, 2 AS source
        FROM clipboard_items c
        WHERE (LOWER(c.content) LIKE ? OR LOWER(c.label) LIKE ? OR LOWER(c.app_source) LIKE ?)
          AND c.id NOT IN (SELECT id FROM fts_results)
      )
      SELECT * FROM (
        SELECT * FROM fts_results
        UNION ALL
        SELECT * FROM like_results
      )
      ORDER BY is_pinned DESC, source ASC, rank ASC, modified_at DESC
      LIMIT ? OFFSET ?
      ''',
      variables: [
        Variable.withString(ftsQuery),
        Variable.withString(likePattern),
        Variable.withString(likePattern),
        Variable.withString(likePattern),
        Variable.withInt(limit),
        Variable.withInt(skip),
      ],
      readsFrom: {_db.clipboardItems},
    ).get();

    return results.map((row) => _fromQueryRow(row)).toList();
  }

  @override
  Future<List<ClipboardItem>> searchAdvanced({
    String? query,
    List<ClipboardContentType>? types,
    List<CardColor>? colors,
    bool? isPinned,
    required int limit,
    required int skip,
  }) async {
    final normalized =
        (query != null && query.isNotEmpty) ? SearchHelper.normalize(query) : null;

    final hasTextQuery = normalized != null && normalized.isNotEmpty;
    final hasTypeFilter = types != null && types.isNotEmpty;
    final hasColorFilter = colors != null && colors.isNotEmpty;
    final hasAnyFilter = hasTextQuery || hasTypeFilter || hasColorFilter;

    final conditions = <String>[];
    final variables = <Variable>[];

    if (!hasAnyFilter && isPinned != null) {
      conditions.add('c.is_pinned = ?');
      variables.add(Variable.withBool(isPinned));
    }

    final effectiveTypes = hasTypeFilter ? types : null;
    if (effectiveTypes != null) {
      final placeholders = List.filled(effectiveTypes.length, '?').join(', ');
      conditions.add('c.type IN ($placeholders)');
      for (final t in effectiveTypes) {
        variables.add(Variable.withInt(t.value));
      }
    }

    final effectiveColors = hasColorFilter ? colors : null;
    if (effectiveColors != null) {
      final placeholders = List.filled(effectiveColors.length, '?').join(', ');
      conditions.add('c.card_color IN ($placeholders)');
      for (final c in effectiveColors) {
        variables.add(Variable.withInt(c.value));
      }
    }

    final filterClause =
        conditions.isEmpty ? '1=1' : conditions.join(' AND ');

    if (hasTextQuery) {
      final ftsQuery =
          '${normalized.replaceAll('"', '""')}*';
      final likePattern = '%$normalized%';

      final results = await _db.customSelect(
        '''
        WITH fts_results AS (
          SELECT c.*, bm25(ClipboardItems_fts) AS rank, 1 AS source
          FROM clipboard_items c
          INNER JOIN ClipboardItems_fts fts ON c.rowid = fts.rowid
          WHERE ClipboardItems_fts MATCH ? AND $filterClause
        ),
        like_results AS (
          SELECT c.*, 0.0 AS rank, 2 AS source
          FROM clipboard_items c
          WHERE $filterClause
            AND (LOWER(c.content) LIKE ? OR LOWER(c.label) LIKE ? OR LOWER(c.app_source) LIKE ?)
            AND c.id NOT IN (SELECT id FROM fts_results)
        )
        SELECT * FROM (
          SELECT * FROM fts_results
          UNION ALL
          SELECT * FROM like_results
        )
        ORDER BY is_pinned DESC, source ASC, rank ASC, modified_at DESC
        LIMIT ? OFFSET ?
        ''',
        variables: [
          Variable.withString(ftsQuery),
          ...variables,
          ...variables,
          Variable.withString(likePattern),
          Variable.withString(likePattern),
          Variable.withString(likePattern),
          Variable.withInt(limit),
          Variable.withInt(skip),
        ],
        readsFrom: {_db.clipboardItems},
      ).get();

      return results.map((row) => _fromQueryRow(row)).toList();
    }

    final results = await _db.customSelect(
      '''
      SELECT c.* FROM clipboard_items c
      WHERE $filterClause
      ORDER BY c.is_pinned DESC, c.modified_at DESC
      LIMIT ? OFFSET ?
      ''',
      variables: [
        ...variables,
        Variable.withInt(limit),
        Variable.withInt(skip),
      ],
      readsFrom: {_db.clipboardItems},
    ).get();

    return results.map((row) => _fromQueryRow(row)).toList();
  }

  @override
  Future<void> close() => _db.close();
}
