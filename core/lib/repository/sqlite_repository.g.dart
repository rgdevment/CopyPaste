// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'sqlite_repository.dart';

// ignore_for_file: type=lint
class $ClipboardItemsTable extends ClipboardItems
    with TableInfo<$ClipboardItemsTable, ClipboardRow> {
  @override
  final GeneratedDatabase attachedDatabase;
  final String? _alias;
  $ClipboardItemsTable(this.attachedDatabase, [this._alias]);
  static const VerificationMeta _idMeta = const VerificationMeta('id');
  @override
  late final GeneratedColumn<String> id = GeneratedColumn<String>(
    'id',
    aliasedName,
    false,
    type: DriftSqlType.string,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _contentMeta = const VerificationMeta(
    'content',
  );
  @override
  late final GeneratedColumn<String> content = GeneratedColumn<String>(
    'content',
    aliasedName,
    false,
    type: DriftSqlType.string,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _typeMeta = const VerificationMeta('type');
  @override
  late final GeneratedColumn<int> type = GeneratedColumn<int>(
    'type',
    aliasedName,
    false,
    type: DriftSqlType.int,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _createdAtMeta = const VerificationMeta(
    'createdAt',
  );
  @override
  late final GeneratedColumn<DateTime> createdAt = GeneratedColumn<DateTime>(
    'created_at',
    aliasedName,
    false,
    type: DriftSqlType.dateTime,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _modifiedAtMeta = const VerificationMeta(
    'modifiedAt',
  );
  @override
  late final GeneratedColumn<DateTime> modifiedAt = GeneratedColumn<DateTime>(
    'modified_at',
    aliasedName,
    false,
    type: DriftSqlType.dateTime,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _appSourceMeta = const VerificationMeta(
    'appSource',
  );
  @override
  late final GeneratedColumn<String> appSource = GeneratedColumn<String>(
    'app_source',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _isPinnedMeta = const VerificationMeta(
    'isPinned',
  );
  @override
  late final GeneratedColumn<bool> isPinned = GeneratedColumn<bool>(
    'is_pinned',
    aliasedName,
    false,
    type: DriftSqlType.bool,
    requiredDuringInsert: false,
    defaultConstraints: GeneratedColumn.constraintIsAlways(
      'CHECK ("is_pinned" IN (0, 1))',
    ),
    defaultValue: const Constant(false),
  );
  static const VerificationMeta _labelMeta = const VerificationMeta('label');
  @override
  late final GeneratedColumn<String> label = GeneratedColumn<String>(
    'label',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _cardColorMeta = const VerificationMeta(
    'cardColor',
  );
  @override
  late final GeneratedColumn<int> cardColor = GeneratedColumn<int>(
    'card_color',
    aliasedName,
    false,
    type: DriftSqlType.int,
    requiredDuringInsert: false,
    defaultValue: const Constant(0),
  );
  static const VerificationMeta _metadataMeta = const VerificationMeta(
    'metadata',
  );
  @override
  late final GeneratedColumn<String> metadata = GeneratedColumn<String>(
    'metadata',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _pasteCountMeta = const VerificationMeta(
    'pasteCount',
  );
  @override
  late final GeneratedColumn<int> pasteCount = GeneratedColumn<int>(
    'paste_count',
    aliasedName,
    false,
    type: DriftSqlType.int,
    requiredDuringInsert: false,
    defaultValue: const Constant(0),
  );
  static const VerificationMeta _contentHashMeta = const VerificationMeta(
    'contentHash',
  );
  @override
  late final GeneratedColumn<String> contentHash = GeneratedColumn<String>(
    'content_hash',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _thumbPathMeta = const VerificationMeta(
    'thumbPath',
  );
  @override
  late final GeneratedColumn<String> thumbPath = GeneratedColumn<String>(
    'thumb_path',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _sourceModifiedAtMeta = const VerificationMeta(
    'sourceModifiedAt',
  );
  @override
  late final GeneratedColumn<DateTime> sourceModifiedAt =
      GeneratedColumn<DateTime>(
        'source_modified_at',
        aliasedName,
        true,
        type: DriftSqlType.dateTime,
        requiredDuringInsert: false,
      );
  static const VerificationMeta _brokenSinceMeta = const VerificationMeta(
    'brokenSince',
  );
  @override
  late final GeneratedColumn<DateTime> brokenSince = GeneratedColumn<DateTime>(
    'broken_since',
    aliasedName,
    true,
    type: DriftSqlType.dateTime,
    requiredDuringInsert: false,
  );
  @override
  List<GeneratedColumn> get $columns => [
    id,
    content,
    type,
    createdAt,
    modifiedAt,
    appSource,
    isPinned,
    label,
    cardColor,
    metadata,
    pasteCount,
    contentHash,
    thumbPath,
    sourceModifiedAt,
    brokenSince,
  ];
  @override
  String get aliasedName => _alias ?? actualTableName;
  @override
  String get actualTableName => $name;
  static const String $name = 'clipboard_items';
  @override
  VerificationContext validateIntegrity(
    Insertable<ClipboardRow> instance, {
    bool isInserting = false,
  }) {
    final context = VerificationContext();
    final data = instance.toColumns(true);
    if (data.containsKey('id')) {
      context.handle(_idMeta, id.isAcceptableOrUnknown(data['id']!, _idMeta));
    } else if (isInserting) {
      context.missing(_idMeta);
    }
    if (data.containsKey('content')) {
      context.handle(
        _contentMeta,
        content.isAcceptableOrUnknown(data['content']!, _contentMeta),
      );
    } else if (isInserting) {
      context.missing(_contentMeta);
    }
    if (data.containsKey('type')) {
      context.handle(
        _typeMeta,
        type.isAcceptableOrUnknown(data['type']!, _typeMeta),
      );
    } else if (isInserting) {
      context.missing(_typeMeta);
    }
    if (data.containsKey('created_at')) {
      context.handle(
        _createdAtMeta,
        createdAt.isAcceptableOrUnknown(data['created_at']!, _createdAtMeta),
      );
    } else if (isInserting) {
      context.missing(_createdAtMeta);
    }
    if (data.containsKey('modified_at')) {
      context.handle(
        _modifiedAtMeta,
        modifiedAt.isAcceptableOrUnknown(data['modified_at']!, _modifiedAtMeta),
      );
    } else if (isInserting) {
      context.missing(_modifiedAtMeta);
    }
    if (data.containsKey('app_source')) {
      context.handle(
        _appSourceMeta,
        appSource.isAcceptableOrUnknown(data['app_source']!, _appSourceMeta),
      );
    }
    if (data.containsKey('is_pinned')) {
      context.handle(
        _isPinnedMeta,
        isPinned.isAcceptableOrUnknown(data['is_pinned']!, _isPinnedMeta),
      );
    }
    if (data.containsKey('label')) {
      context.handle(
        _labelMeta,
        label.isAcceptableOrUnknown(data['label']!, _labelMeta),
      );
    }
    if (data.containsKey('card_color')) {
      context.handle(
        _cardColorMeta,
        cardColor.isAcceptableOrUnknown(data['card_color']!, _cardColorMeta),
      );
    }
    if (data.containsKey('metadata')) {
      context.handle(
        _metadataMeta,
        metadata.isAcceptableOrUnknown(data['metadata']!, _metadataMeta),
      );
    }
    if (data.containsKey('paste_count')) {
      context.handle(
        _pasteCountMeta,
        pasteCount.isAcceptableOrUnknown(data['paste_count']!, _pasteCountMeta),
      );
    }
    if (data.containsKey('content_hash')) {
      context.handle(
        _contentHashMeta,
        contentHash.isAcceptableOrUnknown(
          data['content_hash']!,
          _contentHashMeta,
        ),
      );
    }
    if (data.containsKey('thumb_path')) {
      context.handle(
        _thumbPathMeta,
        thumbPath.isAcceptableOrUnknown(data['thumb_path']!, _thumbPathMeta),
      );
    }
    if (data.containsKey('source_modified_at')) {
      context.handle(
        _sourceModifiedAtMeta,
        sourceModifiedAt.isAcceptableOrUnknown(
          data['source_modified_at']!,
          _sourceModifiedAtMeta,
        ),
      );
    }
    if (data.containsKey('broken_since')) {
      context.handle(
        _brokenSinceMeta,
        brokenSince.isAcceptableOrUnknown(
          data['broken_since']!,
          _brokenSinceMeta,
        ),
      );
    }
    return context;
  }

  @override
  Set<GeneratedColumn> get $primaryKey => {id};
  @override
  ClipboardRow map(Map<String, dynamic> data, {String? tablePrefix}) {
    final effectivePrefix = tablePrefix != null ? '$tablePrefix.' : '';
    return ClipboardRow(
      id: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}id'],
      )!,
      content: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}content'],
      )!,
      type: attachedDatabase.typeMapping.read(
        DriftSqlType.int,
        data['${effectivePrefix}type'],
      )!,
      createdAt: attachedDatabase.typeMapping.read(
        DriftSqlType.dateTime,
        data['${effectivePrefix}created_at'],
      )!,
      modifiedAt: attachedDatabase.typeMapping.read(
        DriftSqlType.dateTime,
        data['${effectivePrefix}modified_at'],
      )!,
      appSource: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}app_source'],
      ),
      isPinned: attachedDatabase.typeMapping.read(
        DriftSqlType.bool,
        data['${effectivePrefix}is_pinned'],
      )!,
      label: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}label'],
      ),
      cardColor: attachedDatabase.typeMapping.read(
        DriftSqlType.int,
        data['${effectivePrefix}card_color'],
      )!,
      metadata: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}metadata'],
      ),
      pasteCount: attachedDatabase.typeMapping.read(
        DriftSqlType.int,
        data['${effectivePrefix}paste_count'],
      )!,
      contentHash: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}content_hash'],
      ),
      thumbPath: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}thumb_path'],
      ),
      sourceModifiedAt: attachedDatabase.typeMapping.read(
        DriftSqlType.dateTime,
        data['${effectivePrefix}source_modified_at'],
      ),
      brokenSince: attachedDatabase.typeMapping.read(
        DriftSqlType.dateTime,
        data['${effectivePrefix}broken_since'],
      ),
    );
  }

  @override
  $ClipboardItemsTable createAlias(String alias) {
    return $ClipboardItemsTable(attachedDatabase, alias);
  }
}

