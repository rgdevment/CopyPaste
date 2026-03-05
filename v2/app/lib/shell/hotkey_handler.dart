import 'package:core/core.dart';
import 'package:flutter/services.dart';
import 'package:hotkey_manager/hotkey_manager.dart';

class HotkeyHandler {
  HotkeyHandler({
    required this.config,
    required this.onHotkey,
  });

  final AppConfig config;
  final void Function() onHotkey;
  HotKey? _hotkey;

  Future<bool> _tryRegister(HotKey hotkey) async {
    try {
      await hotKeyManager.register(
        hotkey,
        keyDownHandler: (_) => onHotkey(),
      );
      _hotkey = hotkey;
      return true;
    } catch (e) {
      AppLogger.error('Hotkey registration failed: $e');
      return false;
    }
  }

  Future<void> registerWithFallback() async {
    final modifiers = <HotKeyModifier>[];
    if (config.hotkeyUseCtrl) modifiers.add(HotKeyModifier.control);
    if (config.hotkeyUseWin) modifiers.add(HotKeyModifier.meta);
    if (config.hotkeyUseAlt) modifiers.add(HotKeyModifier.alt);
    if (config.hotkeyUseShift) modifiers.add(HotKeyModifier.shift);

    final keyCode = _mapVirtualKey(config.hotkeyVirtualKey);
    if (keyCode == null) return;

    final primary = HotKey(
      key: keyCode,
      modifiers: modifiers,
      scope: HotKeyScope.system,
    );

    if (await _tryRegister(primary)) return;

    if (config.hotkeyUseWin) {
      final fallbackMods = modifiers
          .where((m) => m != HotKeyModifier.meta)
          .toList()
        ..add(HotKeyModifier.control);

      final fallback = HotKey(
        key: keyCode,
        modifiers: fallbackMods,
        scope: HotKeyScope.system,
      );
      await _tryRegister(fallback);
    }
  }

  Future<void> unregister() async {
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
