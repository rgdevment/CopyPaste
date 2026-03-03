import 'dart:io';

import 'package:uuid/uuid.dart';

import 'card_color.dart';
import 'clipboard_content_type.dart';

const _uuid = Uuid();

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
  })  : id = id ?? _uuid.v4(),
        createdAt = createdAt ?? DateTime.now().toUtc(),
        modifiedAt = modifiedAt ?? DateTime.now().toUtc();

  static const int maxLabelLength = 40;

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
    String? appSource,
    bool? isPinned,
    String? label,
    CardColor? cardColor,
    String? metadata,
    int? pasteCount,
    String? contentHash,
  }) =>
      ClipboardItem(
        id: id,
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
      );

  bool isFileAvailable() {
    if (!isFileBasedType) return true;
    if (content.isEmpty) return false;
    final paths =
        content.split('\n').where((s) => s.isNotEmpty).toList();
    if (paths.isEmpty) return false;
    return File(paths.first).existsSync() ||
        Directory(paths.first).existsSync();
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is ClipboardItem && id == other.id;

  @override
  int get hashCode => id.hashCode;
}
