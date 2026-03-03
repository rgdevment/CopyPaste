import 'dart:async';
import 'dart:io';

import 'package:core/core.dart';
import 'package:flutter/material.dart';
import 'package:listener/listener.dart';
import 'package:window_manager/window_manager.dart';

import 'shell/app_window.dart';
import 'shell/focus_manager.dart';
import 'shell/hotkey_handler.dart';
import 'shell/startup_helper.dart';
import 'shell/tray_icon.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await windowManager.ensureInitialized();

  final storage = await StorageConfig.create();
  await storage.ensureDirectories();

  AppLogger.initialize('${storage.baseDir}/logs');

  final config = await AppConfig.load(
    '${storage.configPath}/${AppConfig.fileName}',
  );

  final repo = SqliteRepository.fromPath(storage.databasePath);
  final clipboardService = ClipboardService(repo, imagesPath: storage.imagesPath)
    ..pasteIgnoreWindowMs = config.duplicateIgnoreWindowMs
    ..setThumbnailsPath(storage.thumbnailsPath);

  final cleanupService = CleanupService(
    repo,
    () => config.retentionDays,
    storage: storage,
  )..start(storage.baseDir);

  final listener = WindowsClipboardListener();
  final updateChecker = UpdateChecker(configPath: storage.configPath)
    ..start('2.0.0');

  await StartupHelper.apply(config.runOnStartup);

  runApp(
    CopyPasteApp(
      storage: storage,
      config: config,
      repo: repo,
      clipboardService: clipboardService,
      cleanupService: cleanupService,
      listener: listener,
      updateChecker: updateChecker,
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
    required this.updateChecker,
    super.key,
  });

  final StorageConfig storage;
  final AppConfig config;
  final SqliteRepository repo;
  final ClipboardService clipboardService;
  final CleanupService cleanupService;
  final WindowsClipboardListener listener;
  final UpdateChecker updateChecker;

  @override
  State<CopyPasteApp> createState() => _CopyPasteAppState();
}

class _CopyPasteAppState extends State<CopyPasteApp> with WindowListener {
  late final AppWindow _appWindow;
  late final TrayIcon _trayIcon;
  late final HotkeyHandler _hotkeyHandler;
  final WindowFocusManager _focusManager = WindowFocusManager();
  StreamSubscription<ClipboardEvent>? _listenerSubscription;

  @override
  void initState() {
    super.initState();
    _appWindow = AppWindow();
    _trayIcon = TrayIcon(
      onToggle: _toggleWindow,
      onExit: _exitApp,
    );
    _hotkeyHandler = HotkeyHandler(
      config: widget.config,
      onHotkey: _onHotkey,
    );

    _initShell();
  }

  Future<void> _initShell() async {
    await _appWindow.init();
    await _trayIcon.init();
    await _hotkeyHandler.registerWithFallback();
    windowManager.addListener(this);

    _startListening();
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
      } catch (_) {
        if (attempt < 2) {
          await Future<void>.delayed(
            Duration(milliseconds: 100 * (attempt + 1)),
          );
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
        await widget.clipboardService.processImage(
          event.contentHash,
          source: event.source,
          imageBytes: event.bytes,
        );
      case ClipboardContentType.file:
      case ClipboardContentType.folder:
      case ClipboardContentType.audio:
      case ClipboardContentType.video:
        if (event.files != null && event.files!.isNotEmpty) {
          await widget.clipboardService.processFiles(
            event.files!,
            event.type,
            source: event.source,
          );
        }
      case ClipboardContentType.unknown:
        break;
    }
  }

  Future<void> _onHotkey() async {
    _focusManager.capturePreviousWindow();
    await _appWindow.toggle();
  }

  Future<void> _toggleWindow() async {
    await _appWindow.toggle();
  }

  Future<void> _exitApp() async {
    try { await _listenerSubscription?.cancel(); } catch (_) {}
    try { await _hotkeyHandler.unregister(); } catch (_) {}
    try { await _trayIcon.dispose(); } catch (_) {}
    try { widget.clipboardService.dispose(); } catch (_) {}
    try { widget.cleanupService.dispose(); } catch (_) {}
    try { widget.updateChecker.dispose(); } catch (_) {}
    try { await widget.repo.close(); } catch (_) {}
    exit(0);
  }

  @override
  void onWindowBlur() {
    _appWindow.hideIfNotPinned();
  }

  @override
  void onWindowClose() {
    _exitApp();
  }

  @override
  void dispose() {
    windowManager.removeListener(this);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'CopyPaste',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.blue),
        useMaterial3: true,
      ),
      darkTheme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: Colors.blue,
          brightness: Brightness.dark,
        ),
        useMaterial3: true,
      ),
      themeMode: ThemeMode.system,
      home: const Scaffold(
        body: Center(
          child: Text('CopyPaste v2 — Shell Ready'),
        ),
      ),
    );
  }
}

