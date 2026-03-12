import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'package:app/shell/startup_helper.dart';

String _plistPath() {
  const plistLabel = 'com.rgdevment.copypaste';
  final home = Platform.environment['HOME'] ?? '/tmp';
  return '$home/Library/LaunchAgents/$plistLabel.plist';
}

void main() {
  const plistLabel = 'com.rgdevment.copypaste';

  // Remove any plist left over from a previous test run.
  tearDown(() async {
    if (!Platform.isMacOS) return;
    final f = File(_plistPath());
    if (f.existsSync()) f.deleteSync();
  });

  group('StartupHelper – macOS', () {
    test('apply(true) creates the LaunchAgent plist', () async {
      if (!Platform.isMacOS) return;
      await StartupHelper.apply(true);
      expect(File(_plistPath()).existsSync(), isTrue);
    });

    test('plist is a valid XML plist document', () async {
      if (!Platform.isMacOS) return;
      await StartupHelper.apply(true);
      final content = File(_plistPath()).readAsStringSync();
      expect(content, contains('<?xml version="1.0"'));
      expect(content, contains('<plist version="1.0">'));
      expect(content, contains('</plist>'));
    });

    test('plist contains the correct Label key', () async {
      if (!Platform.isMacOS) return;
      await StartupHelper.apply(true);
      final content = File(_plistPath()).readAsStringSync();
      expect(content, contains('<key>Label</key>'));
      expect(content, contains('<string>$plistLabel</string>'));
    });

    test('plist contains RunAtLoad set to true', () async {
      if (!Platform.isMacOS) return;
      await StartupHelper.apply(true);
      final content = File(_plistPath()).readAsStringSync();
      expect(content, contains('<key>RunAtLoad</key>'));
      expect(content, contains('<true/>'));
    });

    test('plist contains KeepAlive set to false', () async {
      if (!Platform.isMacOS) return;
      await StartupHelper.apply(true);
      final content = File(_plistPath()).readAsStringSync();
      expect(content, contains('<key>KeepAlive</key>'));
      expect(content, contains('<false/>'));
    });

    test('plist ProgramArguments contains the executable path', () async {
      if (!Platform.isMacOS) return;
      await StartupHelper.apply(true);
      final content = File(_plistPath()).readAsStringSync();
      expect(content, contains('<key>ProgramArguments</key>'));
      expect(content, contains(Platform.resolvedExecutable));
    });

    test('apply(false) removes the LaunchAgent plist', () async {
      if (!Platform.isMacOS) return;
      await StartupHelper.apply(true);
      expect(File(_plistPath()).existsSync(), isTrue);
      await StartupHelper.apply(false);
      expect(File(_plistPath()).existsSync(), isFalse);
    });

    test('apply(false) does not throw when plist does not exist', () async {
      if (!Platform.isMacOS) return;
      final f = File(_plistPath());
      if (f.existsSync()) f.deleteSync();
      await expectLater(StartupHelper.apply(false), completes);
    });

    test('apply(true) overwrites an existing plist', () async {
      if (!Platform.isMacOS) return;
      await StartupHelper.apply(true);
      final firstModified = File(_plistPath()).lastModifiedSync();

      // Small delay so the timestamp can differ if a write occurs.
      await Future<void>.delayed(const Duration(milliseconds: 5));
      await StartupHelper.apply(true);
      final secondModified = File(_plistPath()).lastModifiedSync();

      // The file should have been rewritten.
      expect(
        secondModified.isAtSameMomentAs(firstModified) ||
            secondModified.isAfter(firstModified),
        isTrue,
      );
    });
  });
}
