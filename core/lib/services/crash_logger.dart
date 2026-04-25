import 'dart:io';

import 'package:path/path.dart' as p;

class CrashLogger {
  CrashLogger._(); // coverage:ignore-line

  static const String fileName = 'crash.log';
  static const int _maxSizeBytes = 512 * 1024;

  static String? _filePath;

  static String? get filePath => _filePath;

  static void initialize(String baseDir) {
    try {
      Directory(baseDir).createSync(recursive: true);
      _filePath = p.join(baseDir, fileName);
    } catch (_) {
      _filePath = null;
    }
  }

  static String? resolveBootstrapPath() {
    try {
      final base = _bootstrapBaseDir();
      if (base == null) return null;
      Directory(base).createSync(recursive: true);
      return p.join(base, fileName);
    } catch (_) {
      return null;
    }
  }

  static void report(
    Object error,
    StackTrace? stack, {
    String context = '',
    String? overridePath,
  }) {
    final target = overridePath ?? _filePath ?? resolveBootstrapPath();
    if (target == null) return;
    try {
      final file = File(target);
      if (file.existsSync() && file.lengthSync() > _maxSizeBytes) {
        file.writeAsStringSync('', flush: true);
      }
      final ts = DateTime.now().toUtc().toIso8601String();
      final sb = StringBuffer()
        ..writeln('==== $ts ====')
        ..writeln(
          'Platform: ${Platform.operatingSystem} '
          '${Platform.operatingSystemVersion}',
        )
        ..writeln('Dart: ${Platform.version}');
      if (context.isNotEmpty) sb.writeln('Context: $context');
      sb.writeln('Error: ${redact(error.toString())}');
      if (stack != null) {
        sb.writeln('Stack:');
        sb.writeln(redact(stack.toString()));
      }
      sb.writeln();
      file.writeAsStringSync(sb.toString(), mode: FileMode.append, flush: true);
    } catch (_) {}
  }

  static String redact(String input) {
    var out = input;
    final userProfile = Platform.environment['USERPROFILE'];
    final home = Platform.environment['HOME'];
    final username =
        Platform.environment['USERNAME'] ?? Platform.environment['USER'];
    for (final raw in [userProfile, home]) {
      if (raw != null && raw.isNotEmpty) {
        out = out.replaceAll(raw, '<HOME>');
      }
    }
    if (username != null && username.isNotEmpty && username.length > 1) {
      out = out.replaceAll(
        RegExp(r'\\Users\\' + RegExp.escape(username), caseSensitive: false),
        r'\Users\<USER>',
      );
      out = out.replaceAll(
        RegExp(r'/Users/' + RegExp.escape(username)),
        '/Users/<USER>',
      );
      out = out.replaceAll(
        RegExp(r'/home/' + RegExp.escape(username)),
        '/home/<USER>',
      );
    }
    out = out.replaceAll(
      RegExp(r'[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}'),
      '<EMAIL>',
    );
    return out;
  }

  static String? _bootstrapBaseDir() {
    // coverage:ignore-start
    if (Platform.isWindows) {
      final local = Platform.environment['LOCALAPPDATA'];
      if (local != null && local.isNotEmpty) {
        return p.join(local, 'CopyPaste');
      }
      final profile = Platform.environment['USERPROFILE'];
      if (profile != null && profile.isNotEmpty) {
        return p.join(profile, 'AppData', 'Local', 'CopyPaste');
      }
      return p.join(Directory.systemTemp.path, 'CopyPaste');
    }
    // coverage:ignore-end
    final home = Platform.environment['HOME'];
    if (home != null && home.isNotEmpty) {
      if (Platform.isMacOS) {
        return p.join(
          home,
          'Library',
          'Application Support',
          'CopyPaste',
        ); // coverage:ignore-line
      }
      return p.join(home, '.local', 'share', 'CopyPaste');
    }
    return p.join(
      Directory.systemTemp.path,
      'CopyPaste',
    ); // coverage:ignore-line
  }
}
