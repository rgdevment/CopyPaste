// coverage:ignore-file
import 'dart:io';

import 'package:core/core.dart';
import 'package:flutter/services.dart';

enum MsixStartupTaskState {
  unknown,
  disabled,
  disabledByUser,
  disabledByPolicy,
  enabled,
  enabledByPolicy,
}

MsixStartupTaskState _parseState(Object? raw) {
  switch (raw) {
    case 'disabled':
      return MsixStartupTaskState.disabled;
    case 'disabledByUser':
      return MsixStartupTaskState.disabledByUser;
    case 'disabledByPolicy':
      return MsixStartupTaskState.disabledByPolicy;
    case 'enabled':
      return MsixStartupTaskState.enabled;
    case 'enabledByPolicy':
      return MsixStartupTaskState.enabledByPolicy;
    default:
      return MsixStartupTaskState.unknown;
  }
}

class MsixStartupTask {
  MsixStartupTask._();

  static const _channel = MethodChannel('copypaste/startup_task');

  static Future<MsixStartupTaskState?> getState(String taskId) async {
    if (!Platform.isWindows) return null;
    try {
      final raw = await _channel.invokeMethod<String>('getState', {
        'taskId': taskId,
      });
      return _parseState(raw);
    } on PlatformException catch (e) {
      AppLogger.error(
        'MsixStartupTask.getState failed: ${e.code} ${e.message}',
      );
      return null;
    }
  }

  static Future<MsixStartupTaskState?> enable(String taskId) async {
    if (!Platform.isWindows) return null;
    try {
      final raw = await _channel.invokeMethod<String>('enable', {
        'taskId': taskId,
      });
      return _parseState(raw);
    } on PlatformException catch (e) {
      AppLogger.error('MsixStartupTask.enable failed: ${e.code} ${e.message}');
      return null;
    }
  }

  static Future<MsixStartupTaskState?> disable(String taskId) async {
    if (!Platform.isWindows) return null;
    try {
      final raw = await _channel.invokeMethod<String>('disable', {
        'taskId': taskId,
      });
      return _parseState(raw);
    } on PlatformException catch (e) {
      AppLogger.error('MsixStartupTask.disable failed: ${e.code} ${e.message}');
      return null;
    }
  }
}
