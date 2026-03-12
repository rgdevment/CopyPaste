import 'dart:typed_data';

import 'package:core/core.dart';

class ClipboardEvent {
  const ClipboardEvent({
    required this.type,
    required this.contentHash,
    this.text,
    this.bytes,
    this.files,
    this.source,
    this.rtfBytes,
    this.htmlBytes,
  });

  factory ClipboardEvent.fromMap(Map<Object?, Object?> map) {
    final typeVal = map['type'] as int? ?? -1;
    return ClipboardEvent(
      type: ClipboardContentType.fromValue(typeVal),
      contentHash: map['contentHash'] as String? ?? '',
      text: map['text'] as String?,
      bytes: map['bytes'] as Uint8List?,
      files: (map['files'] as List<Object?>?)?.whereType<String>().toList(),
      source: map['source'] as String?,
      rtfBytes: map['rtf'] as Uint8List?,
      htmlBytes: map['html'] as Uint8List?,
    );
  }

  final ClipboardContentType type;

  final String contentHash;

  final String? text;

  final Uint8List? bytes;

  final List<String>? files;

  final String? source;

  final Uint8List? rtfBytes;

  final Uint8List? htmlBytes;
}
