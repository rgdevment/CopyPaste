// coverage:ignore-file
import 'dart:io';

import 'package:core/core.dart';
import 'package:flutter/services.dart';
import 'package:hotkey_manager/hotkey_manager.dart';

import 'hotkey_binding.dart';
import 'windows_hotkey_channel.dart';

class HotkeyHandler {
  HotkeyHandler({
    required this.config,
    required void Function() onHotkey,
    required void Function() onPlainPasteHotkey,
  }) : _onHotkey = onHotkey,
       _onPlainPasteHotkey = onPlainPasteHotkey;

  final AppConfig config;
  void Function()? _onHotkey;
  void Function()? _onPlainPasteHotkey;
  HotKey? _hotkey;
  HotKey? _plainPasteHotkey;
  WindowsHotkeyChannel? _windowsHotkeys;
  bool? _plainPasteRegistrationSucceeded;
  bool? get plainPasteRegistrationSucceeded => _plainPasteRegistrationSucceeded;

  HotkeyBinding get _requestedBinding => HotkeyBinding(
    virtualKey: config.hotkeyVirtualKey,
    keyName: config.hotkeyKeyName,
    useCtrl: config.hotkeyUseCtrl,
    useWin: config.hotkeyUseWin,
    useAlt: config.hotkeyUseAlt,
    useShift: config.hotkeyUseShift,
  );

  HotkeyBinding get _plainPasteBinding => HotkeyBinding(
    virtualKey: config.plainPasteHotkeyVirtualKey,
    keyName: config.plainPasteHotkeyKeyName,
    useCtrl: config.plainPasteHotkeyUseCtrl,
    useWin: config.plainPasteHotkeyUseWin,
    useAlt: config.plainPasteHotkeyUseAlt,
    useShift: config.plainPasteHotkeyUseShift,
  );

