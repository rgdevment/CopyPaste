import 'dart:io' show Platform;

import 'package:flutter/foundation.dart';

enum HotkeyRegistrationStatus { registered, fallbackRegistered, failed }

@immutable
class HotkeyBinding {
  const HotkeyBinding({
    required this.virtualKey,
    required this.keyName,
    required this.useCtrl,
    required this.useWin,
    required this.useAlt,
    required this.useShift,
  });

  final int virtualKey;
  final String keyName;
  final bool useCtrl;
  final bool useWin;
  final bool useAlt;
  final bool useShift;

  String label({bool isMac = false}) {
    final parts = <String>[];
    final mac = isMac || Platform.isMacOS;
    if (mac) {
      if (useCtrl) parts.add('Control');
      if (useAlt) parts.add('Option');
      if (useShift) parts.add('Shift');
      if (useWin) parts.add('Command');
    } else {
      if (useCtrl) parts.add('Ctrl');
      if (useWin) parts.add('Win');
      if (useAlt) parts.add('Alt');
      if (useShift) parts.add('Shift');
    }
    parts.add(keyName);
    return parts.join('+');
  }

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    return other is HotkeyBinding &&
        other.virtualKey == virtualKey &&
        other.keyName == keyName &&
        other.useCtrl == useCtrl &&
        other.useWin == useWin &&
        other.useAlt == useAlt &&
        other.useShift == useShift;
  }

  @override
  int get hashCode =>
      Object.hash(virtualKey, keyName, useCtrl, useWin, useAlt, useShift);
}

@immutable
class HotkeyRegistrationResult {
  const HotkeyRegistrationResult({
    required this.status,
    required this.requestedBinding,
    this.effectiveBinding,
  });

  final HotkeyRegistrationStatus status;
  final HotkeyBinding requestedBinding;
  final HotkeyBinding? effectiveBinding;
}
