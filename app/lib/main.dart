// coverage:ignore-file
import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:ui' show PlatformDispatcher;

import 'package:core/core.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_acrylic/flutter_acrylic.dart';
import 'package:listener/listener.dart';
import 'package:window_manager/window_manager.dart';

import 'services/auto_update_service.dart';

import 'shell/app_window.dart';
import 'shell/focus_manager.dart';
import 'shell/hotkey_handler.dart';
import 'shell/linux_hotkey_registration.dart';
import 'shell/linux_session.dart';
import 'shell/linux_shell.dart';
import 'shell/single_instance.dart';
import 'shell/startup_helper.dart';
import 'shell/tray_icon.dart';
import 'shell/windows_balloon.dart';
import 'screens/main_screen.dart';
import 'screens/settings_screen.dart';
import 'screens/wayland_unsupported_screen.dart';
import 'theme/compact_theme.dart';
import 'theme/theme_provider.dart';
import 'l10n/app_localizations.dart';
import 'screens/permission_gate_screen.dart';
import 'screens/windows_onboarding_screen.dart';

// Re-exported so existing tests can import isWaylandSession from main.dart.
export 'shell/linux_session.dart' show isWaylandSession;

bool _isMicaDark(String themeMode) => switch (themeMode) {
  'dark' => true,
  'auto' ||
  'system' => PlatformDispatcher.instance.platformBrightness == Brightness.dark,
  _ => false,
};

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  if (!SingleInstance.acquire()) {
    exit(0);
  }

  await windowManager.ensureInitialized();

  if (Platform.isWindows || Platform.isMacOS) {
    try {
      await Window.initialize();
    } catch (_) {
      // AppLogger not yet initialized here; app continues without acrylic effects
    }
  }

  final storage = await StorageConfig.create();
  await storage.ensureDirectories();
  AppLogger.initialize(storage.logsPath);

  FlutterError.onError = (details) {
    AppLogger.error(
      'FlutterError: ${details.exceptionAsString()}\n${details.stack}',
    );
  };
  PlatformDispatcher.instance.onError = (error, stack) {
    AppLogger.error('Unhandled: $error\n$stack');
    return false;
  };

  final config = await AppConfig.load(
    '${storage.configPath}/${AppConfig.fileName}',
  );

  final repo = SqliteRepository.fromPath(storage.databasePath);
  final clipboardService = ClipboardService(
    repo,
    imagesPath: storage.imagesPath,
  )..pasteIgnoreWindowMs = config.duplicateIgnoreWindowMs;

  final cleanupService = CleanupService(
    repo,
    () => config.retentionDays,
    storage: storage,
  )..start(storage.baseDir);

  final listener = ClipboardListener();

  await StartupHelper.apply(config.runOnStartup);

  try {
    if (Platform.isWindows) {
      await Window.setEffect(
        effect: WindowEffect.mica,
        color: const Color(0x00000000),
        dark: _isMicaDark(config.themeMode),
      );
    } else if (Platform.isMacOS) {
      await Window.setEffect(
        effect: WindowEffect.sidebar,
        color: const Color(0x00000000),
        dark: _isMicaDark(config.themeMode),
      );
    }
  } catch (e) {
    AppLogger.error('Window.setEffect failed: $e');
  }

  runApp(
    CopyPasteApp(
      storage: storage,
      config: config,
      repo: repo,
      clipboardService: clipboardService,
      cleanupService: cleanupService,
      listener: listener,
    ),
  );
}

class CopyPasteApp extends StatefulWidget {
  const CopyPasteApp({
    required this.storage,
    required this.config,
    required this.repo,
    required this.clipboardService,
    required this.cleanupService,
    required this.listener,
    super.key,
  });

  final StorageConfig storage;
  final AppConfig config;
  final SqliteRepository repo;
  final ClipboardService clipboardService;
  final CleanupService cleanupService;
  final ClipboardListener listener;

