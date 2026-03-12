// coverage:ignore-file
import 'dart:async';

import 'package:flutter/services.dart';

class LinuxShell {
  LinuxShell._();

  static const MethodChannel _methodChannel = MethodChannel(
    'copypaste/linux_shell',
  );
  static const EventChannel _eventChannel = EventChannel(
    'copypaste/linux_shell/events',
  );

  static Stream<String>? _events;

  static Stream<String> get events => _events ??= _eventChannel
      .receiveBroadcastStream()
      .map((dynamic event) {
        final map = Map<Object?, Object?>.from(event as Map);
        return map['type'] as String? ?? '';
      })
      .where((event) => event.isNotEmpty)
      .asBroadcastStream();

  static Future<bool> initTray({
    required String iconPath,
    required String showHideLabel,
    required String exitLabel,
    required String tooltip,
  }) async {
    final result = await _methodChannel.invokeMethod<bool>('initTray', {
      'iconPath': iconPath,
      'showHideLabel': showHideLabel,
      'exitLabel': exitLabel,
      'tooltip': tooltip,
    });
    return result ?? false;
  }

  static Future<bool> updateTray({
    required String iconPath,
    required String showHideLabel,
    required String exitLabel,
    required String tooltip,
  }) async {
    final result = await _methodChannel.invokeMethod<bool>('updateTray', {
      'iconPath': iconPath,
      'showHideLabel': showHideLabel,
      'exitLabel': exitLabel,
      'tooltip': tooltip,
    });
    return result ?? false;
  }

  static Future<void> destroyTray() async {
    try {
      await _methodChannel.invokeMethod<bool>('destroyTray');
    } catch (_) {}
  }

  static Future<bool> registerHotkey({
    required int virtualKey,
    required bool useCtrl,
    required bool useWin,
    required bool useAlt,
    required bool useShift,
  }) async {
    try {
      final result = await _methodChannel.invokeMethod<bool>('registerHotkey', {
        'virtualKey': virtualKey,
        'useCtrl': useCtrl,
        'useWin': useWin,
        'useAlt': useAlt,
        'useShift': useShift,
      });
      return result ?? false;
    } catch (_) {
      return false;
    }
  }

  static Future<void> unregisterHotkey() async {
    try {
      await _methodChannel.invokeMethod<bool>('unregisterHotkey');
    } catch (_) {}
  }
}
