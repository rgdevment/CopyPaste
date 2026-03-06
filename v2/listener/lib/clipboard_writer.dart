import 'dart:convert';

import 'package:flutter/services.dart';

class ClipboardWriter {
  static const MethodChannel _channel = MethodChannel(
    'copypaste/clipboard_writer',
  );

  static Future<bool> setText(
    String content, {
    String? metadata,
    bool plainText = false,
  }) async {
    final args = <String, Object?>{
      'type': 0,
      'content': content,
      'plainText': plainText,
    };

    if (!plainText && metadata != null && metadata.isNotEmpty) {
      try {
        final json = jsonDecode(metadata) as Map<String, dynamic>;
        final rtfB64 = json['rtf'] as String?;
        if (rtfB64 != null && rtfB64.isNotEmpty) {
          args['rtf'] = base64Decode(rtfB64);
        }
        final htmlB64 = json['html'] as String?;
        if (htmlB64 != null && htmlB64.isNotEmpty) {
          args['html'] = base64Decode(htmlB64);
        }
      } catch (_) {}
    }

    final result = await _channel.invokeMethod<bool>(
      'setClipboardContent',
      args,
    );
    return result ?? false;
  }

  static Future<bool> setImage(String imagePath) async {
    final result = await _channel.invokeMethod<bool>(
      'setClipboardContent',
      <String, Object?>{'type': 1, 'content': imagePath},
    );
    return result ?? false;
  }

  static Future<bool> setFiles(String content, int typeValue) async {
    final result = await _channel.invokeMethod<bool>(
      'setClipboardContent',
      <String, Object?>{'type': typeValue, 'content': content},
    );
    return result ?? false;
  }

  static Future<bool> setFromItem({
    required int typeValue,
    required String content,
    String? metadata,
    bool plainText = false,
  }) async {
    switch (typeValue) {
      case 0:
      case 4:
        return setText(content, metadata: metadata, plainText: plainText);
      case 1:
        return setImage(content);
      case 2:
      case 3:
      case 5:
      case 6:
        return setFiles(content, typeValue);
      default:
        return setText(content, plainText: true);
    }
  }

  static Future<Map<String, Object?>?> getMediaInfo(String path) async {
    try {
      final result = await _channel.invokeMapMethod<String, Object?>(
        'getMediaInfo',
        <String, Object?>{'path': path},
      );
      return result;
    } catch (_) {
      return null;
    }
  }
}
