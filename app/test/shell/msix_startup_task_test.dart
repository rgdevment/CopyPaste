import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/shell/msix_startup_task.dart';

const _channel = MethodChannel('copypaste/startup_task');

void _setHandler(Future<Object?> Function(MethodCall) handler) {
  TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
      .setMockMethodCallHandler(_channel, handler);
}

void _clearHandler() {
  TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
      .setMockMethodCallHandler(_channel, null);
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  tearDown(_clearHandler);

  group('MsixStartupTask.getState', () {
    test('returns enabled when channel replies "enabled"', () async {
      _setHandler((_) async => 'enabled');
      final state = await MsixStartupTask.getState('TestTaskId');
      expect(state, MsixStartupTaskState.enabled);
    });

    test('returns disabled when channel replies "disabled"', () async {
      _setHandler((_) async => 'disabled');
      final state = await MsixStartupTask.getState('TestTaskId');
      expect(state, MsixStartupTaskState.disabled);
    });

    test(
      'returns disabledByUser when channel replies "disabledByUser"',
      () async {
        _setHandler((_) async => 'disabledByUser');
        final state = await MsixStartupTask.getState('TestTaskId');
        expect(state, MsixStartupTaskState.disabledByUser);
      },
    );

    test(
      'returns disabledByPolicy when channel replies "disabledByPolicy"',
      () async {
        _setHandler((_) async => 'disabledByPolicy');
        final state = await MsixStartupTask.getState('TestTaskId');
        expect(state, MsixStartupTaskState.disabledByPolicy);
      },
    );

    test(
      'returns enabledByPolicy when channel replies "enabledByPolicy"',
      () async {
        _setHandler((_) async => 'enabledByPolicy');
        final state = await MsixStartupTask.getState('TestTaskId');
        expect(state, MsixStartupTaskState.enabledByPolicy);
      },
    );

    test('returns unknown for unrecognised reply', () async {
      _setHandler((_) async => 'someFutureState');
      final state = await MsixStartupTask.getState('TestTaskId');
      expect(state, MsixStartupTaskState.unknown);
    });

    test('passes the taskId as argument', () async {
      MethodCall? captured;
      _setHandler((call) async {
        captured = call;
        return 'enabled';
      });
      await MsixStartupTask.getState('CopyPasteStartup');
      expect((captured!.arguments as Map)['taskId'], 'CopyPasteStartup');
    });

    test('returns null on PlatformException', () async {
      _setHandler((_) async => throw PlatformException(code: 'winrt_error'));
      final state = await MsixStartupTask.getState('TestTaskId');
      expect(state, isNull);
    });
  });

  group('MsixStartupTask.enable', () {
    test('invokes "enable" method on the channel', () async {
      MethodCall? captured;
      _setHandler((call) async {
        captured = call;
        return 'enabled';
      });
      await MsixStartupTask.enable('CopyPasteStartup');
      expect(captured!.method, 'enable');
    });

    test('returns the state from the channel reply', () async {
      _setHandler((_) async => 'disabledByUser');
      final state = await MsixStartupTask.enable('CopyPasteStartup');
      expect(state, MsixStartupTaskState.disabledByUser);
    });

    test('returns null on PlatformException', () async {
      _setHandler((_) async => throw PlatformException(code: 'winrt_error'));
      final state = await MsixStartupTask.enable('CopyPasteStartup');
      expect(state, isNull);
    });
  });

  group('MsixStartupTask.disable', () {
    test('invokes "disable" method on the channel', () async {
      MethodCall? captured;
      _setHandler((call) async {
        captured = call;
        return 'disabled';
      });
      await MsixStartupTask.disable('CopyPasteStartup');
      expect(captured!.method, 'disable');
    });

    test('returns the state from the channel reply', () async {
      _setHandler((_) async => 'disabled');
      final state = await MsixStartupTask.disable('CopyPasteStartup');
      expect(state, MsixStartupTaskState.disabled);
    });

    test('returns null on PlatformException', () async {
      _setHandler((_) async => throw PlatformException(code: 'winrt_error'));
      final state = await MsixStartupTask.disable('CopyPasteStartup');
      expect(state, isNull);
    });
  });
}
