import 'dart:async';
import 'dart:io';

import 'package:flutter/foundation.dart';

import 'windows_balloon.dart';

/// Cross-platform desktop notification helper for tray balloons.
///
/// Routes to the most idiomatic native channel per OS:
/// - Windows → `WindowsBalloon` (Shell_NotifyIconW via FFI).
/// - Linux   → `notify-send` (libnotify CLI shipped on every desktop;
///             talks D-Bus to `org.freedesktop.Notifications`, which all
///             modern DEs implement: GNOME Shell, KDE Plasma, Xfce…).
/// - macOS   → no-op (Mac uses dock badges + window UI; balloons would
///             collide with the system Notification Center conventions).
///
/// Always returns a Future that completes — never throws.
class DesktopNotifier {
  DesktopNotifier._();

  /// Injectable process runner. Override in tests to avoid spawning real
  /// system processes.
  @visibleForTesting
  static Future<ProcessResult> Function(String, List<String>)?
  processRunnerOverride;

  /// Shows a transient notification with [title] and [body].
  /// Returns true when the platform layer accepted the request.
  static Future<bool> show({
    required String title,
    required String body,
  }) async {
    if (Platform.isWindows) {
      return WindowsBalloon.show(title: title, body: body);
    }
    if (Platform.isLinux) {
      return _showLinux(title: title, body: body);
    }
    return false;
  }

  /// Spawns `notify-send` to push a notification through D-Bus
  /// (`org.freedesktop.Notifications`). Silent on systems without it.
  ///
  /// Flags:
  ///   --app-name=CopyPaste → grouping / branding in the shell.
  ///   --icon=copypaste     → DE looks up the icon by name in the theme;
  ///                          falls back gracefully if not installed.
  ///   --expire-time=7000   → matches Windows balloon dismiss window.
  static Future<bool> _showLinux({
    required String title,
    required String body,
  }) async {
    final runner = processRunnerOverride ?? Process.run;
    try {
      final result = await runner('notify-send', <String>[
        '--app-name=CopyPaste',
        '--icon=copypaste',
        '--expire-time=7000',
        title,
        body,
      ]);
      return result.exitCode == 0;
    } on ProcessException {
      // notify-send not installed (rare; ships with libnotify-bin).
      return false;
    } catch (_) {
      return false;
    }
  }
}
