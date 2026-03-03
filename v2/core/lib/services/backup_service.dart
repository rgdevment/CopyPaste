import 'dart:convert';
import 'dart:io';

import 'package:archive/archive.dart';

import '../config/storage_config.dart';

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

  factory BackupManifest.fromJson(Map<String, dynamic> json) =>
      BackupManifest(
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
    String appVersion,
  ) async {
    final archive = Archive();
    var imageCount = 0;

    final dbFile = File(storage.databasePath);
    if (dbFile.existsSync()) {
      archive.addFile(ArchiveFile(
        'clipboard.db',
        dbFile.lengthSync(),
        dbFile.readAsBytesSync(),
      ));
    }

    final imagesDir = Directory(storage.imagesPath);
    if (imagesDir.existsSync()) {
      for (final file in imagesDir.listSync().whereType<File>()) {
        archive.addFile(ArchiveFile(
          'images/${file.uri.pathSegments.last}',
          file.lengthSync(),
          file.readAsBytesSync(),
        ));
        imageCount++;
      }
    }

    final manifest = BackupManifest(
      version: BackupManifest.currentVersion,
      appVersion: appVersion,
      createdAtUtc: DateTime.now().toUtc(),
      itemCount: 0,
      imageCount: imageCount,
      hasPinnedItems: false,
      machineName: _hostName(),
    );

    final manifestBytes = utf8.encode(
      const JsonEncoder.withIndent('  ').convert(manifest.toJson()),
    );
    archive.addFile(
      ArchiveFile('manifest.json', manifestBytes.length, manifestBytes),
    );

    final zipData = ZipEncoder().encode(archive);
    if (zipData != null) {
      await File(outputPath).writeAsBytes(zipData);
    }

    return manifest;
  }

  static Future<BackupManifest?> restoreBackup(
    String backupPath,
    StorageConfig storage,
  ) async {
    final backupFile = File(backupPath);
    if (!backupFile.existsSync()) return null;

    try {
      final archive =
          ZipDecoder().decodeBytes(backupFile.readAsBytesSync());

      final manifestEntry = archive.findFile('manifest.json');
      if (manifestEntry == null) return null;

      final manifestJson = jsonDecode(
        utf8.decode(manifestEntry.content as List<int>),
      ) as Map<String, dynamic>;

      final manifest = BackupManifest.fromJson(manifestJson);
      if (manifest.version > BackupManifest.currentVersion) return null;

      await storage.ensureDirectories();

      for (final file in archive) {
        if (file.isFile && file.name != 'manifest.json') {
          final outPath = '${storage.baseDir}/${file.name}';
          final outFile = File(outPath);
          await outFile.create(recursive: true);
          await outFile.writeAsBytes(file.content as List<int>);
        }
      }

      return manifest;
    } catch (_) {
      return null;
    }
  }

  static String _hostName() {
    try {
      return Platform.localHostname;
    } catch (_) {
      return 'unknown';
    }
  }
}
