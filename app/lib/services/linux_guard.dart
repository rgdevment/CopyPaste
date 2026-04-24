import 'dart:io';

import 'linux_capabilities.dart';

class LinuxGuard {
  const LinuxGuard._(); // coverage:ignore-line

  static LinuxCapabilities get _caps => LinuxCapabilitiesService.current;

  static bool get isLinux => Platform.isLinux;
  static bool get isUsable => isLinux && _caps.isX11;
  static bool get isWayland => isLinux && _caps.isWayland;

  static bool get canRegisterHotkey => isUsable && _caps.hasEwmh;
  static bool get canPasteBack => isUsable && _caps.hasXTest;
  static bool get canShowTray => isUsable && _caps.hasAppIndicator;
  static bool get canPersistClipboard => isUsable && _caps.hasClipboardManager;
  static bool get canAutostart => isUsable;
  static bool get usesNativeWindowEffects => false;
}
