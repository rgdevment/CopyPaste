import 'dart:io';

import 'windows_balloon.dart';

/// Cross-platform desktop notification helper for tray balloons.
///
/// Routes to the most idiomatic native channel per OS:
/// - Windows → `WindowsBalloon` (Shell_NotifyIconW via FFI).
/// - macOS   → no-op (Mac uses dock badges + window UI; balloons would
///             collide with the system Notification Center conventions).
///
/// Always returns a Future that completes — never throws.
class DesktopNotifier {
  DesktopNotifier._();

  /// Shows a transient notification with [title] and [body].
  /// Returns true when the platform layer accepted the request.
  static Future<bool> show({
    required String title,
    required String body,
  }) async {
    if (Platform.isWindows) {
      return WindowsBalloon.show(title: title, body: body);
    }
    return false;
  }
}
