import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/shell/desktop_notifier.dart';

void main() {
  tearDown(() {
    DesktopNotifier.processRunnerOverride = null;
  });

  group('DesktopNotifier – macOS', () {
    test('returns false on macOS (no-op)', () async {
      if (!Platform.isMacOS) return;
      final result = await DesktopNotifier.show(title: 'Test', body: 'Body');
      expect(result, isFalse);
    });
  });

  group('DesktopNotifier – Linux routing', () {
    test('spawns notify-send with correct arguments', () async {
      if (!Platform.isLinux) return;

      String? capturedExe;
      List<String>? capturedArgs;
      DesktopNotifier.processRunnerOverride =
          (String exe, List<String> args) async {
            capturedExe = exe;
            capturedArgs = List<String>.from(args);
            return ProcessResult(0, 0, '', '');
          };

      final result = await DesktopNotifier.show(
        title: 'CopyPaste',
        body: 'Running in the background.',
      );

      expect(result, isTrue);
      expect(capturedExe, equals('notify-send'));
      expect(capturedArgs, isNotNull);
      expect(capturedArgs, contains('--app-name=CopyPaste'));
      expect(capturedArgs, contains('--icon=copypaste'));
      expect(capturedArgs, contains('--expire-time=7000'));
      expect(capturedArgs, contains('CopyPaste'));
      expect(capturedArgs, contains('Running in the background.'));
    });

    test('title and body are forwarded verbatim', () async {
      if (!Platform.isLinux) return;

      const title = 'My Title';
      const body = 'My Body Line';
      String? gotTitle;
      String? gotBody;
      DesktopNotifier.processRunnerOverride =
          (String exe, List<String> args) async {
            gotTitle = args[args.length - 2];
            gotBody = args[args.length - 1];
            return ProcessResult(0, 0, '', '');
          };

      await DesktopNotifier.show(title: title, body: body);
      expect(gotTitle, equals(title));
      expect(gotBody, equals(body));
    });

    test('returns false when notify-send exits with non-zero code', () async {
      if (!Platform.isLinux) return;

      DesktopNotifier.processRunnerOverride =
          (String exe, List<String> args) async {
            return ProcessResult(0, 1, '', 'error');
          };

      final result = await DesktopNotifier.show(title: 'Test', body: 'Body');
      expect(result, isFalse);
    });

    test(
      'returns false when notify-send is not installed (ProcessException)',
      () async {
        if (!Platform.isLinux) return;

        DesktopNotifier.processRunnerOverride =
            (String exe, List<String> args) async {
              throw ProcessException(exe, args, 'No such file or directory', 2);
            };

        final result = await DesktopNotifier.show(title: 'Test', body: 'Body');
        expect(result, isFalse);
      },
    );

    test('returns false on unexpected exception (never throws)', () async {
      if (!Platform.isLinux) return;

      DesktopNotifier.processRunnerOverride =
          (String exe, List<String> args) async {
            throw StateError('unexpected');
          };

      final result = await DesktopNotifier.show(title: 'Test', body: 'Body');
      expect(result, isFalse);
    });
  });

  group('DesktopNotifier – processRunnerOverride lifecycle', () {
    test('override is invoked when set', () async {
      if (!Platform.isLinux) return;

      var called = false;
      DesktopNotifier.processRunnerOverride =
          (String exe, List<String> args) async {
            called = true;
            return ProcessResult(0, 0, '', '');
          };

      await DesktopNotifier.show(title: 'T', body: 'B');
      expect(called, isTrue);
    });

    test('override resets to null after tearDown (isolation)', () {
      // Confirms test isolation — override should be null at this point
      // because tearDown clears it.
      expect(DesktopNotifier.processRunnerOverride, isNull);
    });
  });
}