  Future<HotKey?> _tryRegisterBinding(
    HotkeyBinding binding,
    void Function() callback, {
    bool triggerOnKeyUp = false,
  }) async {
    final keyCode = _mapVirtualKey(binding.virtualKey);
    if (keyCode == null) return null;

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
      await hotKeyManager.register(
        hotkey,
        keyDownHandler: triggerOnKeyUp ? null : (_) => callback(),
        keyUpHandler: triggerOnKeyUp ? (_) => callback() : null,
      );
      return hotkey;
    } catch (e) {
      AppLogger.error('Hotkey registration failed for ${binding.label()}: $e');
      return null;
    }
  }

  Future<HotkeyRegistrationResult> registerWithFallback() async {
    _plainPasteRegistrationSucceeded = config.plainPasteHotkeyEnabled
        ? false
        : null;
    if (_hotkey != null ||
        _plainPasteHotkey != null ||
        _windowsHotkeys != null) {
      await unregister();
    }

    if (Platform.isWindows) {
      return _registerWindowsHotkeys();
    }

    final requestedBinding = _requestedBinding;
    _hotkey = await _tryRegisterBinding(
      requestedBinding,
      () => _onHotkey?.call(),
    );
    if (_hotkey != null) {
      await _registerPlainPasteBinding();
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
      _hotkey = await _tryRegisterBinding(
        fallbackBinding,
        () => _onHotkey?.call(),
      );
      if (_hotkey != null) {
        await _registerPlainPasteBinding();
        return HotkeyRegistrationResult(
          status: HotkeyRegistrationStatus.fallbackRegistered,
          requestedBinding: requestedBinding,
          effectiveBinding: fallbackBinding,
        );
      }
    }

    await _registerPlainPasteBinding();
    return HotkeyRegistrationResult(
      status: HotkeyRegistrationStatus.failed,
      requestedBinding: requestedBinding,
    );
  }

  Future<HotkeyRegistrationResult> _registerWindowsHotkeys() async {
    final requestedBinding = _requestedBinding;
    final channel = WindowsHotkeyChannel();
    _windowsHotkeys = channel;

    try {
      await channel.start((id) {
        if (id == 'open') _onHotkey?.call();
        if (id == 'plainPaste') _onPlainPasteHotkey?.call();
      });
    } catch (e) {
      AppLogger.error('Windows hotkey channel initialization failed: $e');
      _plainPasteRegistrationSucceeded = config.plainPasteHotkeyEnabled
          ? false
          : null;
      return HotkeyRegistrationResult(
        status: HotkeyRegistrationStatus.failed,
        requestedBinding: requestedBinding,
      );
    }

    var effectiveBinding = requestedBinding;
    var openResponse = await _registerWindowsBinding(
      channel,
      id: 'open',
      binding: requestedBinding,
    );
    var status = HotkeyRegistrationStatus.registered;

    if (!openResponse.success && config.hotkeyUseWin) {
      final fallbackBinding = HotkeyBinding(
        virtualKey: requestedBinding.virtualKey,
        keyName: requestedBinding.keyName,
        useCtrl: true,
        useWin: false,
        useAlt: requestedBinding.useAlt,
        useShift: requestedBinding.useShift,
      );
      openResponse = await _registerWindowsBinding(
        channel,
        id: 'open',
        binding: fallbackBinding,
      );
      if (openResponse.success) {
        effectiveBinding = fallbackBinding;
        status = HotkeyRegistrationStatus.fallbackRegistered;
      }
    }

    if (config.plainPasteHotkeyEnabled) {
      final response = await _registerWindowsBinding(
        channel,
        id: 'plainPaste',
        binding: _plainPasteBinding,
      );
      _plainPasteRegistrationSucceeded = response.success;
    }

    if (!openResponse.success) {
      return HotkeyRegistrationResult(
        status: HotkeyRegistrationStatus.failed,
        requestedBinding: requestedBinding,
      );
    }

    AppLogger.info(
      'Windows open hotkey registered: ${effectiveBinding.label()}',
    );
    return HotkeyRegistrationResult(
      status: status,
      requestedBinding: requestedBinding,
      effectiveBinding: effectiveBinding,
    );
  }

  Future<WindowsHotkeyRegistrationResponse> _registerWindowsBinding(
    WindowsHotkeyChannel channel, {
    required String id,
    required HotkeyBinding binding,
  }) async {
    try {
      final response = await channel.register(
        id: id,
        virtualKey: binding.virtualKey,
        useCtrl: binding.useCtrl,
        useWin: binding.useWin,
        useAlt: binding.useAlt,
        useShift: binding.useShift,
      );
      if (!response.success) {
        AppLogger.error(
          'Windows hotkey registration failed: id=$id, '
          'binding=${binding.label()}, error=${response.errorCode}, '
          'win32=${response.win32Error}',
        );
      } else if (id == 'plainPaste') {
        AppLogger.info(
          'Windows plain-paste hotkey registered: ${binding.label()}',
        );
      }
      return response;
    } catch (e) {
      AppLogger.error(
        'Windows hotkey registration call failed: id=$id, '
        'binding=${binding.label()}, error=$e',
      );
      return const WindowsHotkeyRegistrationResponse(
        success: false,
        errorCode: 'platformCallFailed',
      );
    }
  }

  Future<void> _registerPlainPasteBinding() async {
    if (!config.plainPasteHotkeyEnabled) return;
    _plainPasteHotkey = await _tryRegisterBinding(
      _plainPasteBinding,
      () => _onPlainPasteHotkey?.call(),
      triggerOnKeyUp: true,
    );
    _plainPasteRegistrationSucceeded = _plainPasteHotkey != null;
  }

  Future<void> unregister({bool releaseCallbacks = false}) async {
    if (releaseCallbacks) {
      // Break references to the owning State before platform calls. Some
      // hotkey backends can fail during teardown; stale package callbacks then
      // remain harmless and cannot retain the widget tree.
      _onHotkey = null;
      _onPlainPasteHotkey = null;
    }
    if (Platform.isWindows) {
      final channel = _windowsHotkeys;
      _windowsHotkeys = null;
      _hotkey = null;
      _plainPasteHotkey = null;
      _plainPasteRegistrationSucceeded = null;
      if (channel != null) {
        try {
          await channel.dispose();
        } catch (e) {
          AppLogger.error('Windows hotkey cleanup failed: $e');
        }
      }
      return;
    }
    final registered = <HotKey>[];
    if (_hotkey != null) registered.add(_hotkey!);
    if (_plainPasteHotkey != null) registered.add(_plainPasteHotkey!);
    _hotkey = null;
    _plainPasteHotkey = null;
    _plainPasteRegistrationSucceeded = null;

    var individualFailure = false;
    for (final hotkey in registered) {
      try {
        await hotKeyManager.unregister(hotkey);
      } catch (e) {
        individualFailure = true;
        AppLogger.error('Hotkey unregistration failed: $e');
      }
    }
    // hotkey_manager only removes its callback maps after the platform call
    // succeeds. Clear the singleton as a fallback so a failed unregister does
    // not retain this State object through an old callback.
    if (individualFailure) {
      try {
        await hotKeyManager.unregisterAll();
      } catch (e) {
        AppLogger.error('Fallback hotkey cleanup failed: $e');
      }
    }
  }

  Future<void> dispose() => unregister(releaseCallbacks: true);

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
