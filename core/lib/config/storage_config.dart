import 'dart:io';

import '../services/app_logger.dart';

import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';

class StorageConfig {
  StorageConfig._({required this.baseDir})
    : databasePath = p.join(baseDir, 'clipboard.db'),
      imagesPath = p.join(baseDir, 'images'),
      configPath = p.join(baseDir, 'config'),
      logsPath = p.join(baseDir, 'logs');

  final String baseDir;
  final String databasePath;
  final String imagesPath;
  final String configPath;
  final String logsPath;

  String get _initFlagPath => p.join(baseDir, '.initialized');

  static Future<StorageConfig> create({
    String? baseDir,
    String? Function()? windowsLocalAppDataResolver,
  }) async {
    final String base;
    if (baseDir != null) {
      base = baseDir;
    } else if (Platform.isWindows) {
      final resolved =
          windowsLocalAppDataResolver?.call() ??
          Platform.environment['LOCALAPPDATA'];
      base = resolved != null
          ? p.join(resolved, 'CopyPaste')
          : p.join((await getApplicationSupportDirectory()).path, 'CopyPaste');
    } else {
      base = p.join((await getApplicationSupportDirectory()).path, 'CopyPaste');
    }
    return StorageConfig._(baseDir: base);
  }

  Future<void> ensureDirectories() async {
    for (final dir in [baseDir, imagesPath, configPath, logsPath]) {
      await Directory(dir).create(recursive: true);
    }
  }

  bool get isFirstRun => !File(_initFlagPath).existsSync();

  void markAsInitialized() {
    try {
      File(_initFlagPath)
        ..createSync(recursive: true)
        ..writeAsStringSync(DateTime.now().toUtc().toIso8601String());
    } catch (e) {
      AppLogger.error('Failed to mark as initialized: $e');
    }
  }

  void clearInitialized() {
    try {
      final f = File(_initFlagPath);
      if (f.existsSync()) f.deleteSync();
    } catch (e) {
      AppLogger.error('Failed to clear initialized flag: $e');
    }
  }

  void cleanOrphanImages(List<String> validImagePaths) {
    _cleanDirectory(imagesPath, validImagePaths.toSet());
  }

  void _cleanDirectory(String dirPath, Set<String> validFiles) {
    final dir = Directory(dirPath);
    if (!dir.existsSync()) return;
    for (final file in dir.listSync().whereType<File>()) {
      if (!validFiles.contains(file.path)) {
        try {
          file.deleteSync();
        } catch (e) {
          AppLogger.error('Failed to delete orphan file: $e');
        }
      }
    }
  }
}
