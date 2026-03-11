import 'dart:convert';
import 'dart:io';

import 'package:archive/archive.dart';
import 'package:path/path.dart' as p;

import '../config/storage_config.dart';
import 'app_logger.dart';

class BackupManifest {
  const BackupManifest({
    required this.version,
    required this.appVersion,
    required this.createdAtUtc,
    required this.itemCount,
    required this.imageCount,
    required this.hasPinnedItems,
    required this.machineName,
  });

  factory BackupManifest.fromJson(Map<String, dynamic> json) => BackupManifest(
    version: json['version'] as int? ?? 1,
    appVersion: json['appVersion'] as String? ?? '',
    createdAtUtc:
        DateTime.tryParse(json['createdAtUtc'] as String? ?? '') ??
        DateTime.now().toUtc(),
    itemCount: json['itemCount'] as int? ?? 0,
    imageCount: json['imageCount'] as int? ?? 0,
    hasPinnedItems: json['hasPinnedItems'] as bool? ?? false,
    machineName: json['machineName'] as String? ?? '',
  );

  static const int currentVersion = 1;

  final int version;
  final String appVersion;
  final DateTime createdAtUtc;
  final int itemCount;
  final int imageCount;
  final bool hasPinnedItems;
  final String machineName;

  Map<String, dynamic> toJson() => {
    'version': version,
    'appVersion': appVersion,
    'createdAtUtc': createdAtUtc.toIso8601String(),
    'itemCount': itemCount,
    'imageCount': imageCount,
    'hasPinnedItems': hasPinnedItems,
    'machineName': machineName,
  };
}

class BackupService {
  BackupService._();

  static Future<BackupManifest> createBackup(
    String outputPath,
    StorageConfig storage,
    String appVersion, {
    int itemCount = 0,
    bool hasPinnedItems = false,
    Future<void> Function()? walCheckpoint,
  }) async {
    if (walCheckpoint != null) {
      await walCheckpoint();
    }

    final archive = Archive();
    var imageCount = 0;

    final dbFile = File(storage.databasePath);
    if (dbFile.existsSync()) {
      archive.addFile(
        ArchiveFile(
          'clipboard.db',
          dbFile.lengthSync(),
          dbFile.readAsBytesSync(),
        ),
      );
    }

    final imagesDir = Directory(storage.imagesPath);
    if (imagesDir.existsSync()) {
      for (final file in imagesDir.listSync().whereType<File>()) {
        archive.addFile(
          ArchiveFile(
            'images/${file.uri.pathSegments.last}',
            file.lengthSync(),
            file.readAsBytesSync(),
          ),
        );
        imageCount++;
      }
    }

    final configDir = Directory(storage.configPath);
    if (configDir.existsSync()) {
      for (final file in configDir.listSync().whereType<File>()) {
        archive.addFile(
          ArchiveFile(
            'config/${file.uri.pathSegments.last}',
            file.lengthSync(),
            file.readAsBytesSync(),
          ),
        );
      }
    }

    final manifest = BackupManifest(
      version: BackupManifest.currentVersion,
      appVersion: appVersion,
      createdAtUtc: DateTime.now().toUtc(),
      itemCount: itemCount,
      imageCount: imageCount,
      hasPinnedItems: hasPinnedItems,
      machineName: _hostName(),
    );

    final manifestBytes = utf8.encode(
      const JsonEncoder.withIndent('  ').convert(manifest.toJson()),
    );
    archive.addFile(
      ArchiveFile('manifest.json', manifestBytes.length, manifestBytes),
    );

    final zipData = ZipEncoder().encode(archive);

    final tempFile = File(
      '${Directory.systemTemp.path}/copypaste_backup_${DateTime.now().millisecondsSinceEpoch}.zip',
    );
    await tempFile.writeAsBytes(zipData);
    try {
      await tempFile.copy(outputPath);
    } finally {
      if (tempFile.existsSync()) tempFile.deleteSync();
    }

    return manifest;
  }

