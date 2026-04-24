import 'dart:async';
import 'dart:io';

import 'package:path/path.dart' as p;

import '../models/clipboard_content_type.dart';
import '../models/clipboard_item.dart';
import '../repository/i_clipboard_repository.dart';
import 'app_logger.dart';
import 'thumbnail_service.dart';

/// Reasons a [ThumbnailQueue] job can be enqueued. Used only for logging.
enum ThumbnailJobReason { freshItem, staleRegeneration, manualRefresh }

class _ThumbJob {
  _ThumbJob(this.itemId, this.reason);

  final String itemId;
  final ThumbnailJobReason reason;
}

/// Serial queue that generates 256-px PNG thumbnails for image items via
/// [ThumbnailService] and persists the result back to the repository.
///
/// Responsibilities:
///   - Single in-flight job at a time (no CPU/disk thrash).
///   - Race-safe persistence: the item is re-fetched right before update,
///     and if it has been deleted in the meantime the generated thumb file
///     is removed and the update is skipped.
///   - mtime-based staleness check: [enqueueIfStale] re-encodes when the
///     source file's `mtime` no longer matches the recorded
///     `sourceModifiedAt`.
///   - Best-effort emit of the updated item via [onItemUpdated] so the UI
///     can rebuild the affected card.
///
/// All file writes happen inside [ThumbnailService.imagesPath]. Cleanup of
/// orphan thumbs is owned by `CleanupService` (see `getThumbPaths()`).
class ThumbnailQueue {
  ThumbnailQueue({
    required IClipboardRepository repository,
    required ThumbnailService service,
    this.onItemUpdated,
  }) : _repository = repository,
       _service = service;

  final IClipboardRepository _repository;
  final ThumbnailService _service;

  /// Called on the main isolate after a job successfully writes a thumb and
  /// updates the repository row. Never called for skipped or failed jobs.
  final void Function(ClipboardItem item)? onItemUpdated;

  final _queue = <_ThumbJob>[];
  final _enqueuedIds = <String>{};
  bool _processing = false;
  bool _disposed = false;

  /// Visible for tests: number of jobs currently waiting (excludes the
  /// one being processed, if any).
  int get pendingCount => _queue.length;

  /// Enqueues a thumbnail job for [item]. No-ops if the queue is disposed,
  /// the item type is not eligible, or the same id is already queued.
  ///
  /// Returns synchronously; the actual generation runs asynchronously.
  void enqueue(ClipboardItem item, {ThumbnailJobReason? reason}) {
    if (_disposed) return;
    if (!_isEligible(item)) return;
    if (_enqueuedIds.contains(item.id)) return;
    _enqueuedIds.add(item.id);
    _queue.add(_ThumbJob(item.id, reason ?? ThumbnailJobReason.freshItem));
    if (_queue.length > 20) {
      AppLogger.warn('[ThumbQueue] queue depth: ${_queue.length}');
    }
    _scheduleNext();
  }

  /// Enqueues a regeneration only if the source file's current `mtime`
  /// differs from `item.sourceModifiedAt`. Items without a recorded
  /// `sourceModifiedAt` are also enqueued (we have no baseline to compare).
  ///
  /// Safe to call from the UI thread; the file `stat` runs synchronously
  /// but is cheap and only invoked when the card is being resolved.
  void enqueueIfStale(ClipboardItem item) {
    if (_disposed) return;
    if (!_isEligible(item)) return;

    final sourcePath = _singleSourcePath(item);
    if (sourcePath == null) return;

    final file = File(sourcePath);
    if (!file.existsSync()) return;

    final FileStat stat;
    try {
      stat = file.statSync();
    } catch (_) {
      return;
    }

    final recorded = item.sourceModifiedAt;
    final currentUtc = stat.modified.toUtc();
    final isStale =
        recorded == null ||
        currentUtc.millisecondsSinceEpoch != recorded.millisecondsSinceEpoch;

    if (!isStale) return;

    enqueue(item, reason: ThumbnailJobReason.staleRegeneration);
  }

  bool _isEligible(ClipboardItem item) {
    if (item.type != ClipboardContentType.image) return false;
    if (item.content.isEmpty) return false;
    return _singleSourcePath(item) != null;
  }

  String? _singleSourcePath(ClipboardItem item) {
    final paths = item.content.split('\n').where((s) => s.isNotEmpty).toList();
    if (paths.length != 1) return null;
    return paths.single;
  }

  void _scheduleNext() {
    if (_processing || _queue.isEmpty || _disposed) return;
    _processing = true;
    final job = _queue.removeAt(0);
    _runJob(job).whenComplete(() {
      _enqueuedIds.remove(job.itemId);
      _processing = false;
      _scheduleNext();
    });
  }

  Future<void> _runJob(_ThumbJob job) async {
    if (_disposed) return;

    // Re-fetch right before generation: the item may have been deleted or
    // mutated since enqueue. Cheaper to re-read than to encode and discard.
    final fresh = await _repository.getById(job.itemId);
    if (fresh == null) return;
    if (!_isEligible(fresh)) return;

    final result = await _safeGenerate(fresh);
    if (result == null) return;

    if (_disposed) {
      _safeDelete(result.thumbPath);
      return;
    }

    // Race window: the user may have deleted the item while we were
    // encoding. If gone, drop the file we just produced.
    final stillThere = await _repository.getById(job.itemId);
    if (stillThere == null) {
      _safeDelete(result.thumbPath);
      return;
    }

    final updated = stillThere.copyWith(
      thumbPath: result.thumbPath,
      sourceModifiedAt: result.sourceModifiedAt,
    );
    try {
      await _repository.update(updated);
    } catch (e, s) {
      AppLogger.error('[ThumbQueue] update failed for ${job.itemId}: $e\n$s');
      _safeDelete(result.thumbPath);
      return;
    }

    if (!_disposed) onItemUpdated?.call(updated);
  }

  Future<ThumbnailResult?> _safeGenerate(ClipboardItem item) async {
    try {
      return await _service.generateForItem(item);
    } catch (e, s) {
      AppLogger.warn('[ThumbQueue] generate failed for ${item.id}: $e\n$s');
      return null;
    }
  }

  void _safeDelete(String path) {
    try {
      final base = p.canonicalize(_service.imagesPath);
      final target = p.canonicalize(path);
      final sep = base.endsWith(p.separator) ? base : '$base${p.separator}';
      if (!target.startsWith(sep)) return;
      final file = File(target);
      if (file.existsSync()) file.deleteSync();
    } catch (e) {
      AppLogger.warn('[ThumbQueue] _safeDelete failed for "$path": $e');
    }
  }

  /// Cancels pending jobs and waits up to 1500 ms for the active job to
  /// finish. After this call the queue refuses new jobs.
  Future<void> dispose() async {
    if (_disposed) return;
    _disposed = true;
    _queue.clear();
    _enqueuedIds.clear();
    if (_processing) {
      await Future<void>.delayed(const Duration(milliseconds: 1500));
    }
  }
}