class ClipboardRow extends DataClass implements Insertable<ClipboardRow> {
  final String id;
  final String content;
  final int type;
  final DateTime createdAt;
  final DateTime modifiedAt;
  final String? appSource;
  final bool isPinned;
  final String? label;
  final int cardColor;
  final String? metadata;
  final int pasteCount;
  final String? contentHash;
  final String? thumbPath;
  final DateTime? sourceModifiedAt;
  final DateTime? brokenSince;
  const ClipboardRow({
    required this.id,
    required this.content,
    required this.type,
    required this.createdAt,
    required this.modifiedAt,
    this.appSource,
    required this.isPinned,
    this.label,
    required this.cardColor,
    this.metadata,
    required this.pasteCount,
    this.contentHash,
    this.thumbPath,
    this.sourceModifiedAt,
    this.brokenSince,
  });
  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    map['id'] = Variable<String>(id);
    map['content'] = Variable<String>(content);
    map['type'] = Variable<int>(type);
    map['created_at'] = Variable<DateTime>(createdAt);
    map['modified_at'] = Variable<DateTime>(modifiedAt);
    if (!nullToAbsent || appSource != null) {
      map['app_source'] = Variable<String>(appSource);
    }
    map['is_pinned'] = Variable<bool>(isPinned);
    if (!nullToAbsent || label != null) {
      map['label'] = Variable<String>(label);
    }
    map['card_color'] = Variable<int>(cardColor);
    if (!nullToAbsent || metadata != null) {
      map['metadata'] = Variable<String>(metadata);
    }
    map['paste_count'] = Variable<int>(pasteCount);
    if (!nullToAbsent || contentHash != null) {
      map['content_hash'] = Variable<String>(contentHash);
    }
    if (!nullToAbsent || thumbPath != null) {
      map['thumb_path'] = Variable<String>(thumbPath);
    }
    if (!nullToAbsent || sourceModifiedAt != null) {
      map['source_modified_at'] = Variable<DateTime>(sourceModifiedAt);
    }
    if (!nullToAbsent || brokenSince != null) {
      map['broken_since'] = Variable<DateTime>(brokenSince);
    }
    return map;
  }

  ClipboardItemsCompanion toCompanion(bool nullToAbsent) {
    return ClipboardItemsCompanion(
      id: Value(id),
      content: Value(content),
      type: Value(type),
      createdAt: Value(createdAt),
      modifiedAt: Value(modifiedAt),
      appSource: appSource == null && nullToAbsent
          ? const Value.absent()
          : Value(appSource),
      isPinned: Value(isPinned),
      label: label == null && nullToAbsent
          ? const Value.absent()
          : Value(label),
      cardColor: Value(cardColor),
      metadata: metadata == null && nullToAbsent
          ? const Value.absent()
          : Value(metadata),
      pasteCount: Value(pasteCount),
      contentHash: contentHash == null && nullToAbsent
          ? const Value.absent()
          : Value(contentHash),
      thumbPath: thumbPath == null && nullToAbsent
          ? const Value.absent()
          : Value(thumbPath),
      sourceModifiedAt: sourceModifiedAt == null && nullToAbsent
          ? const Value.absent()
          : Value(sourceModifiedAt),
      brokenSince: brokenSince == null && nullToAbsent
          ? const Value.absent()
          : Value(brokenSince),
    );
  }

  factory ClipboardRow.fromJson(
    Map<String, dynamic> json, {
    ValueSerializer? serializer,
  }) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return ClipboardRow(
      id: serializer.fromJson<String>(json['id']),
      content: serializer.fromJson<String>(json['content']),
      type: serializer.fromJson<int>(json['type']),
      createdAt: serializer.fromJson<DateTime>(json['createdAt']),
      modifiedAt: serializer.fromJson<DateTime>(json['modifiedAt']),
      appSource: serializer.fromJson<String?>(json['appSource']),
      isPinned: serializer.fromJson<bool>(json['isPinned']),
      label: serializer.fromJson<String?>(json['label']),
      cardColor: serializer.fromJson<int>(json['cardColor']),
      metadata: serializer.fromJson<String?>(json['metadata']),
      pasteCount: serializer.fromJson<int>(json['pasteCount']),
      contentHash: serializer.fromJson<String?>(json['contentHash']),
      thumbPath: serializer.fromJson<String?>(json['thumbPath']),
      sourceModifiedAt: serializer.fromJson<DateTime?>(
        json['sourceModifiedAt'],
      ),
      brokenSince: serializer.fromJson<DateTime?>(json['brokenSince']),
    );
  }
  @override
  Map<String, dynamic> toJson({ValueSerializer? serializer}) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return <String, dynamic>{
      'id': serializer.toJson<String>(id),
      'content': serializer.toJson<String>(content),
      'type': serializer.toJson<int>(type),
      'createdAt': serializer.toJson<DateTime>(createdAt),
      'modifiedAt': serializer.toJson<DateTime>(modifiedAt),
      'appSource': serializer.toJson<String?>(appSource),
      'isPinned': serializer.toJson<bool>(isPinned),
      'label': serializer.toJson<String?>(label),
      'cardColor': serializer.toJson<int>(cardColor),
      'metadata': serializer.toJson<String?>(metadata),
      'pasteCount': serializer.toJson<int>(pasteCount),
      'contentHash': serializer.toJson<String?>(contentHash),
      'thumbPath': serializer.toJson<String?>(thumbPath),
      'sourceModifiedAt': serializer.toJson<DateTime?>(sourceModifiedAt),
      'brokenSince': serializer.toJson<DateTime?>(brokenSince),
    };
  }

  ClipboardRow copyWith({
    String? id,
    String? content,
    int? type,
    DateTime? createdAt,
    DateTime? modifiedAt,
    Value<String?> appSource = const Value.absent(),
    bool? isPinned,
    Value<String?> label = const Value.absent(),
    int? cardColor,
    Value<String?> metadata = const Value.absent(),
    int? pasteCount,
    Value<String?> contentHash = const Value.absent(),
    Value<String?> thumbPath = const Value.absent(),
    Value<DateTime?> sourceModifiedAt = const Value.absent(),
    Value<DateTime?> brokenSince = const Value.absent(),
  }) => ClipboardRow(
    id: id ?? this.id,
    content: content ?? this.content,
    type: type ?? this.type,
    createdAt: createdAt ?? this.createdAt,
    modifiedAt: modifiedAt ?? this.modifiedAt,
    appSource: appSource.present ? appSource.value : this.appSource,
    isPinned: isPinned ?? this.isPinned,
    label: label.present ? label.value : this.label,
    cardColor: cardColor ?? this.cardColor,
    metadata: metadata.present ? metadata.value : this.metadata,
    pasteCount: pasteCount ?? this.pasteCount,
    contentHash: contentHash.present ? contentHash.value : this.contentHash,
    thumbPath: thumbPath.present ? thumbPath.value : this.thumbPath,
    sourceModifiedAt: sourceModifiedAt.present
        ? sourceModifiedAt.value
        : this.sourceModifiedAt,
    brokenSince: brokenSince.present ? brokenSince.value : this.brokenSince,
  );
  ClipboardRow copyWithCompanion(ClipboardItemsCompanion data) {
    return ClipboardRow(
      id: data.id.present ? data.id.value : this.id,
      content: data.content.present ? data.content.value : this.content,
      type: data.type.present ? data.type.value : this.type,
      createdAt: data.createdAt.present ? data.createdAt.value : this.createdAt,
      modifiedAt: data.modifiedAt.present
          ? data.modifiedAt.value
          : this.modifiedAt,
      appSource: data.appSource.present ? data.appSource.value : this.appSource,
      isPinned: data.isPinned.present ? data.isPinned.value : this.isPinned,
      label: data.label.present ? data.label.value : this.label,
      cardColor: data.cardColor.present ? data.cardColor.value : this.cardColor,
      metadata: data.metadata.present ? data.metadata.value : this.metadata,
      pasteCount: data.pasteCount.present
          ? data.pasteCount.value
          : this.pasteCount,
      contentHash: data.contentHash.present
          ? data.contentHash.value
          : this.contentHash,
      thumbPath: data.thumbPath.present ? data.thumbPath.value : this.thumbPath,
      sourceModifiedAt: data.sourceModifiedAt.present
          ? data.sourceModifiedAt.value
          : this.sourceModifiedAt,
      brokenSince: data.brokenSince.present
          ? data.brokenSince.value
          : this.brokenSince,
    );
  }

  @override
  String toString() {
    return (StringBuffer('ClipboardRow(')
          ..write('id: $id, ')
          ..write('content: $content, ')
          ..write('type: $type, ')
          ..write('createdAt: $createdAt, ')
          ..write('modifiedAt: $modifiedAt, ')
          ..write('appSource: $appSource, ')
          ..write('isPinned: $isPinned, ')
          ..write('label: $label, ')
          ..write('cardColor: $cardColor, ')
          ..write('metadata: $metadata, ')
          ..write('pasteCount: $pasteCount, ')
          ..write('contentHash: $contentHash, ')
          ..write('thumbPath: $thumbPath, ')
          ..write('sourceModifiedAt: $sourceModifiedAt, ')
          ..write('brokenSince: $brokenSince')
          ..write(')'))
        .toString();
  }

  @override
  int get hashCode => Object.hash(
    id,
    content,
    type,
    createdAt,
    modifiedAt,
    appSource,
    isPinned,
    label,
    cardColor,
    metadata,
    pasteCount,
    contentHash,
    thumbPath,
    sourceModifiedAt,
    brokenSince,
  );
  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is ClipboardRow &&
          other.id == this.id &&
          other.content == this.content &&
          other.type == this.type &&
          other.createdAt == this.createdAt &&
          other.modifiedAt == this.modifiedAt &&
          other.appSource == this.appSource &&
          other.isPinned == this.isPinned &&
          other.label == this.label &&
          other.cardColor == this.cardColor &&
          other.metadata == this.metadata &&
          other.pasteCount == this.pasteCount &&
          other.contentHash == this.contentHash &&
          other.thumbPath == this.thumbPath &&
          other.sourceModifiedAt == this.sourceModifiedAt &&
          other.brokenSince == this.brokenSince);
}

