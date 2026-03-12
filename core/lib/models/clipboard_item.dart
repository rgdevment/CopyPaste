import 'dart:io';

import 'package:uuid/uuid.dart';

import 'card_color.dart';
import 'clipboard_content_type.dart';

const _uuid = Uuid();
const _sentinel = Object();

class ClipboardItem {
  ClipboardItem({
    String? id,
    required this.content,
    required this.type,
    DateTime? createdAt,
    DateTime? modifiedAt,
    this.appSource,
    this.isPinned = false,
    this.label,
    this.cardColor = CardColor.none,
    this.metadata,
    this.pasteCount = 0,
    this.contentHash,
  }) : id = id ?? _uuid.v4(),
       createdAt = createdAt ?? DateTime.now().toUtc(),
       modifiedAt = modifiedAt ?? DateTime.now().toUtc();

  static const int maxLabelLength = 50;

  final String id;
  final String content;
  final ClipboardContentType type;
  final DateTime createdAt;
  final DateTime modifiedAt;
  final String? appSource;
  final bool isPinned;
  final String? label;
  final CardColor cardColor;
  final String? metadata;
  final int pasteCount;
  final String? contentHash;

  bool get isFileBasedType =>
      type == ClipboardContentType.file ||
      type == ClipboardContentType.folder ||
      type == ClipboardContentType.audio ||
      type == ClipboardContentType.video;

  ClipboardItem copyWith({
    String? content,
    ClipboardContentType? type,
    DateTime? createdAt,
    DateTime? modifiedAt,
    Object? appSource = _sentinel,
    bool? isPinned,
    Object? label = _sentinel,
    CardColor? cardColor,
    Object? metadata = _sentinel,
    int? pasteCount,
    Object? contentHash = _sentinel,
  }) => ClipboardItem(
    id: id,
    content: content ?? this.content,
    type: type ?? this.type,
    createdAt: createdAt ?? this.createdAt,
    modifiedAt: modifiedAt ?? this.modifiedAt,
    appSource: appSource == _sentinel ? this.appSource : appSource as String?,
    isPinned: isPinned ?? this.isPinned,
    label: label == _sentinel ? this.label : label as String?,
    cardColor: cardColor ?? this.cardColor,
    metadata: metadata == _sentinel ? this.metadata : metadata as String?,
    pasteCount: pasteCount ?? this.pasteCount,
    contentHash: contentHash == _sentinel
        ? this.contentHash
        : contentHash as String?,
  );

  bool isFileAvailable() {
    if (!isFileBasedType) return true;
    if (content.isEmpty) return false;
    final paths = content.split('\n').where((s) => s.isNotEmpty).toList();
    if (paths.isEmpty) return false;
    return paths.every(
      (p) => File(p).existsSync() || Directory(p).existsSync(),
    );
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) || other is ClipboardItem && id == other.id;

  @override
  int get hashCode => id.hashCode;
}
