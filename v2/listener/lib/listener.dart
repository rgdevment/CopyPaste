
import 'listener_platform_interface.dart';

class Listener {
  Future<String?> getPlatformVersion() {
    return ListenerPlatform.instance.getPlatformVersion();
  }
}
