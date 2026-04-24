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
import 'package:path/path.dart' as p;
import 'package:window_manager/window_manager.dart';

import 'services/auto_update_service.dart';
import 'services/install_channel.dart';
import 'services/linux_capabilities.dart';
import 'services/release_manifest_service.dart';

import 'shell/app_window.dart';
import 'shell/focus_manager.dart';
import 'shell/hotkey_handler.dart';
import 'shell/linux_hotkey_registration.dart';
import 'shell/linux_session.dart';
import 'shell/linux_shell.dart';
import 'shell/single_instance.dart';
import 'shell/startup_helper.dart';
import 'shell/tray_icon.dart';
import 'shell/win_known_folders.dart';
import 'shell/win_package_context.dart';
import 'shell/windows_balloon.dart';
import 'screens/main_screen.dart';
import 'screens/settings_screen.dart';
import 'screens/wayland_unsupported_screen.dart';
import 'theme/compact_theme.dart';
import 'theme/theme_provider.dart';
import 'l10n/app_localizations.dart';
import 'screens/permission_gate_screen.dart';
import 'screens/windows_onboarding_screen.dart';
import 'screens/blocked_version_screen.dart';

// Re-exported so existing tests can import isWaylandSession from main.dart.
export 'shell/linux_session.dart' show isWaylandSession;

bool _isMicaDark(String themeMode) => switch (themeMode) {
  'dark' => true,
  'auto' ||
  'system' => PlatformDispatcher.instance.platformBrightness == Brightness.dark,
  _ => false,
};

void main() async {
  await runZonedGuarded<Future<void>>(_run, (error, stack) {
    CrashLogger.report(error, stack, context: 'zoneGuarded');
    AppLogger.error('Zone unhandled: $error\n$stack');
  });
}

