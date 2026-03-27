import 'dart:io';

import 'package:archive/archive.dart';
import 'package:flutter/foundation.dart' show visibleForTesting;
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
    AppLogger.info('exportLogs: starting — savePath=$savePath');
    final logsDir = Directory(storage.logsPath);
    final archive = Archive();

    final logFiles = logsDir.existsSync()
        ? logsDir
              .listSync()
              .whereType<File>()
              .where((f) => f.path.endsWith('.log'))
              .toList()
        : <File>[];

    if (logFiles.isEmpty) {
      AppLogger.warn('exportLogs: no .log files found in ${storage.logsPath}');
    }

    for (final file in logFiles) {
      try {
        final bytes = await file.readAsBytes();
        archive.addFile(
          ArchiveFile(p.basename(file.path), bytes.length, bytes),
        );
      } catch (e) {
        AppLogger.error('exportLogs: failed to read ${file.path}: $e');
      }
    }

    // Add device info so the report is self-contained
    final info = _buildDeviceInfo(appVersion);
    final infoBytes = info.codeUnits;
    archive.addFile(
      ArchiveFile('device_info.txt', infoBytes.length, infoBytes),
    );

    final zipData = ZipEncoder().encode(archive);
    if (zipData.isEmpty) {
      AppLogger.error('exportLogs: ZipEncoder returned empty data');
      throw StateError('Zip encoding produced no output');
    }

    await File(savePath).writeAsBytes(zipData);
    AppLogger.info(
      'exportLogs: done — ${logFiles.length} log file(s) → $savePath',
    );
    return logFiles.length;
  }

  /// Reveals [filePath] in the system file browser (Finder, Explorer, etc.).
  static Future<void> revealFile(String filePath) async {
    AppLogger.info('revealFile: $filePath');
    try {
      if (Platform.isWindows) {
        await Process.run('explorer', ['/select,', filePath]);
      } else if (Platform.isMacOS) {
        await Process.run('open', ['-R', filePath]);
      } else if (Platform.isLinux) {
        await Process.run('xdg-open', [File(filePath).parent.path]);
      }
    } catch (e, s) {
      AppLogger.exception(e, s, 'revealFile');
    }
  }

  /// Opens the logs directory in the system file browser.
  static Future<void> openLogsFolder(StorageConfig storage) async {
    final logsDir = Directory(storage.logsPath);
    if (!logsDir.existsSync()) {
      AppLogger.info('openLogsFolder: logs dir missing, creating it');
      await logsDir.create(recursive: true);
    }

    AppLogger.info('openLogsFolder: opening ${logsDir.path}');
    try {
      if (Platform.isWindows) {
        // Process.run('explorer', path) silently fails in MSIX packages because
        // Windows routes the open request via DDE to the existing shell process,
        // and the AppContainer blocks cross-process DDE. Using cmd's start
        // command calls ShellExecuteEx instead, which works correctly in MSIX.
        await Process.run('cmd', ['/c', 'start', '', logsDir.path]);
      } else if (Platform.isMacOS) {
        await Process.run('open', [logsDir.path]);
      } else if (Platform.isLinux) {
        await Process.run('xdg-open', [logsDir.path]);
      }
    } catch (e, s) {
      AppLogger.exception(e, s, 'openLogsFolder');
      rethrow;
    }
  }

  static String _buildDeviceInfo(String appVersion) {
    final osVersion = Platform.isWindows
        ? correctWindowsVersion(Platform.operatingSystemVersion)
        : Platform.operatingSystemVersion;
    final lines = [
      'CopyPaste v$appVersion',
      'Generated: ${DateTime.now().toUtc().toIso8601String()}',
      '',
      'Platform : ${Platform.operatingSystem}',
      'OS       : $osVersion',
      'Locale   : ${Platform.localeName}',
      'Dart     : ${Platform.version}',
    ];
    return lines.join('\n');
  }

  // Dart/Flutter always reports "Windows 10" even on Windows 11 due to Win32
  // backwards-compat shim. Windows 11 starts at build 22000.
  @visibleForTesting
  static String correctWindowsVersion(String raw) {
    if (!raw.contains('Windows 10')) return raw;
    final match = RegExp(r'Build (\d+)').firstMatch(raw);
    if (match == null) return raw;
    final build = int.tryParse(match.group(1) ?? '') ?? 0;
    return build >= 22000 ? raw.replaceFirst('Windows 10', 'Windows 11') : raw;
  }
}
