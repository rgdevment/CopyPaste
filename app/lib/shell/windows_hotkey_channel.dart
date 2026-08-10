import 'package:core/core.dart';
import 'package:flutter/services.dart';

class WindowsHotkeyRegistrationResponse {
  const WindowsHotkeyRegistrationResponse({
    required this.success,
    this.errorCode,
    this.win32Error,
  });

  final bool success;
  final String? errorCode;
  final int? win32Error;
}

class WindowsPasteInputResponse {
  const WindowsPasteInputResponse({
    required this.success,
    this.sentInputs,
    this.expectedInputs,
    this.errorCode,
    this.win32Error,
    this.attached = false,
    this.focusRepaired = false,
    this.focusBefore,
  });

  final bool success;
  final int? sentInputs;
  final int? expectedInputs;
  final String? errorCode;
  final int? win32Error;
  final bool attached;
  final bool focusRepaired;
  final int? focusBefore;
}

/// Owns the Windows runner channel backed by RegisterHotKey/WM_HOTKEY.
///
/// hotkey_manager's key-up callback is macOS-only. Keeping the Windows path
/// here makes registration results explicit and avoids retaining callbacks in
/// the plugin singleton after settings changes or application shutdown.
class WindowsHotkeyChannel {
  WindowsHotkeyChannel({
    MethodChannel channel = const MethodChannel(_channelName),
  }) : _channel = channel;

  static const _channelName = 'copypaste/windows_hotkeys';

  final MethodChannel _channel;
  void Function(String id)? _onPressed;

  Future<void> start(void Function(String id) onPressed) async {
    _onPressed = onPressed;
    _channel.setMethodCallHandler(_handleNativeCall);
  }

  Future<Object?> _handleNativeCall(MethodCall call) async {
    if (call.method != 'hotkeyPressed') return null;
    final id = call.arguments;
    if (id is! String) {
      AppLogger.warn('Ignored Windows hotkey event with an invalid id');
      return null;
    }
    AppLogger.info('Windows hotkey invoked: id=$id');
    _onPressed?.call(id);
    return null;
  }

  Future<WindowsHotkeyRegistrationResponse> register({
    required String id,
    required int virtualKey,
    required bool useCtrl,
    required bool useWin,
    required bool useAlt,
    required bool useShift,
  }) async {
    final response = await _channel.invokeMethod<Object?>('register', {
      'id': id,
      'virtualKey': virtualKey,
      'useCtrl': useCtrl,
      'useWin': useWin,
      'useAlt': useAlt,
      'useShift': useShift,
    });
    if (response is! Map) {
      return const WindowsHotkeyRegistrationResponse(
        success: false,
        errorCode: 'invalidNativeResponse',
      );
    }
    return WindowsHotkeyRegistrationResponse(
      success: response['success'] == true,
      errorCode: response['errorCode'] as String?,
      win32Error: response['win32Error'] as int?,
    );
  }

  Future<void> dispose() async {
    // Break the owner reference before crossing the platform boundary. Even if
    // teardown fails, the channel can no longer retain the app State callback.
    _onPressed = null;
    try {
      await _channel.invokeMethod<void>('unregisterAll');
    } finally {
      _channel.setMethodCallHandler(null);
    }
  }

  static Future<WindowsPasteInputResponse> sendPaste({
    int targetHwnd = 0,
    int targetFocusHwnd = 0,
    int targetThreadId = 0,
    MethodChannel channel = const MethodChannel(_channelName),
  }) async {
    final response = await channel.invokeMethod<Object?>('sendPaste', {
      'targetHwnd': targetHwnd,
      'targetFocusHwnd': targetFocusHwnd,
      'targetThreadId': targetThreadId,
    });
    if (response is! Map) {
      return const WindowsPasteInputResponse(
        success: false,
        errorCode: 'invalidNativeResponse',
      );
    }
    return WindowsPasteInputResponse(
      success: response['success'] == true,
      sentInputs: response['sentInputs'] as int?,
      expectedInputs: response['expectedInputs'] as int?,
      errorCode: response['errorCode'] as String?,
      win32Error: response['win32Error'] as int?,
      attached: response['attached'] == true,
      focusRepaired: response['focusRepaired'] == true,
      focusBefore: response['focusBefore'] as int?,
    );
  }
}
