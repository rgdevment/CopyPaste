import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/shell/linux_session.dart';
import 'package:copypaste/shell/startup_helper.dart';

String _desktopPath() {
  const appName = 'CopyPaste';
  final home = Platform.environment['HOME'] ?? '/tmp';
  return '$home/.config/autostart/$appName.desktop';
}

void main() {
  tearDown(() async {
    if (!Platform.isLinux) return;
    final f = File(_desktopPath());
    if (f.existsSync()) f.deleteSync();
  });

  group('StartupHelper – Linux', () {
    test('apply(true) creates the autostart .desktop file', () async {
      if (!Platform.isLinux || isWaylandSession()) return;

      await StartupHelper.apply(true);
      expect(File(_desktopPath()).existsSync(), isTrue);
    });

    test('.desktop file contains [Desktop Entry] header', () async {
      if (!Platform.isLinux || isWaylandSession()) return;

      await StartupHelper.apply(true);
      final content = File(_desktopPath()).readAsStringSync();
      expect(content, contains('[Desktop Entry]'));
    });

    test('.desktop file has Type=Application', () async {
      if (!Platform.isLinux || isWaylandSession()) return;

      await StartupHelper.apply(true);
      final content = File(_desktopPath()).readAsStringSync();
      expect(content, contains('Type=Application'));
    });

    test('.desktop file contains Name=CopyPaste', () async {
      if (!Platform.isLinux || isWaylandSession()) return;

      await StartupHelper.apply(true);
      final content = File(_desktopPath()).readAsStringSync();
      expect(content, contains('Name=CopyPaste'));
    });

    test(
      '.desktop file contains Exec pointing to current executable',
      () async {
        if (!Platform.isLinux || isWaylandSession()) return;

        await StartupHelper.apply(true);
        final content = File(_desktopPath()).readAsStringSync();
        expect(content, contains('Exec=${Platform.resolvedExecutable}'));
      },
    );

    test('.desktop file has X-GNOME-Autostart-enabled=true', () async {
      if (!Platform.isLinux || isWaylandSession()) return;

      await StartupHelper.apply(true);
      final content = File(_desktopPath()).readAsStringSync();
      expect(content, contains('X-GNOME-Autostart-enabled=true'));
    });

    test('.desktop file has Terminal=false', () async {
      if (!Platform.isLinux || isWaylandSession()) return;

      await StartupHelper.apply(true);
      final content = File(_desktopPath()).readAsStringSync();
      expect(content, contains('Terminal=false'));
    });

    test('apply(false) removes the autostart .desktop file', () async {
      if (!Platform.isLinux || isWaylandSession()) return;

      await StartupHelper.apply(true);
      expect(File(_desktopPath()).existsSync(), isTrue);
      await StartupHelper.apply(false);
      expect(File(_desktopPath()).existsSync(), isFalse);
    });

    test(
      'apply(false) does not throw when .desktop file does not exist',
      () async {
        if (!Platform.isLinux) return;

        final f = File(_desktopPath());
        if (f.existsSync()) f.deleteSync();
        await expectLater(StartupHelper.apply(false), completes);
      },
    );

    test('apply(true) overwrites an existing .desktop file', () async {
      if (!Platform.isLinux || isWaylandSession()) return;

      await StartupHelper.apply(true);
      final first = File(_desktopPath()).lastModifiedSync();

      await Future<void>.delayed(const Duration(milliseconds: 5));
      await StartupHelper.apply(true);
      final second = File(_desktopPath()).lastModifiedSync();

      expect(second.isAtSameMomentAs(first) || second.isAfter(first), isTrue);
    });

    test('.desktop file is placed in HOME/.config/autostart/', () async {
      if (!Platform.isLinux || isWaylandSession()) return;

      await StartupHelper.apply(true);
      final home = Platform.environment['HOME'] ?? '/tmp';
      final expectedDir = '$home/.config/autostart';
      expect(File(_desktopPath()).parent.path, equals(expectedDir));
    });
  });

  group('StartupHelper – Linux Wayland skip', () {
    test(
      'apply(true) does NOT create .desktop file on Wayland session',
      () async {
        if (!Platform.isLinux) return;
        if (!isWaylandSession()) return; // only meaningful on Wayland

        final f = File(_desktopPath());
        if (f.existsSync()) f.deleteSync();

        await StartupHelper.apply(true);

        expect(
          f.existsSync(),
          isFalse,
          reason: 'Autostart must be skipped on Wayland',
        );
      },
    );

    test(
      'apply(true) removes existing .desktop file on Wayland session',
      () async {
        if (!Platform.isLinux) return;
        if (!isWaylandSession()) return;

        // Pre-create the file to simulate a stale entry from a previous X11 session.
        final f = File(_desktopPath());
        f.parent.createSync(recursive: true);
        f.writeAsStringSync('[Desktop Entry]\nType=Application\n');

        await StartupHelper.apply(true);

        expect(
          f.existsSync(),
          isFalse,
          reason: 'Stale autostart entry must be removed on Wayland',
        );
      },
    );

    test(
      'on X11 session, apply(true) creates the .desktop file normally',
      () async {
        if (!Platform.isLinux) return;
        if (isWaylandSession()) return; // skip on Wayland

        await StartupHelper.apply(true);
        expect(File(_desktopPath()).existsSync(), isTrue);
      },
    );
  });
}
