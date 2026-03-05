import 'dart:io';

class UrlHelper {
  UrlHelper._();

  static Future<void> open(String url) async {
    if (Platform.isWindows) {
      await Process.start('cmd', ['/c', 'start', '', url], runInShell: true);
    } else if (Platform.isMacOS) {
      await Process.start('open', [url]);
    } else if (Platform.isLinux) {
      await Process.start('xdg-open', [url]);
    }
  }
}
