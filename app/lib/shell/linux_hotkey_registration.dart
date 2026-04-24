import 'package:flutter/foundation.dart';

import 'linux_shell.dart';

enum HotkeyRegistrationStatus { registered, fallbackRegistered, failed }

enum HotkeyFailureReason {
  unsupportedKey,
  noModifier,
  grabFailed,
  noX11,
  channelError,
  unknown,
}

HotkeyFailureReason _reasonFromCode(String? code) {
  switch (code) {
    case 'unsupportedKey':
      return HotkeyFailureReason.unsupportedKey;
    case 'noModifier':
      return HotkeyFailureReason.noModifier;
    case 'grabFailed':
      return HotkeyFailureReason.grabFailed;
    case 'noX11':
      return HotkeyFailureReason.noX11;
    case 'channelError':
      return HotkeyFailureReason.channelError;
    default:
      return HotkeyFailureReason.unknown;
  }
}

final Set<int> _supportedLinuxVirtualKeys = <int>{
  for (var k = 0x41; k <= 0x5A; k++) k,
  for (var k = 0x30; k <= 0x39; k++) k,
  for (var k = 0x70; k <= 0x87; k++) k,
  0x08, 0x09, 0x0D, 0x1B, 0x20,
  0x21, 0x22, 0x23, 0x24,
  0x25, 0x26, 0x27, 0x28,
  0x2D, 0x2E,
  0xBA, 0xBB, 0xBC, 0xBD, 0xBE, 0xBF, 0xC0,
  0xDB, 0xDC, 0xDD, 0xDE,
};

bool isLinuxSupportedVirtualKey(int virtualKey) =>
    _supportedLinuxVirtualKeys.contains(virtualKey);

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
    this.failureReason,
  });

  final HotkeyRegistrationStatus status;
  final HotkeyBinding requestedBinding;
  final HotkeyBinding? effectiveBinding;
  final HotkeyFailureReason? failureReason;

  bool get isRegistered =>
      status == HotkeyRegistrationStatus.registered ||
      status == HotkeyRegistrationStatus.fallbackRegistered;
}

abstract class LinuxHotkeyBindingApi {
  Future<HotkeyRegisterResponse> registerHotkey(HotkeyBinding binding);
}

class LinuxShellHotkeyBindingApi implements LinuxHotkeyBindingApi {
  const LinuxShellHotkeyBindingApi();

  @override
  Future<HotkeyRegisterResponse> registerHotkey(HotkeyBinding binding) {
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
  if (!isLinuxSupportedVirtualKey(requestedBinding.virtualKey)) {
    return HotkeyRegistrationResult(
      status: HotkeyRegistrationStatus.failed,
      requestedBinding: requestedBinding,
      failureReason: HotkeyFailureReason.unsupportedKey,
    );
  }

  final primary = await api.registerHotkey(requestedBinding);
  if (primary.success) {
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
      failureReason: _reasonFromCode(primary.errorCode),
    );
  }

  final fallback = await api.registerHotkey(fallbackBinding);
  if (fallback.success) {
    return HotkeyRegistrationResult(
      status: HotkeyRegistrationStatus.fallbackRegistered,
      requestedBinding: requestedBinding,
      effectiveBinding: fallbackBinding,
      failureReason: _reasonFromCode(primary.errorCode),
    );
  }

  return HotkeyRegistrationResult(
    status: HotkeyRegistrationStatus.failed,
    requestedBinding: requestedBinding,
    failureReason: _reasonFromCode(fallback.errorCode ?? primary.errorCode),
  );
}