  static Future<BackupManifest?> restoreBackup(
    String backupPath,
    StorageConfig storage,
  ) async {
    final backupFile = File(backupPath);
    if (!backupFile.existsSync()) return null;

    String? snapshotDir;

    try {
      final archive = ZipDecoder().decodeBytes(backupFile.readAsBytesSync());

      final manifestEntry = archive.findFile('manifest.json');
      if (manifestEntry == null) return null;

      final manifestJson =
          jsonDecode(utf8.decode(manifestEntry.content as List<int>))
              as Map<String, dynamic>;

      final manifest = BackupManifest.fromJson(manifestJson);
      if (manifest.version > BackupManifest.currentVersion) return null;

      snapshotDir = await _createPreRestoreSnapshot(storage);

      _deleteWalFiles(storage.databasePath);

      await storage.ensureDirectories();

      for (final file in archive) {
        if (file.isFile && file.name != 'manifest.json') {
          if (file.name.contains('..')) continue;
          final outPath = p.normalize('${storage.baseDir}/${file.name}');
          if (!outPath.startsWith(storage.baseDir)) continue;
          final outFile = File(outPath);
          await outFile.create(recursive: true);
          await outFile.writeAsBytes(file.content as List<int>);
        }
      }

      _cleanupSnapshot(snapshotDir);
      return manifest;
    } catch (e) {
      AppLogger.error('restoreBackup failed: $e');
      if (snapshotDir != null) {
        await _rollbackFromSnapshot(snapshotDir, storage);
      }
      return null;
    }
  }

  static Future<BackupManifest?> validateBackup(String backupPath) async {
    final backupFile = File(backupPath);
    if (!backupFile.existsSync()) return null;

    try {
      final archive = ZipDecoder().decodeBytes(backupFile.readAsBytesSync());

      final manifestEntry = archive.findFile('manifest.json');
      if (manifestEntry == null) return null;

      final manifestJson =
          jsonDecode(utf8.decode(manifestEntry.content as List<int>))
              as Map<String, dynamic>;

      final manifest = BackupManifest.fromJson(manifestJson);
      if (manifest.version > BackupManifest.currentVersion) return null;

      return manifest;
    } catch (e) {
      AppLogger.error('validateBackup failed: $e');
      return null;
    }
  }

  static void _deleteWalFiles(String dbPath) {
    try {
      final walFile = File('$dbPath-wal');
      final shmFile = File('$dbPath-shm');
      if (walFile.existsSync()) walFile.deleteSync();
      if (shmFile.existsSync()) shmFile.deleteSync();
    } catch (e) {
      AppLogger.error('deleteWalFiles failed: $e');
    }
  }

  static Future<String> _createPreRestoreSnapshot(StorageConfig storage) async {
    final timestamp = DateTime.now().toUtc().millisecondsSinceEpoch;
    final snapshotDir = p.join(storage.baseDir, '.pre-restore-$timestamp');
    final dir = Directory(snapshotDir);
    await dir.create(recursive: true);

    final dbFile = File(storage.databasePath);
    if (dbFile.existsSync()) {
      await dbFile.copy(p.join(snapshotDir, 'clipboard.db'));
    }

    final imagesDir = Directory(storage.imagesPath);
    if (imagesDir.existsSync()) {
      final snapImagesDir = Directory(p.join(snapshotDir, 'images'));
      await snapImagesDir.create();
      for (final file in imagesDir.listSync().whereType<File>()) {
        await file.copy(p.join(snapImagesDir.path, p.basename(file.path)));
      }
    }

    final configDir = Directory(storage.configPath);
    if (configDir.existsSync()) {
      final snapConfigDir = Directory(p.join(snapshotDir, 'config'));
      await snapConfigDir.create();
      for (final file in configDir.listSync().whereType<File>()) {
        await file.copy(p.join(snapConfigDir.path, p.basename(file.path)));
      }
    }

    return snapshotDir;
  }

  static Future<void> _rollbackFromSnapshot(
    String snapshotDir,
    StorageConfig storage,
  ) async {
    try {
      final snapDb = File(p.join(snapshotDir, 'clipboard.db'));
      if (snapDb.existsSync()) {
        await snapDb.copy(storage.databasePath);
      }

      final snapImages = Directory(p.join(snapshotDir, 'images'));
      if (snapImages.existsSync()) {
        for (final file in snapImages.listSync().whereType<File>()) {
          await file.copy(p.join(storage.imagesPath, p.basename(file.path)));
        }
      }

      final snapConfig = Directory(p.join(snapshotDir, 'config'));
      if (snapConfig.existsSync()) {
        for (final file in snapConfig.listSync().whereType<File>()) {
          await file.copy(p.join(storage.configPath, p.basename(file.path)));
        }
      }

      _cleanupSnapshot(snapshotDir);
    } catch (e) {
      AppLogger.error('rollbackFromSnapshot failed: $e');
    }
  }

  static void _cleanupSnapshot(String snapshotDir) {
    try {
      Directory(snapshotDir).deleteSync(recursive: true);
    } catch (e) {
      AppLogger.error('cleanupSnapshot failed: $e');
    }
  }

  static String _hostName() {
    try {
      return Platform.localHostname;
    } catch (e) {
      AppLogger.error('hostName failed: $e');
      return 'unknown';
    }
  }
}