class ClipboardItemsCompanion extends UpdateCompanion<ClipboardRow> {
  final Value<String> id;
  final Value<String> content;
  final Value<int> type;
  final Value<DateTime> createdAt;
  final Value<DateTime> modifiedAt;
  final Value<String?> appSource;
  final Value<bool> isPinned;
  final Value<String?> label;
  final Value<int> cardColor;
  final Value<String?> metadata;
  final Value<int> pasteCount;
  final Value<String?> contentHash;
  final Value<String?> thumbPath;
  final Value<DateTime?> sourceModifiedAt;
  final Value<DateTime?> brokenSince;
  final Value<int> rowid;
  const ClipboardItemsCompanion({
    this.id = const Value.absent(),
    this.content = const Value.absent(),
    this.type = const Value.absent(),
    this.createdAt = const Value.absent(),
    this.modifiedAt = const Value.absent(),
    this.appSource = const Value.absent(),
    this.isPinned = const Value.absent(),
    this.label = const Value.absent(),
    this.cardColor = const Value.absent(),
    this.metadata = const Value.absent(),
    this.pasteCount = const Value.absent(),
    this.contentHash = const Value.absent(),
    this.thumbPath = const Value.absent(),
    this.sourceModifiedAt = const Value.absent(),
    this.brokenSince = const Value.absent(),
    this.rowid = const Value.absent(),
  });
  ClipboardItemsCompanion.insert({
    required String id,
    required String content,
    required int type,
    required DateTime createdAt,
    required DateTime modifiedAt,
    this.appSource = const Value.absent(),
    this.isPinned = const Value.absent(),
    this.label = const Value.absent(),
    this.cardColor = const Value.absent(),
    this.metadata = const Value.absent(),
    this.pasteCount = const Value.absent(),
    this.contentHash = const Value.absent(),
    this.thumbPath = const Value.absent(),
    this.sourceModifiedAt = const Value.absent(),
    this.brokenSince = const Value.absent(),
    this.rowid = const Value.absent(),
  }) : id = Value(id),
       content = Value(content),
       type = Value(type),
       createdAt = Value(createdAt),
       modifiedAt = Value(modifiedAt);
  static Insertable<ClipboardRow> custom({
    Expression<String>? id,
    Expression<String>? content,
    Expression<int>? type,
    Expression<DateTime>? createdAt,
    Expression<DateTime>? modifiedAt,
    Expression<String>? appSource,
    Expression<bool>? isPinned,
    Expression<String>? label,
    Expression<int>? cardColor,
    Expression<String>? metadata,
    Expression<int>? pasteCount,
    Expression<String>? contentHash,
    Expression<String>? thumbPath,
    Expression<DateTime>? sourceModifiedAt,
    Expression<DateTime>? brokenSince,
    Expression<int>? rowid,
  }) {
    return RawValuesInsertable({
      if (id != null) 'id': id,
      if (content != null) 'content': content,
      if (type != null) 'type': type,
      if (createdAt != null) 'created_at': createdAt,
      if (modifiedAt != null) 'modified_at': modifiedAt,
      if (appSource != null) 'app_source': appSource,
      if (isPinned != null) 'is_pinned': isPinned,
      if (label != null) 'label': label,
      if (cardColor != null) 'card_color': cardColor,
      if (metadata != null) 'metadata': metadata,
      if (pasteCount != null) 'paste_count': pasteCount,
      if (contentHash != null) 'content_hash': contentHash,
      if (thumbPath != null) 'thumb_path': thumbPath,
      if (sourceModifiedAt != null) 'source_modified_at': sourceModifiedAt,
      if (brokenSince != null) 'broken_since': brokenSince,
      if (rowid != null) 'rowid': rowid,
    });
  }

