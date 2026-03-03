import 'package:plugin_platform_interface/plugin_platform_interface.dart';

import 'listener_method_channel.dart';

abstract class ListenerPlatform extends PlatformInterface {
  /// Constructs a ListenerPlatform.
  ListenerPlatform() : super(token: _token);

  static final Object _token = Object();

  static ListenerPlatform _instance = MethodChannelListener();

  /// The default instance of [ListenerPlatform] to use.
  ///
  /// Defaults to [MethodChannelListener].
  static ListenerPlatform get instance => _instance;

  /// Platform-specific implementations should set this with their own
  /// platform-specific class that extends [ListenerPlatform] when
  /// they register themselves.
  static set instance(ListenerPlatform instance) {
    PlatformInterface.verifyToken(instance, _token);
    _instance = instance;
  }

  Future<String?> getPlatformVersion() {
    throw UnimplementedError('platformVersion() has not been implemented.');
  }
}
