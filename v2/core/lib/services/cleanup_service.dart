import 'dart:async';
import 'dart:io';

import 'package:path/path.dart' as p;

import '../config/storage_config.dart';
import '../repository/i_clipboard_repository.dart';

class CleanupService {
  CleanupService(this._repository, this._getRetentionDays, {StorageConfig? storage})
      : _storage = storage;

  static const Duration _checkInterval = Duration(hours: 18);
  static const String _lastCleanupFileName = 'last_cleanup.txt';

  final IClipboardRepository _repository;
  final int Function() _getRetentionDays;
  final StorageConfig? _storage;
  Timer? _timer;
  bool _disposed = false;

  String? _baseDirPath;

  String get _cleanupFilePath =>
      p.join(_baseDirPath ?? '', _lastCleanupFileName);

  void start(String baseDirPath) {
    _baseDirPath = baseDirPath;
    _timer = Timer.periodic(_checkInterval, (_) => runCleanupIfNeeded());
    runCleanupIfNeeded();
  }

  Future<void> runCleanupIfNeeded() async {
    if (_disposed) return;
    final retentionDays = _getRetentionDays();
    if (retentionDays <= 0) return;

    final lastCleanup = _loadLastCleanupDate();
    final now = DateTime.now().toUtc();
    if (lastCleanup.year == now.year &&
        lastCleanup.month == now.month &&
        lastCleanup.day == now.day) {
      return;
    }

    try {
      await _repository.clearOldItems(retentionDays, excludePinned: true);
      _saveLastCleanupDate(now);
      await _cleanOrphanImages();
    } catch (_) {}
  }

  DateTime _loadLastCleanupDate() {
    try {
      final file = File(_cleanupFilePath);
      if (file.existsSync()) {
        final content = file.readAsStringSync().trim();
        final parsed = DateTime.tryParse(content);
        if (parsed != null) return parsed.toUtc();
      }
    } catch (_) {}
    return DateTime.utc(2000);
  }

  void _saveLastCleanupDate(DateTime date) {
    try {
      final file = File(_cleanupFilePath);
      file.parent.createSync(recursive: true);
      file.writeAsStringSync(date.toIso8601String());
    } catch (_) {}
  }

  void dispose() {
    _disposed = true;
    _timer?.cancel();
  }

  Future<void> _cleanOrphanImages() async {
    final storage = _storage;
    if (storage == null) return;
    try {
      final validPaths = await _repository.getImagePaths();
      storage.cleanOrphanImages(validPaths);
    } catch (_) {}
  }
}
