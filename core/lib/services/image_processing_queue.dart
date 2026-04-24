import 'dart:async';
import 'dart:io';
import 'dart:isolate';
import 'dart:typed_data';

import 'package:path/path.dart' as p;

import '../models/clipboard_item.dart';
import '../repository/i_clipboard_repository.dart';
import 'app_logger.dart';
import 'image_processor.dart';

/// A job submitted to [ImageProcessingQueue].
class _ImageJob {
  _ImageJob({
    required this.item,
    required this.imageBytes,
    required this.imagesPath,
  });

  final ClipboardItem item;
  final Uint8List imageBytes;
  final String imagesPath;
}

/// Serial queue for image processing jobs.
///
/// Processes one job at a time to avoid saturating CPU and disk.
/// Each job runs [ImageProcessor.processSync] in a dedicated isolate
/// with a configurable [jobTimeout].
///
/// If a job exceeds [jobTimeout], the isolate is killed. The BMP fallback
/// written before launching the job is preserved so the item remains
/// pasteable. The event is logged and the queue moves on.
///
/// Call [dispose] on app shutdown to cancel pending work and release resources.
class ImageProcessingQueue {
  ImageProcessingQueue({
    required IClipboardRepository repository,
    this.jobTimeout = const Duration(seconds: 10),
    this.onItemUpdated,
  }) : _repository = repository;

  final IClipboardRepository _repository;
  final Duration jobTimeout;

  /// Called on the main isolate after a job completes and the repository
  /// entry has been updated with the final PNG path and dimensions.
  final void Function(ClipboardItem item)? onItemUpdated;

  final _queue = <_ImageJob>[];
  bool _processing = false;
  bool _disposed = false;

  /// Enqueues an image processing job. Returns immediately; the job runs when
  /// the queue reaches it. Silently drops jobs after [dispose] is called.
  void enqueue({
    required ClipboardItem item,
    required List<int> imageBytes,
    required String imagesPath,
  }) {
    if (_disposed) return;
    final bytes = imageBytes is Uint8List
        ? imageBytes
        : Uint8List.fromList(imageBytes);
    _queue.add(
      _ImageJob(item: item, imageBytes: bytes, imagesPath: imagesPath),
    );
    if (_queue.length > 10) {
      AppLogger.warn('[ImageQueue] queue depth: ${_queue.length}');
    }
    _scheduleNext();
  }

  void _scheduleNext() {
    if (_processing || _queue.isEmpty || _disposed) return;
    _processing = true;
    final job = _queue.removeAt(0);
    _runJob(job).whenComplete(() {
      _processing = false;
      _scheduleNext();
    });
  }

  Future<void> _runJob(_ImageJob job) async {
    final resultPort = ReceivePort();
    Isolate? isolate;

    try {
      final resultCompleter = Completer<ImageProcessResult?>();

      resultPort.listen((msg) {
        if (!resultCompleter.isCompleted) {
          resultCompleter.complete(msg is ImageProcessResult ? msg : null);
        }
      });

      isolate = await Isolate.spawn(
        _isolateWorker,
        _IsolateParams(
          imageBytes: job.imageBytes,
          id: job.item.id,
          imagesDir: job.imagesPath,
          resultPort: resultPort.sendPort,
        ),
        debugName: 'ImageWorker:${job.item.id}',
        errorsAreFatal: false,
      );

      ImageProcessResult? result;
      try {
        result = await resultCompleter.future.timeout(jobTimeout);
      } on TimeoutException {
        AppLogger.warn(
          '[ImageQueue] timeout (${jobTimeout.inSeconds}s) for ${job.item.id}'
          ' — keeping BMP fallback',
        );
        return; // BMP on disk stays; item remains pasteable.
      }

      if (result == null) {
        AppLogger.warn(
          '[ImageQueue] null result for ${job.item.id}'
          ' (unsupported format) — keeping BMP fallback',
        );
        return;
      }

      // Remove BMP fallback now that the final PNG exists.
      final bmpPath = p.join(job.imagesPath, '${job.item.id}.bmp');
      _deleteOwned(bmpPath, job.imagesPath);

      if (_disposed) return;

      final meta =
          '{"width":${result.width},"height":${result.height},'
          '"size":${result.fileSize}}';
      final updated = job.item.copyWith(
        content: result.imagePath,
        metadata: meta,
      );
      await _repository.update(updated);
      if (!_disposed) onItemUpdated?.call(updated);
    } catch (e, s) {
      AppLogger.error('[ImageQueue] job failed for ${job.item.id}: $e\n$s');
    } finally {
      resultPort.close();
      isolate?.kill(priority: Isolate.beforeNextEvent);
    }
  }

  /// Cancels pending jobs and waits up to 1500 ms for the active job to finish.
  /// After this call the queue refuses new jobs.
  Future<void> dispose() async {
    if (_disposed) return;
    _disposed = true;
    _queue.clear();
    if (_processing) {
      await Future<void>.delayed(const Duration(milliseconds: 1500));
    }
  }

  /// Deletes a file only if it is canonically inside [imagesDir].
  static void _deleteOwned(String path, String imagesDir) {
    try {
      final base = p.canonicalize(imagesDir);
      final target = p.canonicalize(path);
      final sep = base.endsWith(p.separator) ? base : '$base${p.separator}';
      if (!target.startsWith(sep)) return;
      final f = File(target);
      if (f.existsSync()) f.deleteSync();
    } catch (e) {
      AppLogger.warn('[ImageQueue] _deleteOwned failed for "$path": $e');
    }
  }
}

// ---------------------------------------------------------------------------
// Isolate worker — runs in a separate isolate, no access to singletons.
// ---------------------------------------------------------------------------

class _IsolateParams {
  _IsolateParams({
    required this.imageBytes,
    required this.id,
    required this.imagesDir,
    required this.resultPort,
  });

  final Uint8List imageBytes;
  final String id;
  final String imagesDir;
  final SendPort resultPort;
}

void _isolateWorker(_IsolateParams params) {
  final result = ImageProcessor.processSync(
    imageBytes: params.imageBytes,
    id: params.id,
    imagesDir: params.imagesDir,
  );
  params.resultPort.send(result);
}
