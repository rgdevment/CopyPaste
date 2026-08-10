import 'package:copypaste/shell/windows_hotkey_channel.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const channelName = 'copypaste/test_windows_hotkeys';
  const methodChannel = MethodChannel(channelName);
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
  final calls = <MethodCall>[];

  setUp(() {
    calls.clear();
    messenger.setMockMethodCallHandler(methodChannel, (call) async {
      calls.add(call);
      if (call.method == 'register') {
        return <String, Object>{'success': true};
      }
      return null;
    });
  });

  tearDown(() {
    messenger.setMockMethodCallHandler(methodChannel, null);
  });

  test(
    'sends the complete binding and parses successful registration',
    () async {
      final channel = WindowsHotkeyChannel(channel: methodChannel);
      await channel.start((_) {});

      final response = await channel.register(
        id: 'plainPaste',
        virtualKey: 0x56,
        useCtrl: true,
        useWin: false,
        useAlt: true,
        useShift: true,
      );

      expect(response.success, isTrue);
      expect(calls, hasLength(1));
      expect(calls.single.method, 'register');
      expect(calls.single.arguments, {
        'id': 'plainPaste',
        'virtualKey': 0x56,
        'useCtrl': true,
        'useWin': false,
        'useAlt': true,
        'useShift': true,
      });

      await channel.dispose();
      expect(calls.last.method, 'unregisterAll');
    },
  );

  test('forwards WM_HOTKEY events from the runner exactly once', () async {
    final invoked = <String>[];
    final channel = WindowsHotkeyChannel(channel: methodChannel);
    await channel.start(invoked.add);

    final data = const StandardMethodCodec().encodeMethodCall(
      const MethodCall('hotkeyPressed', 'open'),
    );
    await messenger.handlePlatformMessage(channelName, data, (_) {});

    expect(invoked, ['open']);
    await channel.dispose();
  });

  test('preserves native registration diagnostics', () async {
    messenger.setMockMethodCallHandler(methodChannel, (call) async {
      if (call.method == 'register') {
        return <String, Object>{
          'success': false,
          'errorCode': 'registerFailed',
          'win32Error': 1409,
        };
      }
      return null;
    });
    final channel = WindowsHotkeyChannel(channel: methodChannel);
    await channel.start((_) {});

    final response = await channel.register(
      id: 'open',
      virtualKey: 0x43,
      useCtrl: true,
      useWin: false,
      useAlt: true,
      useShift: false,
    );

    expect(response.success, isFalse);
    expect(response.errorCode, 'registerFailed');
    expect(response.win32Error, 1409);
    await channel.dispose();
  });

  test('parses verified SendInput delivery', () async {
    messenger.setMockMethodCallHandler(methodChannel, (call) async {
      expect(call.method, 'sendPaste');
      return <String, Object>{
        'success': true,
        'sentInputs': 9,
        'expectedInputs': 9,
      };
    });

    final response = await WindowsHotkeyChannel.sendPaste(
      channel: methodChannel,
    );

    expect(response.success, isTrue);
    expect(response.sentInputs, 9);
    expect(response.expectedInputs, 9);
  });

  test('preserves SendInput failure diagnostics', () async {
    messenger.setMockMethodCallHandler(methodChannel, (call) async {
      return <String, Object>{
        'success': false,
        'sentInputs': 0,
        'expectedInputs': 9,
        'errorCode': 'sendInputFailed',
        'win32Error': 5,
      };
    });

    final response = await WindowsHotkeyChannel.sendPaste(
      channel: methodChannel,
    );

    expect(response.success, isFalse);
    expect(response.sentInputs, 0);
    expect(response.expectedInputs, 9);
    expect(response.errorCode, 'sendInputFailed');
    expect(response.win32Error, 5);
  });

  test(
    'forwards the destination window, focus and thread to the runner',
    () async {
      MethodCall? captured;
      messenger.setMockMethodCallHandler(methodChannel, (call) async {
        captured = call;
        return <String, Object>{'success': true};
      });

      await WindowsHotkeyChannel.sendPaste(
        targetHwnd: 460450,
        targetFocusHwnd: 461184,
        targetThreadId: 20832,
        channel: methodChannel,
      );

      expect(captured!.method, 'sendPaste');
      expect(captured!.arguments['targetHwnd'], 460450);
      expect(captured!.arguments['targetFocusHwnd'], 461184);
      expect(captured!.arguments['targetThreadId'], 20832);
    },
  );

  test('parses the focus repair diagnostics', () async {
    messenger.setMockMethodCallHandler(methodChannel, (call) async {
      return <String, Object>{
        'success': true,
        'sentInputs': 9,
        'expectedInputs': 9,
        'attached': true,
        'focusRepaired': true,
        'focusBefore': 0,
      };
    });

    final response = await WindowsHotkeyChannel.sendPaste(
      channel: methodChannel,
    );

    expect(response.attached, isTrue);
    expect(response.focusRepaired, isTrue);
    expect(response.focusBefore, 0);
  });

  test(
    'surfaces a destination that lost the foreground before injection',
    () async {
      messenger.setMockMethodCallHandler(methodChannel, (call) async {
        return <String, Object>{
          'success': false,
          'errorCode': 'targetNotForeground',
        };
      });

      final response = await WindowsHotkeyChannel.sendPaste(
        channel: methodChannel,
      );

      expect(response.success, isFalse);
      expect(response.errorCode, 'targetNotForeground');
      expect(response.focusRepaired, isFalse);
    },
  );
}
