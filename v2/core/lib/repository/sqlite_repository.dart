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
}

class SqliteRepository implements IClipboardRepository {
  SqliteRepository._(this._db);

  factory SqliteRepository.fromPath(String dbPath) => SqliteRepository._(
        _AppDatabase(
          LazyDatabase(() async => NativeDatabase(File(dbPath))),
        ),
      );

  factory SqliteRepository.inMemory() =>
      SqliteRepository._(_AppDatabase(NativeDatabase.memory()));

  final _AppDatabase _db;

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
  Future<int> clearOldItems(int days, {bool excludePinned = true}) {
    final cutoff = DateTime.now().toUtc().subtract(Duration(days: days));
    return (_db.delete(_db.clipboardItems)
          ..where((t) {
            final isOld = t.createdAt.isSmallerThanValue(cutoff);
            return excludePinned ? isOld & t.isPinned.equals(false) : isOld;
          }))
        .go();
  }

  @override
  Future<List<ClipboardItem>> search(
    String query, {
    int limit = 50,
    int skip = 0,
  }) async {
    final normalized = SearchHelper.normalize(query);
    final rows = await (_db.select(_db.clipboardItems)
          ..where((t) => t.content.lower().contains(normalized))
          ..orderBy([(t) => OrderingTerm.desc(t.modifiedAt)])
          ..limit(limit, offset: skip))
        .get();
    return rows.map(_fromRow).toList();
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
    final typeVals = types?.map((e) => e.value).toList();
    final colorVals = colors?.map((e) => e.value).toList();

    final stmt = _db.select(_db.clipboardItems)
      ..where((t) {
        Expression<bool> expr = const Constant(true);
        if (normalized != null) {
          expr = expr & t.content.lower().contains(normalized);
        }
        if (typeVals != null) expr = expr & t.type.isIn(typeVals);
        if (colorVals != null) expr = expr & t.cardColor.isIn(colorVals);
        if (isPinned != null) expr = expr & t.isPinned.equals(isPinned);
        return expr;
      })
      ..orderBy([(t) => OrderingTerm.desc(t.modifiedAt)])
      ..limit(limit, offset: skip);

    final rows = await stmt.get();
    return rows.map(_fromRow).toList();
  }

  Future<void> close() => _db.close();
}
