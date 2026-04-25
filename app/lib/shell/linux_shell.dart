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

  static Future<bool> awaitEvent(
    String type, {
    Duration timeout = const Duration(milliseconds: 300),
  }) async {
    final completer = Completer<bool>();
    final sub = events.listen((event) {
      if (event == type && !completer.isCompleted) completer.complete(true);
    });
    final timer = Timer(timeout, () {
      if (!completer.isCompleted) completer.complete(false);
    });
    try {
      return await completer.future;
    } finally {
      timer.cancel();
      await sub.cancel();
    }
  }

  static Future<TrayResponse> initTray({
    required String iconPath,
    required String showHideLabel,
    required String exitLabel,
    required String tooltip,
  }) async {
    return _invokeTrayMethod('initTray', {
      'iconPath': iconPath,
      'showHideLabel': showHideLabel,
      'exitLabel': exitLabel,
      'tooltip': tooltip,
    });
  }

  static Future<TrayResponse> updateTray({
    required String iconPath,
    required String showHideLabel,
    required String exitLabel,
    required String tooltip,
  }) async {
    return _invokeTrayMethod('updateTray', {
      'iconPath': iconPath,
      'showHideLabel': showHideLabel,
      'exitLabel': exitLabel,
      'tooltip': tooltip,
    });
  }

  static Future<TrayResponse> _invokeTrayMethod(
    String method,
    Map<String, Object?> args,
  ) async {
    try {
      final result = await _methodChannel.invokeMethod<Object>(method, args);
      if (result is Map) {
        final map = Map<Object?, Object?>.from(result);
        final code = map['errorCode'];
        return TrayResponse(
          success: map['success'] == true,
          errorCode: code is String ? code : null,
        );
      }
      if (result is bool) {
        return TrayResponse(success: result);
      }
      return const TrayResponse(success: false, errorCode: 'unknown');
    } catch (e) {
      AppLogger.error('LinuxShell.$method failed: $e');
      return const TrayResponse(success: false, errorCode: 'channelError');
    }
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
      final result = await _methodChannel
          .invokeMethod<Object>('registerHotkey', {
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
      return const HotkeyRegisterResponse(
        success: false,
        errorCode: 'channelError',
      );
    }
  }

  static Future<void> unregisterHotkey() async {
    try {
      await _methodChannel.invokeMethod<bool>('unregisterHotkey');
    } catch (e) {
      AppLogger.error('LinuxShell.unregisterHotkey failed: $e');
    }
  }

  static Future<void> focusWindow() async {
    try {
      await _methodChannel.invokeMethod<bool>('focusWindow');
    } catch (e) {
      AppLogger.error('LinuxShell.focusWindow failed: $e');
    }
  }

  static Future<CursorMonitorInfo?> getCursorMonitor() async {
    try {
      final result = await _methodChannel.invokeMethod<Object>(
        'getCursorMonitor',
      );
      if (result is! Map) return null;
      return CursorMonitorInfo(
        cursorX: (result['cursorX'] as num?)?.toDouble() ?? 0,
        cursorY: (result['cursorY'] as num?)?.toDouble() ?? 0,
        x: (result['x'] as num?)?.toDouble() ?? 0,
        y: (result['y'] as num?)?.toDouble() ?? 0,
        width: (result['width'] as num?)?.toDouble() ?? 0,
        height: (result['height'] as num?)?.toDouble() ?? 0,
        scaleFactor: (result['scaleFactor'] as num?)?.toDouble() ?? 1.0,
      );
    } catch (e) {
      AppLogger.error('LinuxShell.getCursorMonitor failed: $e');
      return null;
    }
  }

  static Future<InputFocusInfo?> getInputFocus() async {
    try {
      final result = await _methodChannel.invokeMethod<Object>('getInputFocus');
      if (result is! Map) return null;
      return InputFocusInfo(
        ownsFocus: result['ownsFocus'] as bool? ?? false,
        focusWindow: (result['focusWindow'] as num?)?.toInt() ?? 0,
        ownWindow: (result['ownWindow'] as num?)?.toInt() ?? 0,
      );
    } catch (e) {
      AppLogger.error('LinuxShell.getInputFocus failed: $e');
      return null;
    }
  }
}

class CursorMonitorInfo {
  const CursorMonitorInfo({
    required this.cursorX,
    required this.cursorY,
    required this.x,
    required this.y,
    required this.width,
    required this.height,
    required this.scaleFactor,
  });

  final double cursorX;
  final double cursorY;
  final double x;
  final double y;
  final double width;
  final double height;
  final double scaleFactor;
}

class InputFocusInfo {
  const InputFocusInfo({
    required this.ownsFocus,
    required this.focusWindow,
    required this.ownWindow,
  });

  final bool ownsFocus;
  final int focusWindow;
  final int ownWindow;
}

class HotkeyRegisterResponse {
  const HotkeyRegisterResponse({required this.success, this.errorCode});

  final bool success;
  final String? errorCode;
}

class TrayResponse {
  const TrayResponse({required this.success, this.errorCode});

  final bool success;
  final String? errorCode;
}
