import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

import 'package:core/core.dart';

void main() {
  late Directory tempDir;
  late StorageConfig config;

  setUp(() async {
    tempDir = Directory.systemTemp.createTempSync('storage_test_');
    config = await StorageConfig.create(baseDir: tempDir.path);
  });

  tearDown(() => tempDir.deleteSync(recursive: true));

  group('StorageConfig', () {
    test('paths are derived from baseDir', () {
      expect(config.baseDir, equals(tempDir.path));
      expect(config.databasePath, equals(p.join(tempDir.path, 'clipboard.db')));
      expect(config.imagesPath, equals(p.join(tempDir.path, 'images')));
      expect(config.configPath, equals(p.join(tempDir.path, 'config')));
    });

    test('ensureDirectories creates all required directories', () async {
      await config.ensureDirectories();
      expect(Directory(config.imagesPath).existsSync(), isTrue);
      expect(Directory(config.configPath).existsSync(), isTrue);
    });

    test('isFirstRun is true before markAsInitialized', () {
      expect(config.isFirstRun, isTrue);
    });

    test('markAsInitialized makes isFirstRun false', () {
      config.markAsInitialized();
      expect(config.isFirstRun, isFalse);
    });

    test('cleanOrphanImages removes unlisted image files', () async {
      await config.ensureDirectories();
      final keep = File(p.join(config.imagesPath, 'keep.png'))
        ..writeAsBytesSync([1, 2, 3]);
      final remove = File(p.join(config.imagesPath, 'remove.png'))
        ..writeAsBytesSync([4, 5, 6]);

      config.cleanOrphanImages([keep.path]);

      expect(keep.existsSync(), isTrue);
      expect(remove.existsSync(), isFalse);
    });

    test('cleanOrphanImages does not throw when directory is missing', () {
      expect(() => config.cleanOrphanImages([]), returnsNormally);
    });
  });
}
