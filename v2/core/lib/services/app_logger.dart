import 'dart:io';

import 'package:path/path.dart' as p;

class AppLogger {
  AppLogger._();

  static String? _logDirectory;
  static String? _logFilePath;
  static bool _isInitialized = false;
  static bool isEnabled = true;

  static const int _maxLogAgeDays = 7;
  static const int _maxLogSizeBytes = 10 * 1024 * 1024;

  static String? get logFilePath => _logFilePath;
  static String? get logDirectory => _logDirectory;

  static void initialize(String logsPath) {
    if (_isInitialized) return;
    try {
      _logDirectory = logsPath;
      final now = DateTime.now();
      final dateStr =
          '${now.year}-${_pad(now.month)}-${_pad(now.day)}';
      _logFilePath = p.join(logsPath, 'copypaste_$dateStr.log');
      Directory(logsPath).createSync(recursive: true);
      _cleanOldLogs();
      _isInitialized = true;
      info('Logger initialized');
    } catch (_) {
      isEnabled = false;
    }
  }

  static void info(String message) => _log('INFO', message);

  static void warn(String message) => _log('WARN', message);

  static void error(String message) => _log('ERROR', message);

  static void exception(Object? error, [StackTrace? stackTrace, String context = '']) {
    if (!isEnabled || !_isInitialized || error == null) return;
    final sb = StringBuffer();
    if (context.isNotEmpty) sb.write('$context - ');
    sb.write(error.toString());
    if (stackTrace != null) sb.write('\n$stackTrace');
    _log('ERROR', sb.toString());
  }

  static void _log(String level, String message) {
    if (!isEnabled || !_isInitialized || _logFilePath == null) return;
    try {
      final now = DateTime.now();
      final timestamp =
          '${_pad(now.hour)}:${_pad(now.minute)}:${_pad(now.second)}.${_pad3(now.millisecond)}';
      final entry = '[$timestamp] [$level] $message\n';
      final file = File(_logFilePath!);
      if (file.existsSync() && file.lengthSync() > _maxLogSizeBytes) {
        _rotateLog();
      }
      file.writeAsStringSync(entry, mode: FileMode.append);
    } catch (_) {}
  }

  static void _rotateLog() {
    if (_logFilePath == null || _logDirectory == null) return;
    try {
      final now = DateTime.now();
      final rotatedName =
          'copypaste_${now.year}-${_pad(now.month)}-${_pad(now.day)}_${_pad(now.hour)}${_pad(now.minute)}${_pad(now.second)}.log';
      File(_logFilePath!).renameSync(p.join(_logDirectory!, rotatedName));
    } catch (_) {
      try {
        File(_logFilePath!).deleteSync();
      } catch (_) {}
    }
  }

  static void _cleanOldLogs() {
    if (_logDirectory == null) return;
    try {
      final cutoff = DateTime.now().subtract(
        const Duration(days: _maxLogAgeDays),
      );
      final dir = Directory(_logDirectory!);
      if (!dir.existsSync()) return;
      for (final file in dir.listSync().whereType<File>()) {
        if (p.basename(file.path).startsWith('copypaste_') &&
            file.path.endsWith('.log')) {
          if (file.lastModifiedSync().isBefore(cutoff)) {
            file.deleteSync();
          }
        }
      }
    } catch (_) {}
  }

  static String _pad(int n) => n.toString().padLeft(2, '0');
  static String _pad3(int n) => n.toString().padLeft(3, '0');
}