  ClipboardItemsCompanion copyWith({
    Value<String>? id,
    Value<String>? content,
    Value<int>? type,
    Value<DateTime>? createdAt,
    Value<DateTime>? modifiedAt,
    Value<String?>? appSource,
    Value<bool>? isPinned,
    Value<String?>? label,
    Value<int>? cardColor,
    Value<String?>? metadata,
    Value<int>? pasteCount,
    Value<String?>? contentHash,
    Value<String?>? thumbPath,
    Value<DateTime?>? sourceModifiedAt,
    Value<DateTime?>? brokenSince,
    Value<int>? rowid,
  }) {
    return ClipboardItemsCompanion(
      id: id ?? this.id,
      content: content ?? this.content,
      type: type ?? this.type,
      createdAt: createdAt ?? this.createdAt,
      modifiedAt: modifiedAt ?? this.modifiedAt,
      appSource: appSource ?? this.appSource,
      isPinned: isPinned ?? this.isPinned,
      label: label ?? this.label,
      cardColor: cardColor ?? this.cardColor,
      metadata: metadata ?? this.metadata,
      pasteCount: pasteCount ?? this.pasteCount,
      contentHash: contentHash ?? this.contentHash,
      thumbPath: thumbPath ?? this.thumbPath,
      sourceModifiedAt: sourceModifiedAt ?? this.sourceModifiedAt,
      brokenSince: brokenSince ?? this.brokenSince,
      rowid: rowid ?? this.rowid,
    );
  }

  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    if (id.present) {
      map['id'] = Variable<String>(id.value);
    }
    if (content.present) {
      map['content'] = Variable<String>(content.value);
    }
    if (type.present) {
      map['type'] = Variable<int>(type.value);
    }
    if (createdAt.present) {
      map['created_at'] = Variable<DateTime>(createdAt.value);
    }
    if (modifiedAt.present) {
      map['modified_at'] = Variable<DateTime>(modifiedAt.value);
    }
    if (appSource.present) {
      map['app_source'] = Variable<String>(appSource.value);
    }
    if (isPinned.present) {
      map['is_pinned'] = Variable<bool>(isPinned.value);
    }
    if (label.present) {
      map['label'] = Variable<String>(label.value);
    }
    if (cardColor.present) {
      map['card_color'] = Variable<int>(cardColor.value);
    }
    if (metadata.present) {
      map['metadata'] = Variable<String>(metadata.value);
    }
    if (pasteCount.present) {
      map['paste_count'] = Variable<int>(pasteCount.value);
    }
    if (contentHash.present) {
      map['content_hash'] = Variable<String>(contentHash.value);
    }
    if (thumbPath.present) {
      map['thumb_path'] = Variable<String>(thumbPath.value);
    }
    if (sourceModifiedAt.present) {
      map['source_modified_at'] = Variable<DateTime>(sourceModifiedAt.value);
    }
    if (brokenSince.present) {
      map['broken_since'] = Variable<DateTime>(brokenSince.value);
    }
    if (rowid.present) {
      map['rowid'] = Variable<int>(rowid.value);
    }
    return map;
  }

  @override
  String toString() {
    return (StringBuffer('ClipboardItemsCompanion(')
          ..write('id: $id, ')
          ..write('content: $content, ')
          ..write('type: $type, ')
          ..write('createdAt: $createdAt, ')
          ..write('modifiedAt: $modifiedAt, ')
          ..write('appSource: $appSource, ')
          ..write('isPinned: $isPinned, ')
          ..write('label: $label, ')
          ..write('cardColor: $cardColor, ')
          ..write('metadata: $metadata, ')
          ..write('pasteCount: $pasteCount, ')
          ..write('contentHash: $contentHash, ')
          ..write('thumbPath: $thumbPath, ')
          ..write('sourceModifiedAt: $sourceModifiedAt, ')
          ..write('brokenSince: $brokenSince, ')
          ..write('rowid: $rowid')
          ..write(')'))
        .toString();
  }
}

