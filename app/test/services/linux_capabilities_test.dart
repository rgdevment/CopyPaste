import 'dart:io';

import 'package:copypaste/services/linux_capabilities.dart';
import 'package:copypaste/services/linux_guard.dart';
import 'package:copypaste/shell/linux_session.dart';
import 'package:flutter_test/flutter_test.dart';

class _FakeChannel implements LinuxCapabilitiesChannel {
  _FakeChannel({
    this.shellResponse,
    this.listenerResponse,
    this.shellThrows,
    this.listenerThrows,
    this.shellDelay = Duration.zero,
  });

  final Map<Object?, Object?>? shellResponse;
  final Map<Object?, Object?>? listenerResponse;
  final Object? shellThrows;
  final Object? listenerThrows;
  final Duration shellDelay;

  int shellCalls = 0;
  int listenerCalls = 0;

  @override
  Future<Map<Object?, Object?>?> invokeShell(String method) async {
    shellCalls++;
    if (shellDelay > Duration.zero) await Future<void>.delayed(shellDelay);
    if (shellThrows != null) throw shellThrows!;
    return shellResponse;
  }

  @override
  Future<Map<Object?, Object?>?> invokeListener(String method) async {
    listenerCalls++;
    if (listenerThrows != null) throw listenerThrows!;
    return listenerResponse;
  }
}

