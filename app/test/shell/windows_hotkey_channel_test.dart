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
}