abstract class _$_AppDatabase extends GeneratedDatabase {
  _$_AppDatabase(QueryExecutor e) : super(e);
  $_AppDatabaseManager get managers => $_AppDatabaseManager(this);
  late final $ClipboardItemsTable clipboardItems = $ClipboardItemsTable(this);
  @override
  Iterable<TableInfo<Table, Object?>> get allTables =>
      allSchemaEntities.whereType<TableInfo<Table, Object?>>();
  @override
  List<DatabaseSchemaEntity> get allSchemaEntities => [clipboardItems];
}

typedef $$ClipboardItemsTableCreateCompanionBuilder =
    ClipboardItemsCompanion Function({
      required String id,
      required String content,
      required int type,
      required DateTime createdAt,
      required DateTime modifiedAt,
      Value<String?> appSource,
      Value<bool> isPinned,
      Value<String?> label,
      Value<int> cardColor,
      Value<String?> metadata,
      Value<int> pasteCount,
      Value<String?> contentHash,
      Value<String?> thumbPath,
      Value<DateTime?> sourceModifiedAt,
      Value<DateTime?> brokenSince,
      Value<int> rowid,
    });
typedef $$ClipboardItemsTableUpdateCompanionBuilder =
    ClipboardItemsCompanion Function({
      Value<String> id,
      Value<String> content,
      Value<int> type,
      Value<DateTime> createdAt,
      Value<DateTime> modifiedAt,
      Value<String?> appSource,
      Value<bool> isPinned,
      Value<String?> label,
      Value<int> cardColor,
      Value<String?> metadata,
      Value<int> pasteCount,
      Value<String?> contentHash,
      Value<String?> thumbPath,
      Value<DateTime?> sourceModifiedAt,
      Value<DateTime?> brokenSince,
      Value<int> rowid,
    });

