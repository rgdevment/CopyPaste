// coverage:ignore-file
import 'dart:async';

import 'package:core/core.dart';
import 'package:flutter/services.dart';

class LinuxShell {
  LinuxShell._();

  static const MethodChannel _methodChannel = MethodChannel(
    'copypaste/linux_shell',
  );
  static const EventChannel _eventChannel = EventChannel(
    'copypaste/linux_shell/events',
  );

  static StreamController<String>? _eventsController;
  static StreamSubscription<dynamic>? _eventChannelSubscription;

  static Stream<String> get events {
    if (_eventsController == null) {
      _eventsController = StreamController<String>.broadcast();
      _eventChannelSubscription = _eventChannel.receiveBroadcastStream().listen(
        (dynamic event) {
          if (event is! Map) return;
          final map = Map<Object?, Object?>.from(event);
          final type = map['type'] as String? ?? '';
          if (type.isNotEmpty) _eventsController?.add(type);
        },
        onError: (Object error) => _eventsController?.addError(error),
      );
    }
    return _eventsController!.stream;
  }

  static Future<void> dispose() async {
    await _eventChannelSubscription?.cancel();
    _eventChannelSubscription = null;
    await _eventsController?.close();
    _eventsController = null;
  }

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
    } catch (e) {
      AppLogger.error('LinuxShell.destroyTray failed: $e');
    }
  }

  static Future<HotkeyRegisterResponse> registerHotkey({
    required int virtualKey,
    required bool useCtrl,
    required bool useWin,
    required bool useAlt,
    required bool useShift,
  }) async {
    try {
      final result = await _methodChannel.invokeMethod<Object>('registerHotkey', {
        'virtualKey': virtualKey,
        'useCtrl': useCtrl,
        'useWin': useWin,
        'useAlt': useAlt,
        'useShift': useShift,
      });
      if (result is Map) {
        final map = Map<Object?, Object?>.from(result);
        final success = map['success'] == true;
        final code = map['errorCode'];
        return HotkeyRegisterResponse(
          success: success,
          errorCode: code is String ? code : null,
        );
      }
      if (result is bool) {
        return HotkeyRegisterResponse(success: result);
      }
      return const HotkeyRegisterResponse(success: false, errorCode: 'unknown');
    } catch (e) {
      AppLogger.error('LinuxShell.registerHotkey failed: $e');
      return const HotkeyRegisterResponse(success: false, errorCode: 'channelError');
    }
  }

  static Future<void> unregisterHotkey() async {
    try {
      await _methodChannel.invokeMethod<bool>('unregisterHotkey');
    } catch (e) {
      AppLogger.error('LinuxShell.unregisterHotkey failed: $e');
    }
  }

  /// Raises and focuses the GTK window using the X11 hotkey event timestamp,
  /// bypassing GNOME's focus-stealing prevention.
  static Future<void> focusWindow() async {
    try {
      await _methodChannel.invokeMethod<bool>('focusWindow');
    } catch (e) {
      AppLogger.error('LinuxShell.focusWindow failed: $e');
    }
  }
}

class HotkeyRegisterResponse {
  const HotkeyRegisterResponse({required this.success, this.errorCode});

  final bool success;
  final String? errorCode;
}
