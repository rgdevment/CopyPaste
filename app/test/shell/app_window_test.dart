import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:app/shell/app_window.dart';

// ---------------------------------------------------------------------------
// Mock for window_manager and screen_retriever MethodChannels
// ---------------------------------------------------------------------------

const _wmChannel = MethodChannel('window_manager');
const _screenRetrieverChannel = MethodChannel(
  'dev.leanflutter.plugins/screen_retriever',
);

void _setupWindowManagerMock({required List<MethodCall> calls}) {
  TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
      .setMockMethodCallHandler(_wmChannel, (call) async {
        calls.add(call);
        switch (call.method) {
          case 'getBounds':
            return {'x': 100.0, 'y': 100.0, 'width': 368.0, 'height': 500.0};
          case 'getSize':
            return {'width': 368.0, 'height': 500.0};
          case 'getPosition':
            return {'x': 100.0, 'y': 100.0};
          case 'isMinimized':
          case 'isMaximized':
          case 'isFullScreen':
          case 'isVisible':
            return false;
          default:
            return null;
        }
      });

  // screen_retriever is used by _positionNearCursorNative
  TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
      .setMockMethodCallHandler(_screenRetrieverChannel, (call) async {
        switch (call.method) {
          // getCursorScreenPoint returns {dx, dy} per _OffsetConverter
          case 'getCursorScreenPoint':
            return {'dx': 200.0, 'dy': 200.0};
          // Display JSON: id + size {width,height} + visiblePosition {dx,dy}
          // visiblePosition is null-checked by calc_window_position.dart
          case 'getPrimaryDisplay':
            return {
              'id': 'screen1',
              'size': {'width': 1920.0, 'height': 1080.0},
              'visiblePosition': {'dx': 0.0, 'dy': 0.0},
              'visibleSize': {'width': 1920.0, 'height': 1040.0},
              'scaleFactor': 1.0,
            };
          case 'getAllDisplays':
            return {
              'displays': [
                {
                  'id': 'screen1',
                  'size': {'width': 1920.0, 'height': 1080.0},
                  'visiblePosition': {'dx': 0.0, 'dy': 0.0},
                  'visibleSize': {'width': 1920.0, 'height': 1040.0},
                  'scaleFactor': 1.0,
                },
              ],
            };
          default:
            return null;
        }
      });
}

void _teardownWindowManagerMock() {
  TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
      .setMockMethodCallHandler(_wmChannel, null);
  TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
      .setMockMethodCallHandler(_screenRetrieverChannel, null);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('AppWindow.show() on Linux', () {
    late List<MethodCall> calls;

    setUp(() {
      calls = [];
      _setupWindowManagerMock(calls: calls);
    });

    tearDown(_teardownWindowManagerMock);

    test('calls setOpacity(1.0) on Linux after show()', () async {
      if (!Platform.isLinux) return;

      bool visibilityChanged = false;
      final window = AppWindow(
        onVisibilityChanged: (_) => visibilityChanged = true,
        popupWidth: 368,
        popupHeight: 500,
      );

      await window.show();

      final opacityCall = calls.where((c) => c.method == 'setOpacity').toList();
      expect(
        opacityCall,
        isNotEmpty,
        reason: 'setOpacity should be called on Linux',
      );
      expect(opacityCall.last.arguments, equals({'opacity': 1.0}));
      expect(visibilityChanged, isTrue);
    });

    test('does NOT call setOpacity on macOS', () async {
      if (!Platform.isMacOS) return;

      final window = AppWindow(popupWidth: 368, popupHeight: 500);
      await window.show();

      final opacityCall = calls.where((c) => c.method == 'setOpacity').toList();
      expect(
        opacityCall,
        isEmpty,
        reason: 'setOpacity should not be called on macOS',
      );
    });

    test('isVisible becomes true after show()', () async {
      if (!Platform.isLinux && !Platform.isMacOS) return;

      final window = AppWindow(popupWidth: 368, popupHeight: 500);
      expect(window.isVisible, isFalse);
      await window.show();
      expect(window.isVisible, isTrue);
    });

    test('isVisible becomes false after hide()', () async {
      if (!Platform.isLinux && !Platform.isMacOS) return;

      final window = AppWindow(popupWidth: 368, popupHeight: 500);
      await window.show();
      expect(window.isVisible, isTrue);
      await window.hide();
      expect(window.isVisible, isFalse);
    });

    test('hide() is no-op when already hidden', () async {
      if (!Platform.isLinux && !Platform.isMacOS) return;

      final window = AppWindow(popupWidth: 368, popupHeight: 500);
      // Not yet shown — hiding should do nothing.
      final callCountBefore = calls.length;
      await window.hide();
      expect(
        calls.length,
        equals(callCountBefore),
        reason: 'hide() should be a no-op when already hidden',
      );
    });

    test('toggle() shows when hidden and hides when visible', () async {
      if (!Platform.isLinux && !Platform.isMacOS) return;

      final window = AppWindow(popupWidth: 368, popupHeight: 500);
      expect(window.isVisible, isFalse);

      await window.toggle(); // hidden → visible
      expect(window.isVisible, isTrue);

      await window.toggle(); // visible → hidden
      expect(window.isVisible, isFalse);
    });

    test('onVisibilityChanged callback fires on show and hide', () async {
      if (!Platform.isLinux && !Platform.isMacOS) return;

      final events = <bool>[];
      final window = AppWindow(
        onVisibilityChanged: events.add,
        popupWidth: 368,
        popupHeight: 500,
      );

      await window.show();
      await window.hide();

      expect(events, equals([true, false]));
    });
  });

  group('AppWindow.updatePopupSize', () {
    test('updates width and height', () {
      final window = AppWindow(popupWidth: 360, popupHeight: 500);
      window.updatePopupSize(400, 600);
      // No direct getters, but we can verify it doesn't throw.
    });
  });
}