  @override
  State<CopyPasteApp> createState() => _CopyPasteAppState();
}

class _CopyPasteAppState extends State<CopyPasteApp>
    with WindowListener, WidgetsBindingObserver {
  late final AppWindow _appWindow;
  late final TrayIcon _trayIcon;
  late HotkeyHandler _hotkeyHandler;
  late AppConfig _config;
  final WindowFocusManager _focusManager = WindowFocusManager();
  final _mainScreenKey = GlobalKey<MainScreenState>();
  final _navigatorKey = GlobalKey<NavigatorState>();
  StreamSubscription<ClipboardEvent>? _listenerSubscription;
  String? _lastTrayLocale;
  bool _showPermissionGate = false;
  bool _showWindowsOnboarding = false;
  bool _showWaylandUnsupported = false;
  String? _availableUpdateVersion;
  bool _programmaticRestore = false;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    _config = widget.config;
    _appWindow = AppWindow(
      onVisibilityChanged: _onWindowVisibilityChanged,
      showInTaskbar: _config.showInTaskbar,
      popupWidth: _config.popupWidth.toDouble(),
      popupHeight: _config.popupHeight.toDouble(),
    );
    _trayIcon = TrayIcon(onToggle: _toggleWindow, onExit: _exitApp);
    _hotkeyHandler = HotkeyHandler(config: _config, onHotkey: _onHotkey);

    unawaited(
      _initShell().catchError(
        (Object e, StackTrace s) =>
            AppLogger.error('_initShell failed: $e\n$s'),
      ),
    );
  }

  @override
  void didChangePlatformBrightness() {
    if (_config.themeMode == 'auto' &&
        (Platform.isWindows || Platform.isMacOS)) {
      unawaited(
        _appWindow
            .applyEffect(dark: _isMicaDark('auto'))
            .catchError(
              (Object e) => AppLogger.error('applyEffect failed: $e'),
            ),
      );
    }
  }

  Future<void> _initShell() async {
    windowManager.addListener(this);
    final isFirstRun = widget.storage.isFirstRun;
    final wayland = Platform.isLinux && isWaylandSession();

    if (wayland) {
      // Show the unsupported screen and stop all further initialisation.
      await _appWindow.init(startVisible: true);
      await _appWindow.enterGateMode();
      if (mounted) setState(() => _showWaylandUnsupported = true);
      return;
    }

    _startListening();

    bool macosGranted = true;
    if (Platform.isMacOS) {
      macosGranted = await ClipboardWriter.checkAccessibility();
    }

    final showOnStart =
        isFirstRun && (Platform.isLinux || (Platform.isMacOS && macosGranted));
    await _appWindow.init(startVisible: showOnStart);
    SingleInstance.listenForWakeup(() {
      unawaited(_safeShow());
      if (Platform.isWindows) unawaited(_showWakeupBalloon());
    });

    try {
      if (Platform.isWindows || Platform.isMacOS) {
        await _appWindow.applyEffect(dark: _isMicaDark(_config.themeMode));
      }
    } catch (e) {
      AppLogger.error('applyEffect in _initShell failed: $e');
    }

    try {
      if (!Platform.isMacOS || _config.showTrayIcon) {
        await _trayIcon.init();
      }
    } catch (e) {
      AppLogger.error('trayIcon.init failed: $e');
    }

    if (Platform.isWindows && !isFirstRun && _config.hasSeenWindowsOnboarding) {
      WidgetsBinding.instance.addPostFrameCallback(
        (_) => unawaited(_showStartupBalloon()),
      );
    }

    await _registerHotkeyWithFeedback();

    if (Platform.isMacOS) {
      if (!macosGranted) {
        setState(() => _showPermissionGate = true);
        await _appWindow.enterGateMode();
      } else {
        if (!_config.accessibilityWasGranted) {
          _config = _config.copyWith(accessibilityWasGranted: true);
          unawaited(
            _config.save('${widget.storage.configPath}/${AppConfig.fileName}'),
          );
        }
        if (isFirstRun) {
          widget.storage.markAsInitialized();
        }
      }
    } else {
      final isUpdate = _config.lastRunVersion != AppConfig.appVersion;
      final shouldShowOnboarding =
          Platform.isWindows && (!_config.hasSeenWindowsOnboarding || isUpdate);
      if (shouldShowOnboarding) {
        if (isFirstRun) widget.storage.markAsInitialized();
        setState(() => _showWindowsOnboarding = true);
        await _appWindow.enterGateMode();
      } else if (isFirstRun) {
        widget.storage.markAsInitialized();
      }
      if (isUpdate && Platform.isWindows) {
        _config = _config.copyWith(lastRunVersion: AppConfig.appVersion);
        unawaited(
          _config.save('${widget.storage.configPath}/${AppConfig.fileName}'),
        );
      }
    }

    AutoUpdateService.onUpdateAvailable = _onUpdateAvailable;
    unawaited(AutoUpdateService.initialize());
    if (_needsClassifierMigration(_config.lastRunVersion)) {
      unawaited(_runClassifierMigration());
    }
  }

  Future<void> _runClassifierMigration() async {
    try {
      await widget.clipboardService.reclassifyLegacyTextItems();
    } catch (e, s) {
      AppLogger.error('Classifier migration failed: $e\n$s');
      return; // version not saved → retries on next startup
    }
    _config = _config.copyWith(lastRunVersion: AppConfig.appVersion);
    unawaited(
      _config.save('${widget.storage.configPath}/${AppConfig.fileName}'),
    );
  }

  static bool _needsClassifierMigration(String lastVersion) {
    if (lastVersion.isEmpty) return true;
    final parts = lastVersion.split('.');
    if (parts.length < 3) return true;
    final major = int.tryParse(parts[0]) ?? 0;
    final minor = int.tryParse(parts[1]) ?? 0;
    final patch = int.tryParse(parts[2]) ?? 0;
    if (major < 2) return true;
    if (major == 2 && minor < 1) return true;
    if (major == 2 && minor == 1 && patch <= 5) return true;
    return false;
  }

  Future<void> _registerHotkeyWithFeedback() async {
    if (!Platform.isLinux) {
      await _hotkeyHandler.registerWithFallback();
      return;
    }

    // Wayland is blocked before this point in _initShell — only X11 reaches here.
    final result = await _hotkeyHandler.registerWithFallback();
    if (result.status == HotkeyRegistrationStatus.fallbackRegistered) {
      AppLogger.info(
        'Primary Linux hotkey failed, using temporary fallback: '
        '${result.requestedBinding.label()} -> '
        '${result.effectiveBinding?.label()}',
      );
      _showLinuxNotice(
        (l) => l.linuxHotkeyFallbackWarning(
          result.requestedBinding.label(),
          result.effectiveBinding?.label() ??
              kLinuxTemporaryFallbackHotkey.label(),
        ),
      );
      return;
    }

    if (result.status == HotkeyRegistrationStatus.failed) {
      AppLogger.error(
        'Linux hotkey registration failed for ${result.requestedBinding.label()}',
      );
      _showLinuxNotice(
        (l) => l.linuxHotkeyConflictWarning(
          result.requestedBinding.label(),
          kLinuxTemporaryFallbackHotkey.label(),
        ),
      );
    }
  }

  void _showLinuxNotice(String Function(AppLocalizations l) messageBuilder) {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      final ctx = _navigatorKey.currentContext;
      if (ctx == null || !ctx.mounted) return;
      final messenger = ScaffoldMessenger.maybeOf(ctx);
      if (messenger == null) return;

      messenger.showSnackBar(
        SnackBar(
          content: Text(messageBuilder(AppLocalizations.of(ctx))),
          duration: const Duration(seconds: 12),
        ),
      );
    });
  }

  void _startListening() {
    if (!Platform.isWindows && !Platform.isMacOS && !Platform.isLinux) return;
    _listenerSubscription = widget.listener.onEvent.listen(
      _onClipboardEvent,
      onError: (Object e, StackTrace s) {
        AppLogger.error('Clipboard listener error: $e\n$s');
        // Re-subscribe so a single error does not permanently stop capturing.
        _listenerSubscription?.cancel();
        _startListening();
      },
      cancelOnError: false,
    );
  }

  Future<void> _onClipboardEvent(ClipboardEvent event) async {
    for (var attempt = 0; attempt < 3; attempt++) {
      try {
        await _processClipboardEvent(event);
        return;
      } catch (e, s) {
        if (attempt < 2) {
          await Future<void>.delayed(
            Duration(milliseconds: 500 * (attempt + 1)),
          );
        } else {
          AppLogger.error('Clipboard event failed after 3 retries: $e\n$s');
        }
      }
    }
  }

  Future<void> _processClipboardEvent(ClipboardEvent event) async {
    switch (event.type) {
      case ClipboardContentType.text:
      case ClipboardContentType.link:
        await widget.clipboardService.processText(
          event.text ?? '',
          event.type,
          source: event.source,
          rtfBytes: event.rtfBytes,
          htmlBytes: event.htmlBytes,
        );
      case ClipboardContentType.image:
        if (event.bytes != null && event.bytes!.isNotEmpty) {
          await widget.clipboardService.processImage(
            event.contentHash,
            source: event.source,
            imageBytes: event.bytes,
          );
        } else if (event.files != null && event.files!.isNotEmpty) {
          final item = await widget.clipboardService.processImage(
            event.contentHash,
            source: event.source,
            imagePath: event.files!.first,
          );
          if (item != null) {
            unawaited(_processMediaMetadata(item, event.files!.first));
          }
        }
      case ClipboardContentType.file:
      case ClipboardContentType.folder:
        if (event.files != null && event.files!.isNotEmpty) {
          await widget.clipboardService.processFiles(
            event.files!,
            event.type,
            source: event.source,
          );
        }
      case ClipboardContentType.audio:
      case ClipboardContentType.video:
        if (event.files != null && event.files!.isNotEmpty) {
          final item = await widget.clipboardService.processFiles(
            event.files!,
            event.type,
            source: event.source,
          );
          if (item != null) {
            unawaited(_processMediaMetadata(item, event.files!.first));
          }
        }
      case ClipboardContentType.email:
      case ClipboardContentType.phone:
      case ClipboardContentType.color:
      case ClipboardContentType.ip:
      case ClipboardContentType.uuid:
      case ClipboardContentType.json:
      case ClipboardContentType.unknown:
        break;
    }
  }

  Future<void> _processMediaMetadata(
    ClipboardItem item,
    String filePath,
  ) async {
    try {
      final meta = <String, Object>{};
      if (item.metadata != null && item.metadata!.isNotEmpty) {
        final existing = jsonDecode(item.metadata!) as Map<String, dynamic>;
        existing.forEach((k, v) {
          if (v != null) meta[k] = v as Object;
        });
      }

      final mediaInfo = await ClipboardWriter.getMediaInfo(filePath);
      if (mediaInfo != null) {
        mediaInfo.forEach((k, v) {
          if (v != null) meta[k] = v;
        });
      }

      if (meta.isNotEmpty) {
        await widget.clipboardService.updateMetadata(item.id, jsonEncode(meta));
      }
    } catch (e, s) {
      AppLogger.error('Media metadata failed: $e\n$s');
    }
  }

  /// Shows the window safely — errors from Mica/acrylic effects are logged
  /// but never propagate to callers (e.g. the wakeup signal callback).
  Future<void> _safeShow() async {
    try {
      await _appWindow.show();
    } catch (e) {
      AppLogger.error('show failed: $e');
    }
  }

  Future<void> _showWakeupBalloon() async {
    final binding = HotkeyBinding(
      virtualKey: _config.hotkeyVirtualKey,
      keyName: _config.hotkeyKeyName,
      useCtrl: _config.hotkeyUseCtrl,
      useWin: _config.hotkeyUseWin,
      useAlt: _config.hotkeyUseAlt,
      useShift: _config.hotkeyUseShift,
    );
    final ctx = _navigatorKey.currentContext;
    final l = ctx != null && ctx.mounted ? AppLocalizations.of(ctx) : null;
    await WindowsBalloon.show(
      title: l?.balloonWakeupTitle ?? 'CopyPaste is already open',
      body:
          l?.balloonWakeupBody(binding.label()) ??
          'Press ${binding.label()} or click the tray icon to bring it up.',
    );
  }

  Future<void> _showStartupBalloon() async {
    final binding = HotkeyBinding(
      virtualKey: _config.hotkeyVirtualKey,
      keyName: _config.hotkeyKeyName,
      useCtrl: _config.hotkeyUseCtrl,
      useWin: _config.hotkeyUseWin,
      useAlt: _config.hotkeyUseAlt,
      useShift: _config.hotkeyUseShift,
    );
    final ctx = _navigatorKey.currentContext;
    final l = ctx != null && ctx.mounted ? AppLocalizations.of(ctx) : null;
    await WindowsBalloon.show(
      title: 'CopyPaste',
      body:
          l?.balloonStartupBody(binding.label()) ??
          'Running in the background. Press ${binding.label()} or click the tray icon.',
    );
  }

  Future<void> _onHotkey() async {
    _programmaticRestore = true;
    if (!_appWindow.isVisible) {
      await _focusManager.capturePreviousWindow();
    }
    await _appWindow.toggle();
    _programmaticRestore = false; // fallback if onWindowRestore never fires
  }

  void _dismissHint() {
    if (_config.hasSeenHint) return;
    _config = _config.copyWith(hasSeenHint: true);
    unawaited(
      _config.save('${widget.storage.configPath}/${AppConfig.fileName}'),
    );
    if (mounted) setState(() {});
  }

  Future<void> _toggleWindow() async {
    _programmaticRestore = true;
    await _appWindow.toggle();
    _programmaticRestore = false; // fallback if onWindowRestore never fires
  }

  void _onWindowVisibilityChanged(bool visible) {
    if (visible) {
      _mainScreenKey.currentState?.onWindowShow();
    } else {
      _mainScreenKey.currentState?.onWindowHide();
      final ctx = _navigatorKey.currentContext;
      if (ctx != null && ctx.mounted) {
        ScaffoldMessenger.maybeOf(ctx)?.clearSnackBars();
      }
    }
  }

  Future<void> _onPasteItem(
    ClipboardItem item, {
    bool plainText = false,
  }) async {
    if (item.isFileBasedType && !item.isFileAvailable()) return;
    await widget.clipboardService.notifyPasteInitiated(item.id);
    await widget.clipboardService.recordPaste(item.id);
    final ok = await ClipboardWriter.setFromItem(
      typeValue: item.type.value,
      content: item.content,
      metadata: item.metadata,
      plainText: plainText,
    );
    if (!ok) return;
    await _appWindow.hide();
    try {
      await _focusManager.restoreAndPaste(
        delayBeforeFocusMs: _config.delayBeforeFocusMs,
        maxFocusVerifyAttempts: _config.maxFocusVerifyAttempts,
        delayBeforePasteMs: _config.delayBeforePasteMs,
      );
    } on PlatformException catch (e) {
      if (e.code == 'ACCESSIBILITY_DENIED' && mounted) {
        _enterPermissionGate();
      }
    }
  }

  Future<void> _cleanup() async {
    SingleInstance.stopListening();
    try {
      await _listenerSubscription?.cancel();
    } catch (e) {
      AppLogger.error('cleanup listener: $e');
    }
    try {
      await _hotkeyHandler.unregister();
    } catch (e) {
      AppLogger.error('cleanup hotkey: $e');
    }
    try {
      await _trayIcon.dispose();
    } catch (e) {
      AppLogger.error('cleanup tray: $e');
    }
    if (Platform.isLinux) {
      try {
        await LinuxShell.dispose();
      } catch (e) {
        AppLogger.error('cleanup linux shell: $e');
      }
    }
    try {
      widget.clipboardService.dispose();
    } catch (e) {
      AppLogger.error('cleanup clipboard: $e');
    }
    try {
      widget.cleanupService.dispose();
    } catch (e) {
      AppLogger.error('cleanup cleanup: $e');
    }
    try {
      await widget.repo.close();
    } catch (e) {
      AppLogger.error('cleanup repo: $e');
    }
  }

  Future<void> _exitApp() async {
    await _cleanup();
    SingleInstance.release();
    exit(0);
  }

  /// Resets config and first-run flag, preserves clipboard history, restarts.
  Future<void> _softReset() async {
    await _cleanup();
    SingleInstance.release();
    try {
      // Remove config so next run starts with defaults
      final configFile = File(
        '${widget.storage.configPath}/${AppConfig.fileName}',
      );
      if (configFile.existsSync()) configFile.deleteSync();
      // Remove .initialized so first-run onboarding shows again
      widget.storage.clearInitialized();
    } catch (e) {
      AppLogger.error('softReset file cleanup: $e');
    }
    await Process.start(
      Platform.resolvedExecutable,
      [],
      mode: ProcessStartMode.detached,
    );
    exit(0);
  }

  /// Deletes all data (db, images, config, first-run flag), restarts.
  Future<void> _hardReset() async {
    await _cleanup();
    SingleInstance.release();
    try {
      final base = Directory(widget.storage.baseDir);
      if (base.existsSync()) base.deleteSync(recursive: true);
    } catch (e) {
      AppLogger.error('hardReset dir cleanup: $e');
    }
    await Process.start(
      Platform.resolvedExecutable,
      [],
      mode: ProcessStartMode.detached,
    );
    exit(0);
  }

  Future<void> _openSettings(BuildContext ctx) async {
    await _appWindow.enterSettingsMode();
    if (!ctx.mounted) return;
    await Navigator.of(ctx).push(
      PageRouteBuilder<void>(
        pageBuilder: (context, animation, secondaryAnimation) => SettingsScreen(
          config: _config,
          configPath: '${widget.storage.configPath}/${AppConfig.fileName}',
          clipboardService: widget.clipboardService,
          storage: widget.storage,
          onSoftReset: _softReset,
          onHardReset: _hardReset,
          onSave: (newConfig, hotkeyChanged) async {
            final oldShowTray = _config.showTrayIcon;
            setState(() => _config = newConfig);
            widget.cleanupService.updateRetentionCallback(
              () => newConfig.retentionDays,
            );
            widget.clipboardService.pasteIgnoreWindowMs =
                newConfig.duplicateIgnoreWindowMs;
            _appWindow.updatePopupSize(
              newConfig.popupWidth.toDouble(),
              newConfig.popupHeight.toDouble(),
            );
            _appWindow.showInTaskbar = newConfig.showInTaskbar;
            if (Platform.isWindows || Platform.isMacOS) {
              await _appWindow.applyEffect(
                dark: _isMicaDark(newConfig.themeMode),
              );
            }
            if (hotkeyChanged) {
              await _hotkeyHandler.unregister();
              _hotkeyHandler = HotkeyHandler(
                config: newConfig,
                onHotkey: _onHotkey,
              );
              await _registerHotkeyWithFeedback();
            }
            if (Platform.isMacOS && newConfig.showTrayIcon != oldShowTray) {
              if (newConfig.showTrayIcon) {
                await _trayIcon.init();
              } else {
                await _trayIcon.dispose();
              }
            }
          },
        ),
        transitionsBuilder: (context, animation, secondaryAnimation, child) =>
            child,
        transitionDuration: Duration.zero,
        reverseTransitionDuration: Duration.zero,
      ),
    );
    await _appWindow.exitSettingsMode();
  }

  @override
  void onWindowBlur() {
    if (!_appWindow.isReady || !_appWindow.isVisible) return;
    if (_appWindow.isGateMode) return;
    if (!_config.hideOnDeactivate) return;
    unawaited(_appWindow.hideIfNotPinned());
  }

  @override
  void onWindowClose() {
    _appWindow.hide();
  }

  @override
  void onWindowRestore() {
    if (!_config.showInTaskbar || !Platform.isWindows) return;
    if (_programmaticRestore) {
      _programmaticRestore = false; // consume the flag on the first event
      return;
    }
    // Native user click on the taskbar button
    unawaited(_safeShow());
    _showTaskbarOpenHint();
  }

  void _showTaskbarOpenHint() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      final ctx = _navigatorKey.currentContext;
      if (ctx == null || !ctx.mounted) return;
      if (_navigatorKey.currentState?.canPop() ?? false) return;
      final messenger = ScaffoldMessenger.maybeOf(ctx);
      if (messenger == null) return;
      final binding = HotkeyBinding(
        virtualKey: _config.hotkeyVirtualKey,
        keyName: _config.hotkeyKeyName,
        useCtrl: _config.hotkeyUseCtrl,
        useWin: _config.hotkeyUseWin,
        useAlt: _config.hotkeyUseAlt,
        useShift: _config.hotkeyUseShift,
      );
      messenger
        ..clearSnackBars()
        ..showSnackBar(
          SnackBar(
            content: Text(
              AppLocalizations.of(ctx).taskbarOpenHint(binding.label()),
            ),
            duration: const Duration(seconds: 4),
          ),
        );
    });
  }

  void _enterPermissionGate() {
    setState(() => _showPermissionGate = true);
    _appWindow.enterGateMode();
  }

  Future<void> _onPermissionGranted() async {
    _config = _config.copyWith(accessibilityWasGranted: true);
    unawaited(
      _config.save('${widget.storage.configPath}/${AppConfig.fileName}'),
    );
    await _appWindow.exitGateMode();
    if (mounted) setState(() => _showPermissionGate = false);
  }

  Future<void> _onOnboardingDismissed() async {
    _config = _config.copyWith(
      hasSeenWindowsOnboarding: true,
      lastRunVersion: AppConfig.appVersion,
    );
    unawaited(
      _config.save('${widget.storage.configPath}/${AppConfig.fileName}'),
    );
    setState(() => _showWindowsOnboarding = false);
    await _appWindow.exitGateMode();
    unawaited(_showStartupBalloon());
  }

  Future<void> _onOnboardingGoSettings(BuildContext ctx) async {
    _config = _config.copyWith(
      hasSeenWindowsOnboarding: true,
      lastRunVersion: AppConfig.appVersion,
    );
    unawaited(
      _config.save('${widget.storage.configPath}/${AppConfig.fileName}'),
    );
    setState(() => _showWindowsOnboarding = false);
    await _appWindow.exitGateMode();
    await Future<void>.delayed(const Duration(milliseconds: 150));
    if (ctx.mounted) await _openSettings(ctx);
    unawaited(_showStartupBalloon());
  }

  Future<void> _restartApp() async {
    await _cleanup();
    SingleInstance.release();
    await Process.start(
      Platform.resolvedExecutable,
      [],
      mode: ProcessStartMode.detached,
    );
    exit(0);
  }

  void _onUpdateAvailable(String version) {
    if (!mounted) return;
    setState(() => _availableUpdateVersion = version);
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    windowManager.removeListener(this);
    AutoUpdateService.dispose();
    unawaited(_cleanup());
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return CopyPasteTheme(
      themeData: CompactTheme(),
      child: MaterialApp(
        navigatorKey: _navigatorKey,
        title: 'CopyPaste',
        debugShowCheckedModeBanner: false,
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        locale: _config.preferredLanguage == 'auto'
            ? null
            : Locale(_config.preferredLanguage),
        theme: ThemeData(
          colorScheme: ColorScheme.fromSeed(seedColor: const Color(0xFF4F46E5)),
          scaffoldBackgroundColor: Colors.transparent,
          fontFamily: 'Inter',
          useMaterial3: true,
        ),
        darkTheme: ThemeData(
          colorScheme: ColorScheme.fromSeed(
            seedColor: const Color(0xFF4F46E5),
            brightness: Brightness.dark,
          ),
          scaffoldBackgroundColor: Colors.transparent,
          fontFamily: 'Inter',
          useMaterial3: true,
        ),
        themeMode: switch (_config.themeMode) {
          'dark' => ThemeMode.dark,
          'auto' || 'system' => ThemeMode.system,
          _ => ThemeMode.light,
        },
        home: Builder(
          builder: (ctx) {
            final l = AppLocalizations.of(ctx);
            final currentLocale = Localizations.localeOf(ctx).toString();
            if (_lastTrayLocale != currentLocale) {
              _lastTrayLocale = currentLocale;
              if (!Platform.isMacOS || _config.showTrayIcon) {
                unawaited(
                  _trayIcon.rebuild(
                    showHideLabel: l.trayShowHide,
                    exitLabel: l.trayExit,
                    tooltip: l.trayTooltip,
                  ),
                );
              }
            }

            if (_showWaylandUnsupported) {
              return WaylandUnsupportedScreen(
                onClose: () => unawaited(_exitApp()),
              );
            }

            if (_showWindowsOnboarding) {
              final binding = HotkeyBinding(
                virtualKey: _config.hotkeyVirtualKey,
                keyName: _config.hotkeyKeyName,
                useCtrl: _config.hotkeyUseCtrl,
                useWin: _config.hotkeyUseWin,
                useAlt: _config.hotkeyUseAlt,
                useShift: _config.hotkeyUseShift,
              );
              return WindowsOnboardingScreen(
                hotkey: binding.label(),
                onDismiss: () => unawaited(_onOnboardingDismissed()),
                onSettings: () => unawaited(_onOnboardingGoSettings(ctx)),
              );
            }

            if (_showPermissionGate) {
              return PermissionGateScreen(
                previouslyGranted: _config.accessibilityWasGranted,
                onGranted: _onPermissionGranted,
                onRestart: _restartApp,
              );
            }

            final bg = (Platform.isWindows || Platform.isMacOS)
                ? CopyPasteTheme.colorsOf(
                    ctx,
                  ).background.withValues(alpha: 0.85)
                : CopyPasteTheme.colorsOf(ctx).background;
            return Scaffold(
              backgroundColor: bg,
              body: LayoutBuilder(
                builder: (_, constraints) {
                  if (constraints.maxHeight < 100) {
                    return const SizedBox.shrink();
                  }
                  return MainScreen(
                    key: _mainScreenKey,
                    clipboardService: widget.clipboardService,
                    colorLabels: _config.colorLabels,
                    resetScrollOnShow: _config.resetScrollOnShow,
                    resetSearchOnShow: _config.resetSearchOnShow,
                    cardMinLines: _config.cardMinLines,
                    cardMaxLines: _config.cardMaxLines,
                    showHint: !_config.hasSeenHint,
                    onDismissHint: _dismissHint,
                    onPaste: _onPasteItem,
                    onPastePlain: (item) => _onPasteItem(item, plainText: true),
                    onExit: () => _appWindow.hide(),
                    onSettings: () => _openSettings(ctx),
                    updateVersion: _availableUpdateVersion,
                  );
                },
              ),
            );
          },
        ),
      ),
    );
  }
}