Future<void> _run() async {
  try {
    WidgetsFlutterBinding.ensureInitialized();

    if (!SingleInstance.acquire()) {
      exit(0);
    }

    await windowManager.ensureInitialized();

    bool acrylicInitialized = false;
    if (Platform.isWindows || Platform.isMacOS) {
      try {
        await Window.initialize().timeout(const Duration(seconds: 3));
        acrylicInitialized = true;
      } catch (e, s) {
        CrashLogger.report(e, s, context: 'Window.initialize');
      }
    }

    final storage = await StorageConfig.create(
      windowsLocalAppDataResolver: Platform.isWindows
          ? WinKnownFolders.localAppData
          : null,
    );
    await storage.ensureDirectories();
    CrashLogger.initialize(storage.baseDir);
    AppLogger.initialize(storage.logsPath);
    final isMsix = Platform.isWindows && WinPackageContext.isMsix;
    AppLogger.info(
      'Bootstrap: CopyPaste ${AppConfig.appVersion} starting '
      '(platform=${Platform.operatingSystem}, '
      'osVersion=${Platform.operatingSystemVersion}, '
      'msix=$isMsix, '
      'package=${WinPackageContext.packageFullName ?? '-'}, '
      'base=${storage.baseDir}, '
      'acrylicInit=$acrylicInitialized)',
    );

    FlutterError.onError = (details) {
      AppLogger.error(
        'FlutterError: ${details.exceptionAsString()}\n${details.stack}',
      );
      CrashLogger.report(
        details.exception,
        details.stack,
        context: 'FlutterError',
      );
    };
    PlatformDispatcher.instance.onError = (error, stack) {
      AppLogger.error('Unhandled: $error\n$stack');
      CrashLogger.report(error, stack, context: 'PlatformDispatcher');
      return false;
    };

    final config = await AppConfig.load(
      '${storage.configPath}/${AppConfig.fileName}',
    );

    final repo = SqliteRepository.fromPath(storage.databasePath);
    final NativeThumbnailProvider? nativeThumbProvider = Platform.isWindows
        ? WindowsNativeThumbnailProvider()
        : Platform.isMacOS
        ? MacOSNativeThumbnailProvider()
        : null;
    final clipboardService = ClipboardService(
      repo,
      imagesPath: storage.imagesPath,
      nativeThumbnailProvider: nativeThumbProvider,
      isThumbnailTypeEnabled: (t) => switch (t) {
        ClipboardContentType.image => config.generateImageThumbnails,
        ClipboardContentType.video => config.generateVideoThumbnails,
        ClipboardContentType.audio => config.generateAudioThumbnails,
        _ => true,
      },
      getMaxImageBytes: () => config.maxImageProcessingSizeMB * 1024 * 1024,
    )..pasteIgnoreWindowMs = config.duplicateIgnoreWindowMs;

    final cleanupService = CleanupService(
      repo,
      () => config.retentionDays,
      storage: storage,
      getKeepBrokenDays: () => config.keepBrokenItemsDays,
      getImagesQuotaMB: () => config.imagesQuotaMB,
    )..start(storage.baseDir);

    final listener = ClipboardListener();

    await StartupHelper.apply(config.runOnStartup);

    try {
      if (Platform.isWindows) {
        AppLogger.info('main: applying initial Mica effect');
        await Window.setEffect(
          effect: WindowEffect.mica,
          color: const Color(0x00000000),
          dark: _isMicaDark(config.themeMode),
        ).timeout(const Duration(seconds: 2));
        AppLogger.info('main: Mica effect applied');
      } else if (Platform.isMacOS) {
        await Window.setEffect(
          effect: WindowEffect.sidebar,
          color: const Color(0x00000000),
          dark: _isMicaDark(config.themeMode),
        ).timeout(const Duration(seconds: 2));
      }
    } catch (e) {
      AppLogger.warn('main: Window.setEffect failed (non-fatal): $e');
    }

    if (Platform.isLinux) {
      try {
        final caps = await LinuxCapabilitiesService.detect();
        AppLogger.info('main: linux capabilities $caps');
      } catch (e) {
        AppLogger.warn('main: LinuxCapabilities.detect failed (non-fatal): $e');
      }
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
  } catch (e, s) {
    CrashLogger.report(e, s, context: 'main');
    rethrow;
  }
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
  bool _linuxPrefersDark = false;
  String? _availableUpdateVersion;
  ManifestState? _manifestState;
  StreamSubscription<ManifestState?>? _manifestSub;
  bool _programmaticRestore = false;
  Timer? _blurHideTimer;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    _config = widget.config;
    _appWindow = AppWindow(
      onVisibilityChanged: _onWindowVisibilityChanged,
      showInTaskbar: false,
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
    var initCompleted = false;
    final watchdog = Timer(const Duration(seconds: 10), () {
      if (initCompleted) return;
      AppLogger.error('Watchdog: _initShell did not complete within 10s');
      CrashLogger.report(
        StateError('Init watchdog fired'),
        StackTrace.current,
        context: '_initShell watchdog',
      );
      unawaited(_forceVisibleFallback());
    });
    try {
      await _initShellBody();
    } catch (e, s) {
      AppLogger.error('_initShell crashed: $e\n$s');
      CrashLogger.report(e, s, context: '_initShell');
      await _forceVisibleFallback();
    } finally {
      initCompleted = true;
      watchdog.cancel();
    }
  }

  Future<void> _forceVisibleFallback() async {
    try {
      if (!_appWindow.isGateMode) {
        await _appWindow.enterGateMode();
      }
    } catch (e) {
      AppLogger.error('forceVisibleFallback failed: $e');
    }
  }

  Future<void> _initShellBody() async {
    windowManager.addListener(this);
    final isFirstRun = widget.storage.isFirstRun;
    final wayland = Platform.isLinux && isWaylandSession();

    if (wayland) {
      await _appWindow.init(startVisible: true);
      await _appWindow.enterGateMode();
      if (mounted) setState(() => _showWaylandUnsupported = true);
      return;
    }

    if (Platform.isLinux) {
      final isDark = await linuxPrefersDarkMode();
      if (mounted) setState(() => _linuxPrefersDark = isDark);
    }
    _startListening();

    bool macosGranted = true;
    if (Platform.isMacOS) {
      macosGranted = await ClipboardWriter.checkAccessibility();
    }

    final isUpdate = _config.lastRunVersion != AppConfig.appVersion;
    final windowsNeedsOnboarding =
        Platform.isWindows && (!_config.hasSeenWindowsOnboarding || isUpdate);
    final showOnStart =
        isFirstRun &&
            (Platform.isLinux ||
                (Platform.isMacOS && macosGranted) ||
                Platform.isWindows) ||
        windowsNeedsOnboarding;
    await _appWindow.init(startVisible: showOnStart);
    if (showOnStart && Platform.isWindows) {
      try {
        await _appWindow.enterGateMode();
      } catch (e) {
        AppLogger.error('Initial enterGateMode failed: $e');
      }
    }
    SingleInstance.listenForWakeup(() {
      if (Platform.isWindows) {
        unawaited(_showOnboardingFromWakeup());
      } else {
        unawaited(_safeShow());
      }
    });

    try {
      if (Platform.isWindows || Platform.isMacOS) {
        await _appWindow.applyEffect(dark: _isMicaDark(_config.themeMode));
      }
    } catch (e) {
      AppLogger.error('applyEffect in _initShell failed: $e');
    }

    try {
      await _trayIcon.init();
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
      final shouldShowOnboarding = windowsNeedsOnboarding;
      if (shouldShowOnboarding) {
        if (isFirstRun) widget.storage.markAsInitialized();
        if (mounted) setState(() => _showWindowsOnboarding = true);
        if (!_appWindow.isGateMode) {
          try {
            await _appWindow.enterGateMode();
          } catch (e) {
            AppLogger.error('enterGateMode (post-init) failed: $e');
          }
        }
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
    unawaited(
      AutoUpdateService.initialize(storageConfigDir: widget.storage.configPath),
    );
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
      if (result.failureReason == HotkeyFailureReason.grabFailed) {
        _showLinuxNotice(
          (l) =>
              l.linuxHotkeyGrabFailedWarning(result.requestedBinding.label()),
        );
      } else {
        _showLinuxNotice(
          (l) => l.linuxHotkeyFallbackWarning(
            result.requestedBinding.label(),
            result.effectiveBinding?.label() ??
                kLinuxTemporaryFallbackHotkey.label(),
          ),
        );
      }
      return;
    }

    if (result.status == HotkeyRegistrationStatus.failed) {
      AppLogger.error(
        'Linux hotkey registration failed for ${result.requestedBinding.label()}',
      );
      if (result.failureReason == HotkeyFailureReason.grabFailed) {
        _showLinuxNotice(
          (l) =>
              l.linuxHotkeyGrabFailedWarning(result.requestedBinding.label()),
        );
      } else {
        _showLinuxNotice(
          (l) => l.linuxHotkeyConflictWarning(
            result.requestedBinding.label(),
            kLinuxTemporaryFallbackHotkey.label(),
          ),
        );
      }
    }
  }

  ThemeMode get _effectiveThemeMode {
    final mode = _config.themeMode;
    if (Platform.isLinux && (mode == 'auto' || mode == 'system')) {
      return _linuxPrefersDark ? ThemeMode.dark : ThemeMode.light;
    }
    return switch (mode) {
      'dark' => ThemeMode.dark,
      'auto' || 'system' => ThemeMode.system,
      _ => ThemeMode.light,
    };
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

  Future<void> _showOnboardingFromWakeup() async {
    if (_showWindowsOnboarding || _appWindow.isSettingsMode) {
      try {
        await windowManager.show();
        await windowManager.focus();
      } catch (_) {}
      return;
    }
    setState(() => _showWindowsOnboarding = true);
    try {
      await _appWindow.enterGateMode();
    } catch (e) {
      AppLogger.error('enterGateMode failed on wakeup: $e');
    }
    unawaited(_showWakeupBalloon());
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

  Future<void> _updateLinuxConfig(AppConfig Function(AppConfig) update) async {
    final next = update(_config);
    if (identical(next, _config)) return;
    _config = next;
    if (mounted) setState(() {});
    await _config.save('${widget.storage.configPath}/${AppConfig.fileName}');
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
      final response = await _focusManager.restoreAndPaste(
        delayBeforeFocusMs: _config.delayBeforeFocusMs,
        maxFocusVerifyAttempts: _config.maxFocusVerifyAttempts,
        delayBeforePasteMs: _config.delayBeforePasteMs,
      );
      if (Platform.isLinux && response.isFocusTimeout) {
        _showLinuxNotice((l) => l.linuxPasteFocusTimeoutWarning);
      }
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
      await widget.clipboardService.dispose();
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
      final storage = widget.storage;
      final baseDir = Directory(storage.baseDir);
      if (baseDir.existsSync() && _isSafeToWipe(storage)) {
        baseDir.deleteSync(recursive: true);
      } else {
        AppLogger.error(
          'hardReset refused: baseDir failed safety check '
          '("${storage.baseDir}")',
        );
      }
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

  /// Safety guard for [_hardReset]. Refuses to wipe a directory that does not
  /// look like our own data folder. Requires the path to:
  ///   - be non-empty and not a filesystem root,
  ///   - end with "CopyPaste" (our fixed app folder name),
  ///   - contain at least one of our known subpaths (db, images, config, logs).
  bool _isSafeToWipe(StorageConfig storage) {
    final base = storage.baseDir;
    if (base.isEmpty) return false;
    final canonical = p.canonicalize(base);
    final parent = p.dirname(canonical);
    if (canonical == parent) return false; // filesystem root
    if (p.basename(canonical) != 'CopyPaste') return false;
    final hasOwnedChild =
        File(storage.databasePath).existsSync() ||
        Directory(storage.imagesPath).existsSync() ||
        Directory(storage.configPath).existsSync() ||
        Directory(storage.logsPath).existsSync();
    return hasOwnedChild;
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
            setState(() => _config = newConfig);
            widget.cleanupService.updateRetentionCallback(
              () => newConfig.retentionDays,
            );
            widget.cleanupService.updateKeepBrokenCallback(
              () => newConfig.keepBrokenItemsDays,
            );
            widget.cleanupService.updateImagesQuotaCallback(
              () => newConfig.imagesQuotaMB,
            );
            widget.clipboardService.updateThumbnailTypeGate(
              (t) => switch (t) {
                ClipboardContentType.image => newConfig.generateImageThumbnails,
                ClipboardContentType.video => newConfig.generateVideoThumbnails,
                ClipboardContentType.audio => newConfig.generateAudioThumbnails,
                _ => true,
              },
            );
            widget.clipboardService.updateMaxImageBytesGate(
              () => newConfig.maxImageProcessingSizeMB * 1024 * 1024,
            );
            widget.clipboardService.pasteIgnoreWindowMs =
                newConfig.duplicateIgnoreWindowMs;
            _appWindow.updatePopupSize(
              newConfig.popupWidth.toDouble(),
              newConfig.popupHeight.toDouble(),
            );
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
  void onWindowFocus() {
    _blurHideTimer?.cancel();
    _blurHideTimer = null;
  }

  @override
  void onWindowBlur() {
    if (!_appWindow.isReady || !_appWindow.isVisible) return;
    if (_appWindow.isGateMode) return;
    if (!_config.hideOnDeactivate) return;
    if (Platform.isLinux) {
      // On Linux/GTK, window-move and other WM operations briefly steal focus.
      // Delay the hide so we can cancel it if focus returns quickly (e.g. drag).
      _blurHideTimer?.cancel();
      _blurHideTimer = Timer(const Duration(milliseconds: 300), () {
        _blurHideTimer = null;
        unawaited(_appWindow.hideIfNotPinned());
      });
    } else {
      unawaited(_appWindow.hideIfNotPinned());
    }
  }

  @override
  void onWindowClose() {
    _appWindow.hide();
  }

  @override
  void onWindowRestore() {
    if (!Platform.isWindows) return;
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

  Future<void> _onOnboardingDismissed(AppConfig fromOnboarding) async {
    _config = fromOnboarding.copyWith(
      hasSeenWindowsOnboarding: true,
      hasCompletedOnboarding: true,
      lastRunVersion: AppConfig.appVersion,
    );
    _applyOnboardingPersistence();
    unawaited(
      _config.save('${widget.storage.configPath}/${AppConfig.fileName}'),
    );
    setState(() => _showWindowsOnboarding = false);
    await _appWindow.exitGateMode();
    unawaited(_showStartupBalloon());
  }

  Future<void> _onOnboardingGoSettings(
    BuildContext ctx,
    AppConfig fromOnboarding,
  ) async {
    _config = fromOnboarding.copyWith(
      hasSeenWindowsOnboarding: true,
      hasCompletedOnboarding: true,
      lastRunVersion: AppConfig.appVersion,
    );
    _applyOnboardingPersistence();
    unawaited(
      _config.save('${widget.storage.configPath}/${AppConfig.fileName}'),
    );
    setState(() => _showWindowsOnboarding = false);
    await _appWindow.exitGateMode();
    await Future<void>.delayed(const Duration(milliseconds: 150));
    if (ctx.mounted) await _openSettings(ctx);
    unawaited(_showStartupBalloon());
  }

  void _applyOnboardingPersistence() {
    widget.cleanupService.updateKeepBrokenCallback(
      () => _config.keepBrokenItemsDays,
    );
    widget.clipboardService.updateThumbnailTypeGate(
      (t) => switch (t) {
        ClipboardContentType.image => _config.generateImageThumbnails,
        ClipboardContentType.video => _config.generateVideoThumbnails,
        ClipboardContentType.audio => _config.generateAudioThumbnails,
        _ => true,
      },
    );
    widget.clipboardService.updateMaxImageBytesGate(
      () => _config.maxImageProcessingSizeMB * 1024 * 1024,
    );
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
    unawaited(_manifestSub?.cancel());
    _manifestSub = null;
    unawaited(AutoUpdateService.dispose());
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
        themeMode: _effectiveThemeMode,
        home: Builder(
          builder: (ctx) {
            final l = AppLocalizations.of(ctx);
            final currentLocale = Localizations.localeOf(ctx).toString();
            if (_lastTrayLocale != currentLocale) {
              _lastTrayLocale = currentLocale;
              unawaited(
                _trayIcon.rebuild(
                  showHideLabel: l.trayShowHide,
                  exitLabel: l.trayExit,
                  tooltip: l.trayTooltip,
                ),
              );
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
                initialConfig: _config,
                onDismiss: (updated) =>
                    unawaited(_onOnboardingDismissed(updated)),
                onSettings: (updated) =>
                    unawaited(_onOnboardingGoSettings(ctx, updated)),
              );
            }

            if (_showPermissionGate) {
              return PermissionGateScreen(
                previouslyGranted: _config.accessibilityWasGranted,
                onGranted: _onPermissionGranted,
                onRestart: _restartApp,
              );
            }

            if (_manifestState != null &&
                InstallChannelDetector.detect() != InstallChannel.msStore &&
                ReleaseManifestService.isBlocked(
                  current: AppConfig.appVersion,
                  state: _manifestState,
                )) {
              return BlockedVersionScreen(
                currentVersion: AppConfig.appVersion,
                manifest: _manifestState!.manifest,
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
                    resetFiltersOnShow: _config.resetFiltersOnShow,
                    cardMinLines: _config.cardMinLines,
                    cardMaxLines: _config.cardMaxLines,
                    showHint: !_config.hasSeenHint,
                    onDismissHint: _dismissHint,
                    onPaste: _onPasteItem,
                    onPastePlain: (item) => _onPasteItem(item, plainText: true),
                    onExit: () => _appWindow.hide(),
                    onSettings: () => _openSettings(ctx),
                    updateVersion: _availableUpdateVersion,
                    updateSeverity: ReleaseManifestService.badgeSeverity(
                      current: AppConfig.appVersion,
                      state: _manifestState,
                    ),
                    appConfig: Platform.isLinux ? _config : null,
                    linuxCapabilities: Platform.isLinux
                        ? LinuxCapabilitiesService.current
                        : null,
                    onLinuxConfigUpdate:
                        Platform.isLinux ? _updateLinuxConfig : null,
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
