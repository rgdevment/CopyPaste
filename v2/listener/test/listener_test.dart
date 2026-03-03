import 'package:flutter_test/flutter_test.dart';
import 'package:listener/listener.dart';
import 'package:listener/listener_platform_interface.dart';
import 'package:listener/listener_method_channel.dart';
import 'package:plugin_platform_interface/plugin_platform_interface.dart';

class MockListenerPlatform
    with MockPlatformInterfaceMixin
    implements ListenerPlatform {
  @override
  Future<String?> getPlatformVersion() => Future.value('42');
}

void main() {
  final ListenerPlatform initialPlatform = ListenerPlatform.instance;

  test('$MethodChannelListener is the default instance', () {
    expect(initialPlatform, isInstanceOf<MethodChannelListener>());
  });

  test('getPlatformVersion', () async {
    final Listener listenerPlugin = Listener();
    final MockListenerPlatform fakePlatform = MockListenerPlatform();
    ListenerPlatform.instance = fakePlatform;

    expect(await listenerPlugin.getPlatformVersion(), '42');
  });
}
