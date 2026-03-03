import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';

class StorageConfig {
  StorageConfig._({required this.baseDir})
      : databasePath = p.join(baseDir, 'clipboard.db'),
        imagesPath = p.join(baseDir, 'images'),
        thumbnailsPath = p.join(baseDir, 'thumbs'),
        configPath = p.join(baseDir, 'config');

  final String baseDir;
  final String databasePath;
  final String imagesPath;
  final String thumbnailsPath;
  final String configPath;

  String get _initFlagPath => p.join(baseDir, '.initialized');

  static Future<StorageConfig> create({String? baseDir}) async {
    final base = baseDir ??
        p.join((await getApplicationSupportDirectory()).path, 'CopyPaste');
    return StorageConfig._(baseDir: base);
  }

  Future<void> ensureDirectories() async {
    for (final dir in [baseDir, imagesPath, thumbnailsPath, configPath]) {
      await Directory(dir).create(recursive: true);
    }
  }

  bool get isFirstRun => !File(_initFlagPath).existsSync();

  void markAsInitialized() {
    try {
      File(_initFlagPath)
        ..createSync(recursive: true)
        ..writeAsStringSync(DateTime.now().toUtc().toIso8601String());
    } catch (_) {}
  }

  void cleanOrphanImages(List<String> validImagePaths) {
    final validSet = validImagePaths.toSet();
    _cleanDirectory(imagesPath, validSet);
    final validThumbs = validImagePaths.map((path) {
      final name = p.basenameWithoutExtension(path);
      return p.join(thumbnailsPath, '${name}_t.png');
    }).toSet();
    _cleanDirectory(thumbnailsPath, validThumbs);
  }

  void _cleanDirectory(String dirPath, Set<String> validFiles) {
    final dir = Directory(dirPath);
    if (!dir.existsSync()) return;
    for (final file in dir.listSync().whereType<File>()) {
      if (!validFiles.contains(file.path)) {
        try {
          file.deleteSync();
        } catch (_) {}
      }
    }
  }
}
