import 'package:flutter/foundation.dart';

import 'linux_shell.dart';

enum HotkeyRegistrationStatus { registered, fallbackRegistered, failed }

@immutable
class HotkeyBinding {
  const HotkeyBinding({
    required this.virtualKey,
    required this.keyName,
    required this.useCtrl,
    required this.useWin,
    required this.useAlt,
    required this.useShift,
  });

  final int virtualKey;
  final String keyName;
  final bool useCtrl;
  final bool useWin;
  final bool useAlt;
  final bool useShift;

  String label({bool isMac = false}) {
    final parts = <String>[];
    if (useCtrl) parts.add('Ctrl');
    if (useWin) parts.add(isMac ? 'Cmd' : 'Win');
    if (useAlt) parts.add(isMac ? 'Option' : 'Alt');
    if (useShift) parts.add('Shift');
    parts.add(keyName);
    return parts.join('+');
  }

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    return other is HotkeyBinding &&
        other.virtualKey == virtualKey &&
        other.keyName == keyName &&
        other.useCtrl == useCtrl &&
        other.useWin == useWin &&
        other.useAlt == useAlt &&
        other.useShift == useShift;
  }

  @override
  int get hashCode =>
      Object.hash(virtualKey, keyName, useCtrl, useWin, useAlt, useShift);
}

const HotkeyBinding kLinuxTemporaryFallbackHotkey = HotkeyBinding(
  virtualKey: 0x56,
  keyName: 'V',
  useCtrl: true,
  useWin: false,
  useAlt: true,
  useShift: true,
);

@immutable
class HotkeyRegistrationResult {
  const HotkeyRegistrationResult({
    required this.status,
    required this.requestedBinding,
    this.effectiveBinding,
  });

  final HotkeyRegistrationStatus status;
  final HotkeyBinding requestedBinding;
  final HotkeyBinding? effectiveBinding;

  bool get isRegistered =>
      status == HotkeyRegistrationStatus.registered ||
      status == HotkeyRegistrationStatus.fallbackRegistered;
}

abstract class LinuxHotkeyBindingApi {
  Future<bool> registerHotkey(HotkeyBinding binding);
}

class LinuxShellHotkeyBindingApi implements LinuxHotkeyBindingApi {
  const LinuxShellHotkeyBindingApi();

  @override
  Future<bool> registerHotkey(HotkeyBinding binding) {
    return LinuxShell.registerHotkey(
      virtualKey: binding.virtualKey,
      useCtrl: binding.useCtrl,
      useWin: binding.useWin,
      useAlt: binding.useAlt,
      useShift: binding.useShift,
    );
  }
}

Future<HotkeyRegistrationResult> registerLinuxHotkeyWithFallback({
  required LinuxHotkeyBindingApi api,
  required HotkeyBinding requestedBinding,
  HotkeyBinding fallbackBinding = kLinuxTemporaryFallbackHotkey,
}) async {
  if (await api.registerHotkey(requestedBinding)) {
    return HotkeyRegistrationResult(
      status: HotkeyRegistrationStatus.registered,
      requestedBinding: requestedBinding,
      effectiveBinding: requestedBinding,
    );
  }

  if (requestedBinding == fallbackBinding) {
    return HotkeyRegistrationResult(
      status: HotkeyRegistrationStatus.failed,
      requestedBinding: requestedBinding,
    );
  }

  if (await api.registerHotkey(fallbackBinding)) {
    return HotkeyRegistrationResult(
      status: HotkeyRegistrationStatus.fallbackRegistered,
      requestedBinding: requestedBinding,
      effectiveBinding: fallbackBinding,
    );
  }

  return HotkeyRegistrationResult(
    status: HotkeyRegistrationStatus.failed,
    requestedBinding: requestedBinding,
  );
}
