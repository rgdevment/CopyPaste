import 'dart:io';

import 'package:archive/archive.dart';
import 'package:path/path.dart' as p;

import '../config/storage_config.dart';
import 'app_logger.dart';

class SupportService {
  SupportService._();

  /// Exports all log files into a zip archive saved at [savePath].
  ///
  /// The zip includes:
  /// - All `.log` files from [StorageConfig.logsPath].
  /// - A `device_info.txt` with basic platform and version details.
  ///
  /// Returns the number of log files included, or throws on failure.
  static Future<int> exportLogs(
    StorageConfig storage,
    String appVersion,
    String savePath,
  ) async {
    final logsDir = Directory(storage.logsPath);
    final archive = Archive();

    final logFiles = logsDir.existsSync()
        ? logsDir
              .listSync()
              .whereType<File>()
              .where((f) => f.path.endsWith('.log'))
              .toList()
        : <File>[];

    for (final file in logFiles) {
      final bytes = await file.readAsBytes();
      archive.addFile(ArchiveFile(p.basename(file.path), bytes.length, bytes));
    }

    // Add device info so the report is self-contained
    final info = _buildDeviceInfo(appVersion);
    final infoBytes = info.codeUnits;
    archive.addFile(
      ArchiveFile('device_info.txt', infoBytes.length, infoBytes),
    );

    final zipData = ZipEncoder().encode(archive);

    await File(savePath).writeAsBytes(zipData);
    AppLogger.info('Logs exported to $savePath (${logFiles.length} files)');
    return logFiles.length;
  }

  /// Opens the logs directory in the system file browser.
  static Future<void> openLogsFolder(StorageConfig storage) async {
    final logsDir = Directory(storage.logsPath);
    if (!logsDir.existsSync()) await logsDir.create(recursive: true);

    if (Platform.isWindows) {
      await Process.run('explorer', [logsDir.path]);
    } else if (Platform.isMacOS) {
      await Process.run('open', [logsDir.path]);
    } else if (Platform.isLinux) {
      await Process.run('xdg-open', [logsDir.path]);
    }
  }

  static String _buildDeviceInfo(String appVersion) {
    final lines = [
      'CopyPaste v$appVersion',
      'Generated: ${DateTime.now().toUtc().toIso8601String()}',
      '',
      'Platform : ${Platform.operatingSystem}',
      'OS       : ${Platform.operatingSystemVersion}',
      'Locale   : ${Platform.localeName}',
      'Dart     : ${Platform.version}',
    ];
    return lines.join('\n');
  }
}
