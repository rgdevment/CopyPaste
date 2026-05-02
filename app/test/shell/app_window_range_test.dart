import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/shell/app_window.dart';

void main() {
  group('AppWindow.isPositionInSaneRange', () {
    test('origin (0, 0) is valid', () {
      expect(AppWindow.isPositionInSaneRange(0, 0), isTrue);
    });

    test('(-9999, -9999) is within range', () {
      expect(AppWindow.isPositionInSaneRange(-9999, -9999), isTrue);
    });

    test('x below -10000 is out of range', () {
      expect(AppWindow.isPositionInSaneRange(-10001, 0), isFalse);
    });

    test('y below -10000 is out of range', () {
      expect(AppWindow.isPositionInSaneRange(0, -10001), isFalse);
    });

    test('x above 50000 is out of range', () {
      expect(AppWindow.isPositionInSaneRange(50001, 0), isFalse);
    });

    test('y above 30000 is out of range', () {
      expect(AppWindow.isPositionInSaneRange(0, 30001), isFalse);
    });

    test('NaN x is invalid', () {
      expect(AppWindow.isPositionInSaneRange(double.nan, 0), isFalse);
    });

    test('NaN y is invalid', () {
      expect(AppWindow.isPositionInSaneRange(0, double.nan), isFalse);
    });

    test('infinite x is invalid', () {
      expect(AppWindow.isPositionInSaneRange(double.infinity, 0), isFalse);
    });

    test('(-32000, -32000) is out of range (minimized Windows position)', () {
      expect(AppWindow.isPositionInSaneRange(-32000, -32000), isFalse);
    });

    test('(1920, 1080) is valid (typical secondary monitor)', () {
      expect(AppWindow.isPositionInSaneRange(1920, 1080), isTrue);
    });

    test('(3840, 0) is valid (4K secondary monitor to the right)', () {
      expect(AppWindow.isPositionInSaneRange(3840, 0), isTrue);
    });
  });
}