class $$ClipboardItemsTableFilterComposer
    extends Composer<_$_AppDatabase, $ClipboardItemsTable> {
  $$ClipboardItemsTableFilterComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnFilters<String> get id => $composableBuilder(
    column: $table.id,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get content => $composableBuilder(
    column: $table.content,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<int> get type => $composableBuilder(
    column: $table.type,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<DateTime> get createdAt => $composableBuilder(
    column: $table.createdAt,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<DateTime> get modifiedAt => $composableBuilder(
    column: $table.modifiedAt,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get appSource => $composableBuilder(
    column: $table.appSource,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<bool> get isPinned => $composableBuilder(
    column: $table.isPinned,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get label => $composableBuilder(
    column: $table.label,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<int> get cardColor => $composableBuilder(
    column: $table.cardColor,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get metadata => $composableBuilder(
    column: $table.metadata,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<int> get pasteCount => $composableBuilder(
    column: $table.pasteCount,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get contentHash => $composableBuilder(
    column: $table.contentHash,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get thumbPath => $composableBuilder(
    column: $table.thumbPath,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<DateTime> get sourceModifiedAt => $composableBuilder(
    column: $table.sourceModifiedAt,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<DateTime> get brokenSince => $composableBuilder(
    column: $table.brokenSince,
    builder: (column) => ColumnFilters(column),
  );
}

class $$ClipboardItemsTableOrderingComposer
    extends Composer<_$_AppDatabase, $ClipboardItemsTable> {
  $$ClipboardItemsTableOrderingComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnOrderings<String> get id => $composableBuilder(
    column: $table.id,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get content => $composableBuilder(
    column: $table.content,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<int> get type => $composableBuilder(
    column: $table.type,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<DateTime> get createdAt => $composableBuilder(
    column: $table.createdAt,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<DateTime> get modifiedAt => $composableBuilder(
    column: $table.modifiedAt,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get appSource => $composableBuilder(
    column: $table.appSource,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<bool> get isPinned => $composableBuilder(
    column: $table.isPinned,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get label => $composableBuilder(
    column: $table.label,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<int> get cardColor => $composableBuilder(
    column: $table.cardColor,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get metadata => $composableBuilder(
    column: $table.metadata,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<int> get pasteCount => $composableBuilder(
    column: $table.pasteCount,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get contentHash => $composableBuilder(
    column: $table.contentHash,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get thumbPath => $composableBuilder(
    column: $table.thumbPath,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<DateTime> get sourceModifiedAt => $composableBuilder(
    column: $table.sourceModifiedAt,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<DateTime> get brokenSince => $composableBuilder(
    column: $table.brokenSince,
    builder: (column) => ColumnOrderings(column),
  );
}

class $$ClipboardItemsTableAnnotationComposer
    extends Composer<_$_AppDatabase, $ClipboardItemsTable> {
  $$ClipboardItemsTableAnnotationComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  GeneratedColumn<String> get id =>
      $composableBuilder(column: $table.id, builder: (column) => column);

  GeneratedColumn<String> get content =>
      $composableBuilder(column: $table.content, builder: (column) => column);

  GeneratedColumn<int> get type =>
      $composableBuilder(column: $table.type, builder: (column) => column);

  GeneratedColumn<DateTime> get createdAt =>
      $composableBuilder(column: $table.createdAt, builder: (column) => column);

  GeneratedColumn<DateTime> get modifiedAt => $composableBuilder(
    column: $table.modifiedAt,
    builder: (column) => column,
  );

  GeneratedColumn<String> get appSource =>
      $composableBuilder(column: $table.appSource, builder: (column) => column);

  GeneratedColumn<bool> get isPinned =>
      $composableBuilder(column: $table.isPinned, builder: (column) => column);

  GeneratedColumn<String> get label =>
      $composableBuilder(column: $table.label, builder: (column) => column);

  GeneratedColumn<int> get cardColor =>
      $composableBuilder(column: $table.cardColor, builder: (column) => column);

  GeneratedColumn<String> get metadata =>
      $composableBuilder(column: $table.metadata, builder: (column) => column);

  GeneratedColumn<int> get pasteCount => $composableBuilder(
    column: $table.pasteCount,
    builder: (column) => column,
  );

  GeneratedColumn<String> get contentHash => $composableBuilder(
    column: $table.contentHash,
    builder: (column) => column,
  );

  GeneratedColumn<String> get thumbPath =>
      $composableBuilder(column: $table.thumbPath, builder: (column) => column);

  GeneratedColumn<DateTime> get sourceModifiedAt => $composableBuilder(
    column: $table.sourceModifiedAt,
    builder: (column) => column,
  );

  GeneratedColumn<DateTime> get brokenSince => $composableBuilder(
    column: $table.brokenSince,
    builder: (column) => column,
  );
}

class $$ClipboardItemsTableTableManager
    extends
        RootTableManager<
          _$_AppDatabase,
          $ClipboardItemsTable,
          ClipboardRow,
          $$ClipboardItemsTableFilterComposer,
          $$ClipboardItemsTableOrderingComposer,
          $$ClipboardItemsTableAnnotationComposer,
          $$ClipboardItemsTableCreateCompanionBuilder,
          $$ClipboardItemsTableUpdateCompanionBuilder,
          (
            ClipboardRow,
            BaseReferences<_$_AppDatabase, $ClipboardItemsTable, ClipboardRow>,
          ),
          ClipboardRow,
          PrefetchHooks Function()
        > {
  $$ClipboardItemsTableTableManager(
    _$_AppDatabase db,
    $ClipboardItemsTable table,
  ) : super(
        TableManagerState(
          db: db,
          table: table,
          createFilteringComposer: () =>
              $$ClipboardItemsTableFilterComposer($db: db, $table: table),
          createOrderingComposer: () =>
              $$ClipboardItemsTableOrderingComposer($db: db, $table: table),
          createComputedFieldComposer: () =>
              $$ClipboardItemsTableAnnotationComposer($db: db, $table: table),
          updateCompanionCallback:
              ({
                Value<String> id = const Value.absent(),
                Value<String> content = const Value.absent(),
                Value<int> type = const Value.absent(),
                Value<DateTime> createdAt = const Value.absent(),
                Value<DateTime> modifiedAt = const Value.absent(),
                Value<String?> appSource = const Value.absent(),
                Value<bool> isPinned = const Value.absent(),
                Value<String?> label = const Value.absent(),
                Value<int> cardColor = const Value.absent(),
                Value<String?> metadata = const Value.absent(),
                Value<int> pasteCount = const Value.absent(),
                Value<String?> contentHash = const Value.absent(),
                Value<String?> thumbPath = const Value.absent(),
                Value<DateTime?> sourceModifiedAt = const Value.absent(),
                Value<DateTime?> brokenSince = const Value.absent(),
                Value<int> rowid = const Value.absent(),
              }) => ClipboardItemsCompanion(
                id: id,
                content: content,
                type: type,
                createdAt: createdAt,
                modifiedAt: modifiedAt,
                appSource: appSource,
                isPinned: isPinned,
                label: label,
                cardColor: cardColor,
                metadata: metadata,
                pasteCount: pasteCount,
                contentHash: contentHash,
                thumbPath: thumbPath,
                sourceModifiedAt: sourceModifiedAt,
                brokenSince: brokenSince,
                rowid: rowid,
              ),
          createCompanionCallback:
              ({
                required String id,
                required String content,
                required int type,
                required DateTime createdAt,
                required DateTime modifiedAt,
                Value<String?> appSource = const Value.absent(),
                Value<bool> isPinned = const Value.absent(),
                Value<String?> label = const Value.absent(),
                Value<int> cardColor = const Value.absent(),
                Value<String?> metadata = const Value.absent(),
                Value<int> pasteCount = const Value.absent(),
                Value<String?> contentHash = const Value.absent(),
                Value<String?> thumbPath = const Value.absent(),
                Value<DateTime?> sourceModifiedAt = const Value.absent(),
                Value<DateTime?> brokenSince = const Value.absent(),
                Value<int> rowid = const Value.absent(),
              }) => ClipboardItemsCompanion.insert(
                id: id,
                content: content,
                type: type,
                createdAt: createdAt,
                modifiedAt: modifiedAt,
                appSource: appSource,
                isPinned: isPinned,
                label: label,
                cardColor: cardColor,
                metadata: metadata,
                pasteCount: pasteCount,
                contentHash: contentHash,
                thumbPath: thumbPath,
                sourceModifiedAt: sourceModifiedAt,
                brokenSince: brokenSince,
                rowid: rowid,
              ),
          withReferenceMapper: (p0) => p0
              .map((e) => (e.readTable(table), BaseReferences(db, table, e)))
              .toList(),
          prefetchHooksCallback: null,
        ),
      );
}

typedef $$ClipboardItemsTableProcessedTableManager =
    ProcessedTableManager<
      _$_AppDatabase,
      $ClipboardItemsTable,
      ClipboardRow,
      $$ClipboardItemsTableFilterComposer,
      $$ClipboardItemsTableOrderingComposer,
      $$ClipboardItemsTableAnnotationComposer,
      $$ClipboardItemsTableCreateCompanionBuilder,
      $$ClipboardItemsTableUpdateCompanionBuilder,
      (
        ClipboardRow,
        BaseReferences<_$_AppDatabase, $ClipboardItemsTable, ClipboardRow>,
      ),
      ClipboardRow,
      PrefetchHooks Function()
    >;

class $_AppDatabaseManager {
  final _$_AppDatabase _db;
  $_AppDatabaseManager(this._db);
  $$ClipboardItemsTableTableManager get clipboardItems =>
      $$ClipboardItemsTableTableManager(_db, _db.clipboardItems);
}
