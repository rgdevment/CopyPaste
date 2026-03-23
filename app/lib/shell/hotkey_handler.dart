// coverage:ignore-file
import 'dart:async';
import 'dart:io';

import 'package:core/core.dart';
import 'package:flutter/services.dart';
import 'package:hotkey_manager/hotkey_manager.dart';

import 'linux_hotkey_registration.dart';
import 'linux_shell.dart';

class HotkeyHandler {
  HotkeyHandler({required this.config, required this.onHotkey});

  final AppConfig config;
  final void Function() onHotkey;
  HotKey? _hotkey;
  StreamSubscription<String>? _linuxEventsSubscription;

  HotkeyBinding get _requestedBinding => HotkeyBinding(
    virtualKey: config.hotkeyVirtualKey,
    keyName: config.hotkeyKeyName,
    useCtrl: config.hotkeyUseCtrl,
    useWin: config.hotkeyUseWin,
    useAlt: config.hotkeyUseAlt,
    useShift: config.hotkeyUseShift,
  );

  Future<bool> _tryRegisterBinding(HotkeyBinding binding) async {
    final keyCode = _mapVirtualKey(binding.virtualKey);
    if (keyCode == null) return false;

    final modifiers = <HotKeyModifier>[];
    if (binding.useCtrl) modifiers.add(HotKeyModifier.control);
    if (binding.useWin) modifiers.add(HotKeyModifier.meta);
    if (binding.useAlt) modifiers.add(HotKeyModifier.alt);
    if (binding.useShift) modifiers.add(HotKeyModifier.shift);

    final hotkey = HotKey(
      key: keyCode,
      modifiers: modifiers,
      scope: HotKeyScope.system,
    );

    try {
      await hotKeyManager.register(hotkey, keyDownHandler: (_) => onHotkey());
      _hotkey = hotkey;
      return true;
    } catch (e) {
      AppLogger.error('Hotkey registration failed: $e');
      return false;
    }
  }

  Future<HotkeyRegistrationResult> registerWithFallback() async {
    if (_hotkey != null || _linuxEventsSubscription != null) {
      await unregister();
    }

    if (Platform.isLinux) {
      _linuxEventsSubscription ??= LinuxShell.events.listen((event) {
        if (event == 'hotkey') onHotkey();
      });
      return registerLinuxHotkeyWithFallback(
        api: const LinuxShellHotkeyBindingApi(),
        requestedBinding: _requestedBinding,
      );
    }

    final requestedBinding = _requestedBinding;
    if (await _tryRegisterBinding(requestedBinding)) {
      return HotkeyRegistrationResult(
        status: HotkeyRegistrationStatus.registered,
        requestedBinding: requestedBinding,
        effectiveBinding: requestedBinding,
      );
    }

    if (config.hotkeyUseWin) {
      final fallbackBinding = HotkeyBinding(
        virtualKey: requestedBinding.virtualKey,
        keyName: requestedBinding.keyName,
        useCtrl: true,
        useWin: false,
        useAlt: requestedBinding.useAlt,
        useShift: requestedBinding.useShift,
      );
      if (await _tryRegisterBinding(fallbackBinding)) {
        return HotkeyRegistrationResult(
          status: HotkeyRegistrationStatus.fallbackRegistered,
          requestedBinding: requestedBinding,
          effectiveBinding: fallbackBinding,
        );
      }
    }

    return HotkeyRegistrationResult(
      status: HotkeyRegistrationStatus.failed,
      requestedBinding: requestedBinding,
    );
  }

  Future<void> unregister() async {
    if (Platform.isLinux) {
      await _linuxEventsSubscription?.cancel();
      _linuxEventsSubscription = null;
      await LinuxShell.unregisterHotkey();
      _hotkey = null;
      return;
    }

    if (_hotkey != null) {
      await hotKeyManager.unregister(_hotkey!);
      _hotkey = null;
    }
  }

  static PhysicalKeyboardKey? _mapVirtualKey(int vk) {
    const map = <int, PhysicalKeyboardKey>{
      0x41: PhysicalKeyboardKey.keyA,
      0x42: PhysicalKeyboardKey.keyB,
      0x43: PhysicalKeyboardKey.keyC,
      0x44: PhysicalKeyboardKey.keyD,
      0x45: PhysicalKeyboardKey.keyE,
      0x46: PhysicalKeyboardKey.keyF,
      0x47: PhysicalKeyboardKey.keyG,
      0x48: PhysicalKeyboardKey.keyH,
      0x49: PhysicalKeyboardKey.keyI,
      0x4A: PhysicalKeyboardKey.keyJ,
      0x4B: PhysicalKeyboardKey.keyK,
      0x4C: PhysicalKeyboardKey.keyL,
      0x4D: PhysicalKeyboardKey.keyM,
      0x4E: PhysicalKeyboardKey.keyN,
      0x4F: PhysicalKeyboardKey.keyO,
      0x50: PhysicalKeyboardKey.keyP,
      0x51: PhysicalKeyboardKey.keyQ,
      0x52: PhysicalKeyboardKey.keyR,
      0x53: PhysicalKeyboardKey.keyS,
      0x54: PhysicalKeyboardKey.keyT,
      0x55: PhysicalKeyboardKey.keyU,
      0x56: PhysicalKeyboardKey.keyV,
      0x57: PhysicalKeyboardKey.keyW,
      0x58: PhysicalKeyboardKey.keyX,
      0x59: PhysicalKeyboardKey.keyY,
      0x5A: PhysicalKeyboardKey.keyZ,
    };
    return map[vk];
  }
}