void main() {
  setUp(() {
    LinuxCapabilitiesService.resetForTesting();
  });

  group('LinuxCapabilitiesService.detect', () {
    test('returns unsupported on non-Linux platforms', () async {
      if (Platform.isLinux) return;
      final caps = await LinuxCapabilitiesService.detect();
      expect(caps, equals(LinuxCapabilities.unsupported));
      expect(LinuxCapabilitiesService.isInitialized, isTrue);
      expect(LinuxCapabilitiesService.current, equals(caps));
    });

    test('parses full capability map from both channels', () async {
      if (!Platform.isLinux) return;
      final channel = _FakeChannel(
        shellResponse: const {
          'isX11': true,
          'hasAppIndicator': true,
          'hasEwmh': true,
          'desktopEnv': 'GNOME',
          'wmName': 'Mutter',
        },
        listenerResponse: const {'isX11': true, 'hasXTest': true},
      );
      final caps = await LinuxCapabilitiesService.detect(channel: channel);
      expect(caps.hasXTest, isTrue);
      expect(caps.hasAppIndicator, isTrue);
      expect(caps.hasEwmh, isTrue);
      expect(caps.detectedDesktopEnv, equals('GNOME'));
      expect(caps.detectedWmName, equals('Mutter'));
      expect(caps.detectionTimedOut, isFalse);
    });

    test('returns conservative defaults when channels throw', () async {
      if (!Platform.isLinux) return;
      final channel = _FakeChannel(
        shellThrows: Exception('shell boom'),
        listenerThrows: Exception('listener boom'),
      );
      final caps = await LinuxCapabilitiesService.detect(channel: channel);
      expect(caps.hasXTest, isFalse);
      expect(caps.hasAppIndicator, isFalse);
      expect(caps.hasEwmh, isFalse);
      expect(caps.detectionTimedOut, isFalse);
    });

    test('marks detectionTimedOut when timeout fires', () async {
      if (!Platform.isLinux) return;
      final channel = _FakeChannel(
        shellResponse: const {'isX11': true, 'hasEwmh': true},
        shellDelay: const Duration(milliseconds: 200),
      );
      final caps = await LinuxCapabilitiesService.detect(
        channel: channel,
        timeout: const Duration(milliseconds: 20),
      );
      expect(caps.detectionTimedOut, isTrue);
      expect(caps.hasEwmh, isFalse);
    });

    test('does not query channels when session is not X11', () async {
      if (Platform.isLinux) return;
      final channel = _FakeChannel(shellResponse: const {'isX11': true});
      await LinuxCapabilitiesService.detect(channel: channel);
      expect(channel.shellCalls, equals(0));
      expect(channel.listenerCalls, equals(0));
    });

    test('caches the last detected value in current', () async {
      if (!Platform.isLinux) return;
      final channel = _FakeChannel(
        shellResponse: const {'isX11': true, 'hasEwmh': true},
        listenerResponse: const {'hasXTest': true},
      );
      final caps = await LinuxCapabilitiesService.detect(channel: channel);
      expect(LinuxCapabilitiesService.current, equals(caps));
    });
  });

  group('LinuxGuard', () {
    test('isLinux delegates to Platform', () {
      expect(LinuxGuard.isLinux, equals(Platform.isLinux));
    });

    test('all guards are false when capabilities are unsupported', () {
      LinuxCapabilitiesService.resetForTesting(LinuxCapabilities.unsupported);
      expect(LinuxGuard.canRegisterHotkey, isFalse);
      expect(LinuxGuard.canPasteBack, isFalse);
      expect(LinuxGuard.canShowTray, isFalse);
      expect(LinuxGuard.canAutostart, isFalse);
      expect(LinuxGuard.usesNativeWindowEffects, isFalse);
    });

    test('canPasteBack requires X11 + XTest', () {
      if (!Platform.isLinux) return;
      LinuxCapabilitiesService.resetForTesting(
        LinuxCapabilities.unsupported.copyWith(isX11: true, hasXTest: true),
      );
      expect(LinuxGuard.canPasteBack, isTrue);
      LinuxCapabilitiesService.resetForTesting(
        LinuxCapabilities.unsupported.copyWith(isX11: true, hasXTest: false),
      );
      expect(LinuxGuard.canPasteBack, isFalse);
    });

    test('canShowTray requires X11 + AppIndicator', () {
      if (!Platform.isLinux) return;
      LinuxCapabilitiesService.resetForTesting(
        LinuxCapabilities.unsupported.copyWith(
          isX11: true,
          hasAppIndicator: true,
        ),
      );
      expect(LinuxGuard.canShowTray, isTrue);
      LinuxCapabilitiesService.resetForTesting(
        LinuxCapabilities.unsupported.copyWith(isX11: true),
      );
      expect(LinuxGuard.canShowTray, isFalse);
    });

    test('canRegisterHotkey requires X11 + EWMH', () {
      if (!Platform.isLinux) return;
      LinuxCapabilitiesService.resetForTesting(
        LinuxCapabilities.unsupported.copyWith(isX11: true, hasEwmh: true),
      );
      expect(LinuxGuard.canRegisterHotkey, isTrue);
    });

    test('isWayland returns true when capabilities have isWayland', () {
      if (!Platform.isLinux) return;
      const waylandSession = LinuxSessionInfo(
        sessionType: 'wayland',
        hasDisplay: false,
        hasWaylandDisplay: true,
        hasWaylandSocket: false,
        desktopEnv: '',
        wmName: '',
      );
      const waylandCaps = LinuxCapabilities(
        session: waylandSession,
        isX11: false,
        hasXTest: false,
        hasAppIndicator: false,
        hasEwmh: false,
        detectedDesktopEnv: '',
        detectedWmName: '',
        detectionTimedOut: false,
      );
      LinuxCapabilitiesService.resetForTesting(waylandCaps);
      expect(LinuxGuard.isWayland, isTrue);
      LinuxCapabilitiesService.resetForTesting(LinuxCapabilities.unsupported);
      expect(LinuxGuard.isWayland, isFalse);
    });

    test('isUsable requires isLinux and X11', () {
      if (!Platform.isLinux) return;
      LinuxCapabilitiesService.resetForTesting(
        LinuxCapabilities.unsupported.copyWith(isX11: true),
      );
      expect(LinuxGuard.isUsable, isTrue);
      LinuxCapabilitiesService.resetForTesting(LinuxCapabilities.unsupported);
      expect(LinuxGuard.isUsable, isFalse);
    });
  });

  group('LinuxCapabilities', () {
    test('copyWith preserves unchanged fields', () {
      if (!Platform.isLinux) return;
      final original = LinuxCapabilities.unsupported.copyWith(
        isX11: true,
        hasXTest: true,
        hasEwmh: true,
        detectedDesktopEnv: 'GNOME',
        detectedWmName: 'Mutter',
      );
      final copy = original.copyWith(hasAppIndicator: true);
      expect(copy.isX11, isTrue);
      expect(copy.hasXTest, isTrue);
      expect(copy.hasEwmh, isTrue);
      expect(copy.hasAppIndicator, isTrue);
      expect(copy.detectedDesktopEnv, 'GNOME');
      expect(copy.detectedWmName, 'Mutter');
    });

    test('toString contains key field values', () {
      if (!Platform.isLinux) return;
      final caps = LinuxCapabilities.unsupported.copyWith(isX11: true);
      final s = caps.toString();
      expect(s, contains('isX11=true'));
      expect(s, contains('LinuxCapabilities('));
    });

    test('isUsable is false when isX11 is false', () {
      if (!Platform.isLinux) return;
      const caps = LinuxCapabilities.unsupported;
      expect(caps.isUsable, isFalse);
    });

    test('isUsable is true when isX11 is true and running on Linux', () {
      if (!Platform.isLinux) return;
      final caps = LinuxCapabilities.unsupported.copyWith(isX11: true);
      expect(caps.isUsable, isTrue);
    });
  });
}
