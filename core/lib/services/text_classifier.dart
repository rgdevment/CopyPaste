import 'dart:convert';

import '../models/clipboard_content_type.dart';

abstract final class TextClassifier {
  static final _email = RegExp(
    r'^[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}$',
  );

  static final _phone = RegExp(r'^\+?[\d\s\(\)\-]{7,25}$');

  static final _hexColor = RegExp(
    r'^#([0-9a-fA-F]{3}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$',
  );

  static final _rgbColor = RegExp(
    r'^rgba?\(\s*\d{1,3}\s*,\s*\d{1,3}\s*,\s*\d{1,3}(\s*,\s*[\d.]+)?\s*\)$',
    caseSensitive: false,
  );

  static final _hslColor = RegExp(
    r'^hsla?\(\s*\d{1,3}\s*,\s*\d{1,3}%\s*,\s*\d{1,3}%(\s*,\s*[\d.]+)?\s*\)$',
    caseSensitive: false,
  );

  static final _ip = RegExp(
    r'^((25[0-5]|2[0-4]\d|[01]?\d\d?)\.){3}(25[0-5]|2[0-4]\d|[01]?\d\d?)$',
  );

  static final _uuid = RegExp(
    r'^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$',
    caseSensitive: false,
  );

  static ClipboardContentType classify(String content) {
    final t = content.trim();
    if (t.isEmpty) return ClipboardContentType.text;

    if (!t.contains('\n')) {
      if (_email.hasMatch(t)) return ClipboardContentType.email;
      if (_isPhone(t)) return ClipboardContentType.phone;
      if (_hexColor.hasMatch(t) ||
          _rgbColor.hasMatch(t) ||
          _hslColor.hasMatch(t)) {
        return ClipboardContentType.color;
      }
      if (_ip.hasMatch(t)) return ClipboardContentType.ip;
      if (_uuid.hasMatch(t)) return ClipboardContentType.uuid;
    }

    if (_isJson(t)) return ClipboardContentType.json;
    return ClipboardContentType.text;
  }

  static bool _isPhone(String value) {
    if (!_phone.hasMatch(value)) return false;
    final digits = value.replaceAll(RegExp(r'\D'), '');
    return digits.length >= 7 && digits.length <= 15;
  }

  static bool _isJson(String value) {
    final t = value.trim();
    if (!((t.startsWith('{') && t.endsWith('}')) ||
        (t.startsWith('[') && t.endsWith(']')))) {
      return false;
    }
    try {
      jsonDecode(t);
      return true;
    } catch (_) {
      return false;
    }
  }
}
