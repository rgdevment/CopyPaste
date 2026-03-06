import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:ui' show PlatformDispatcher;

import 'package:core/core.dart';
import 'package:flutter/material.dart';
import 'package:flutter_acrylic/flutter_acrylic.dart';
import 'package:listener/listener.dart';
import 'package:window_manager/window_manager.dart';

import 'services/auto_update_service.dart';

import 'shell/app_window.dart';
import 'shell/focus_manager.dart';
import 'shell/hotkey_handler.dart';
import 'shell/single_instance.dart';
import 'shell/startup_helper.dart';
import 'shell/tray_icon.dart';
import 'screens/main_screen.dart';
import 'screens/settings_screen.dart';
import 'theme/compact_theme.dart';
import 'theme/theme_provider.dart';
import 'l10n/app_localizations.dart';

bool _isMicaDark(String themeMode) => switch (themeMode) {
  'dark' => true,
  'auto' ||
  'system' => PlatformDispatcher.instance.platformBrightness == Brightness.dark,
  _ => false,
};

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await windowManager.ensureInitialized();

  if (Platform.isWindows) {
    await Window.initialize();
  }

  if (!SingleInstance.acquire()) {
    exit(0);
  }

  final storage = await StorageConfig.create();
  await storage.ensureDirectories();
  AppLogger.initialize('${storage.baseDir}/logs');

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

  final listener = WindowsClipboardListener();

  await StartupHelper.apply(config.runOnStartup);

  if (Platform.isWindows) {
    await Window.setEffect(
      effect: WindowEffect.mica,
      color: const Color(0x00000000),
      dark: _isMicaDark(config.themeMode),
    );
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
  final WindowsClipboardListener listener;

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
  StreamSubscription<ClipboardEvent>? _listenerSubscription;
  String? _lastTrayLocale;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    _config = widget.config;
    _appWindow = AppWindow(
      onVisibilityChanged: _onWindowVisibilityChanged,
      popupWidth: _config.popupWidth.toDouble(),
      popupHeight: _config.popupHeight.toDouble(),
    );
    _trayIcon = TrayIcon(onToggle: _toggleWindow, onExit: _exitApp);
    _hotkeyHandler = HotkeyHandler(config: _config, onHotkey: _onHotkey);

    _initShell();
  }

  @override
  void didChangePlatformBrightness() {
    if (_config.themeMode == 'auto' && Platform.isWindows) {
      _appWindow.applyMica(dark: _isMicaDark('auto'));
    }
  }

  Future<void> _initShell() async {
    windowManager.addListener(this);
    final isFirstRun = widget.storage.isFirstRun;
    await _appWindow.init();
    if (Platform.isWindows) {
      await _appWindow.applyMica(dark: _isMicaDark(_config.themeMode));
    }
    await _trayIcon.init();
    await _hotkeyHandler.registerWithFallback();
    if (isFirstRun) {
      widget.storage.markAsInitialized();
      await _appWindow.show();
    }
    _startListening();
    unawaited(AutoUpdateService.initialize());
  }

  void _startListening() {
    if (!Platform.isWindows) return;
    _listenerSubscription = widget.listener.onEvent.listen(_onClipboardEvent);
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

  Future<void> _onHotkey() async {
    _focusManager.capturePreviousWindow();
    await _appWindow.toggle();
  }

  void _dismissHint() {
    if (_config.hasSeenHint) return;
    _config = _config.copyWith(hasSeenHint: true);
    _config.save('${widget.storage.configPath}/${AppConfig.fileName}');
    if (mounted) setState(() {});
  }

  Future<void> _toggleWindow() async {
    await _appWindow.toggle();
  }

  void _onWindowVisibilityChanged(bool visible) {
    if (visible) {
      _mainScreenKey.currentState?.onWindowShow();
    } else {
      _mainScreenKey.currentState?.onWindowHide();
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
    await _focusManager.restoreAndPaste(
      delayBeforeFocusMs: _config.delayBeforeFocusMs,
      maxFocusVerifyAttempts: _config.maxFocusVerifyAttempts,
      delayBeforePasteMs: _config.delayBeforePasteMs,
    );
  }

  Future<void> _cleanup() async {
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
          onSave: (newConfig, hotkeyChanged) async {
            setState(() => _config = newConfig);
            _appWindow.updatePopupSize(
              newConfig.popupWidth.toDouble(),
              newConfig.popupHeight.toDouble(),
            );
            if (Platform.isWindows) {
              await _appWindow.applyMica(
                dark: _isMicaDark(newConfig.themeMode),
              );
            }
            if (hotkeyChanged) {
              await _hotkeyHandler.unregister();
              _hotkeyHandler = HotkeyHandler(
                config: newConfig,
                onHotkey: _onHotkey,
              );
              await _hotkeyHandler.registerWithFallback();
            }
          },
        ),
        transitionsBuilder: (context, animation, secondaryAnimation, child) =>
            FadeTransition(opacity: animation, child: child),
        transitionDuration: const Duration(milliseconds: 150),
      ),
    );
    await _appWindow.exitSettingsMode();
  }

  @override
  void onWindowBlur() {
    if (!_appWindow.isReady || !_appWindow.isVisible) return;
    if (!_config.hideOnDeactivate) return;
    _appWindow.hideIfNotPinned();
  }

  @override
  void onWindowClose() {
    _appWindow.hide();
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    windowManager.removeListener(this);
    unawaited(_cleanup());
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return CopyPasteTheme(
      themeData: CompactTheme(),
      child: MaterialApp(
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
              unawaited(
                _trayIcon.rebuild(
                  showHideLabel: l.trayShowHide,
                  exitLabel: l.trayExit,
                  tooltip: l.trayTooltip,
                ),
              );
            }
            final bg = Platform.isWindows
                ? CopyPasteTheme.colorsOf(
                    ctx,
                  ).background.withValues(alpha: 0.85)
                : CopyPasteTheme.colorsOf(ctx).background;
            return Scaffold(
              backgroundColor: bg,
              body: MainScreen(
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
              ),
            );
          },
        ),
      ),
    );
  }
}
