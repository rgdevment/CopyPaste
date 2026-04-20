// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Spanish Castilian (`es`).
class AppLocalizationsEs extends AppLocalizations {
  AppLocalizationsEs([String locale = 'es']) : super(locale);

  @override
  String get searchPlaceholder => 'Buscar en portapapeles…';

  @override
  String get emptyState => 'No hay elementos en esta sección';

  @override
  String get emptyStateSubtitle => 'Copia algo para comenzar';

  @override
  String get hintBannerText =>
      'CopyPaste se ejecuta en segundo plano — encuéntralo en la bandeja del sistema o usa tu atajo de teclado. Personaliza tu experiencia en';

  @override
  String get hintBannerAction => 'Ajustes';

  @override
  String get settingsTitle => 'Configuración';

  @override
  String get sectionShortcuts => 'ATAJOS DE TECLADO';

  @override
  String get sectionStorage => 'ALMACENAMIENTO';

  @override
  String get settingRunOnStartup => 'Iniciar con el sistema';

  @override
  String get settingLanguage => 'Idioma de la interfaz';

  @override
  String get hotkeyWillApply => 'El atajo se aplicará de inmediato';

  @override
  String get sectionSupport => 'SOPORTE';

  @override
  String get supportExportLogs => 'Exportar registros';

  @override
  String get supportExportLogsSubtitle =>
      'Guarda un zip con registros de la app para adjuntar a un reporte. El contenido del portapapeles nunca se incluye.';

  @override
  String get supportOpenLogsFolder => 'Abrir carpeta de registros';

  @override
  String get supportOpenLogsFolderSubtitle =>
      'Explora los archivos de registro en tu gestor de archivos.';

  @override
  String get supportGitHub => 'Reportar un error en GitHub';

  @override
  String get supportExportSuccess => 'Registros guardados en Descargas.';

  @override
  String get supportShowInFiles => 'Mostrar';

  @override
  String get supportExportEmpty => 'No se encontraron archivos de registro.';

  @override
  String get supportExportError => 'Error al exportar los registros.';

  @override
  String get sectionReset => 'RESTABLECER E INSTALACIÓN LIMPIA';

  @override
  String get resetSoftLabel => 'Restablecimiento suave';

  @override
  String get resetSoftSubtitle =>
      'Restablece la configuración a los valores predeterminados y marca la app como nueva instalación. El historial del portapapeles se conserva.';

  @override
  String get resetHardLabel => 'Restablecimiento completo';

  @override
  String get resetHardSubtitle =>
      'Elimina todo el historial, imágenes y configuración. Esta acción no se puede deshacer.';

  @override
  String get resetSoftConfirmTitle => '¿Restablecimiento suave?';

  @override
  String get resetSoftConfirmMessage =>
      'Toda la configuración volverá a los valores predeterminados y la app se reiniciará como si fuera una instalación nueva. El historial del portapapeles no se eliminará.';

  @override
  String get resetHardConfirmTitle => '¿Restablecimiento completo?';

  @override
  String get resetHardConfirmMessage =>
      'Se eliminará permanentemente todo el historial, imágenes y configuración, y luego la app se reiniciará. Esta acción no se puede deshacer.';

  @override
  String get resetConfirmButton => 'Restablecer y Reiniciar';

  @override
  String get clearHistoryConfirmTitle => '¿Limpiar historial?';

  @override
  String get clearHistoryConfirmMessage =>
      'Esto eliminará permanentemente todos los elementos no anclados. Esta acción no se puede deshacer.';

  @override
  String get clearHistoryConfirmButton => 'Limpiar';

  @override
  String backupLastDate(String date) {
    return 'Último respaldo: $date';
  }

  @override
  String get backupNone => 'Aún no se ha creado un respaldo.';

  @override
  String get backupCreateLabel => 'Crear respaldo';

  @override
  String get backupRestoreLabel => 'Restaurar respaldo';

  @override
  String get backupError =>
      'Error al crear el respaldo. Verifica los permisos.';

  @override
  String get restoreDialogTitle => 'Restaurar respaldo';

  @override
  String get restoreDialogWarning =>
      'Esto reemplazará todos los datos actuales con el contenido del respaldo. ¿Continuar?';

  @override
  String get restoreFileNotFound => 'Archivo no encontrado.';

  @override
  String restoreSuccess(int count) {
    return 'Se restauraron $count elementos.';
  }

  @override
  String get restoreError =>
      'Error al restaurar. Tus datos anteriores se han preservado.';

  @override
  String get buttonSave => 'Guardar';

  @override
  String get buttonCancel => 'Cancelar';

  @override
  String get buttonReset => 'Restaurar predeterminados';

  @override
  String get menuPaste => 'Pegar';

  @override
  String get menuPastePlain => 'Pegar sin formato';

  @override
  String get menuPin => 'Anclar';

  @override
  String get menuUnpin => 'Desanclar';

  @override
  String get menuEdit => 'Editar tarjeta';

  @override
  String get menuDelete => 'Eliminar';

  @override
  String get editColorLabel => 'Color';

  @override
  String get colorRed => 'Rojo';

  @override
  String get colorGreen => 'Verde';

  @override
  String get colorPurple => 'Morado';

  @override
  String get colorYellow => 'Amarillo';

  @override
  String get colorBlue => 'Azul';

  @override
  String get colorOrange => 'Naranja';

  @override
  String get typeText => 'Texto';

  @override
  String get typeImage => 'Imagen';

  @override
  String get typeFile => 'Archivo';

  @override
  String get typeFolder => 'Carpeta';

  @override
  String get typeLink => 'Enlace';

  @override
  String get typeAudio => 'Audio';

  @override
  String get typeVideo => 'Video';

  @override
  String get typeEmail => 'Email';

  @override
  String get typePhone => 'Teléfono';

  @override
  String get typeColor => 'Color';

  @override
  String get typeIp => 'IP';

  @override
  String get typeUuid => 'UUID';

  @override
  String get typeJson => 'JSON';

  @override
  String get filterAll => 'Todo';

  @override
  String get filterPinned => 'Anclados';

  @override
  String get trayTooltip => 'CopyPaste';

  @override
  String get trayExit => 'Salir';

  @override
  String get shortcutOpenClose => 'Abrir / cerrar CopyPaste';

  @override
  String get shortcutEscape => 'Limpiar búsqueda o cerrar ventana';

  @override
  String get shortcutTab1 => 'Cambiar a pestaña Recientes';

  @override
  String get shortcutTab2 => 'Cambiar a pestaña Anclados';

  @override
  String get shortcutArrows => 'Navegar entre elementos';

  @override
  String get shortcutEnter => 'Pegar elemento seleccionado';

  @override
  String get shortcutDelete => 'Eliminar elemento seleccionado';

  @override
  String get shortcutPin => 'Anclar / Desanclar elemento';

  @override
  String get shortcutEdit => 'Editar tarjeta (etiqueta y color)';

  @override
  String get tabGeneral => 'General';

  @override
  String get tabBackupRestore => 'Respaldo';

  @override
  String get tabAppearance => 'Apariencia';

  @override
  String get tabShortcuts => 'Atajos';

  @override
  String get tabAbout => 'Acerca de';

  @override
  String get sectionLanguage => 'IDIOMA';

  @override
  String get sectionStartup => 'INICIO';

  @override
  String get sectionKeyboardShortcut => 'ATAJO DE TECLADO';

  @override
  String get sectionCategories => 'CATEGORÍAS';

  @override
  String get sectionPerformance => 'RENDIMIENTO';

  @override
  String get sectionPaste => 'PEGADO';

  @override
  String get sectionBackupRestore => 'RESPALDO Y RESTAURACIÓN';

  @override
  String get sectionAppearance => 'APARIENCIA';

  @override
  String get settingTheme => 'Tema';

  @override
  String get themeLight => 'Claro';

  @override
  String get themeDark => 'Oscuro';

  @override
  String get themeAuto => 'Auto';

  @override
  String get sectionBehavior => 'COMPORTAMIENTO';

  @override
  String get sectionAbout => 'COPYPASTE';

  @override
  String get sectionLinks => 'ENLACES';

  @override
  String get settingItemsPerPage => 'Elementos por página';

  @override
  String get settingMemoryLimit => 'Límite de memoria';

  @override
  String get settingScrollThreshold => 'Umbral de desplazamiento (px)';

  @override
  String get settingPasteSpeed => 'Velocidad de pegado';

  @override
  String get settingPanelWidth => 'Ancho del panel (px)';

  @override
  String get settingPanelHeight => 'Alto del panel (px)';

  @override
  String get settingLinesCollapsed => 'Líneas contraídas';

  @override
  String get settingLinesExpanded => 'Líneas expandidas';

  @override
  String get settingHideOnDeactivate => 'Ocultar al hacer clic fuera';

  @override
  String get settingScrollToTopOnOpen => 'Ir al inicio al abrir';

  @override
  String get settingClearSearchOnOpen => 'Limpiar búsqueda al abrir';

  @override
  String get settingShowTrayIcon => 'Mostrar icono en la bandeja';

  @override
  String get settingRetentionDaysLabel => 'Días de retención (0 = sin límite)';

  @override
  String get settingClearHistoryLabel => 'Limpiar historial del portapapeles';

  @override
  String get settingHotkeyShortcutLabel => 'Atajo para abrir/cerrar CopyPaste';

  @override
  String get subtitleStartupDesc =>
      'Se inicia en segundo plano al iniciar sesión';

  @override
  String get subtitleHideOnDeactivate =>
      'Cerrar la ventana al hacer clic fuera';

  @override
  String get subtitleScrollToTopOnOpen =>
      'Restablece el desplazamiento y selecciona el último elemento';

  @override
  String get subtitleClearSearchOnOpen => 'Borra el texto de búsqueda cada vez';

  @override
  String get subtitleShowTrayIcon =>
      'Mostrar icono en la barra de menú. Usa el atajo si está oculto';

  @override
  String get subtitlePasteSpeed => 'Ajustar tiempos de restauración y pegado';

  @override
  String get subtitleCategories =>
      'Personaliza los nombres de las categorías de color.';

  @override
  String get linkGitHub => 'Soporte y Código fuente — GitHub';

  @override
  String get linkCoffee => 'Invítame un café';

  @override
  String get editDialogTitle => 'Etiqueta y Color';

  @override
  String get editDialogHint => 'Agregar una etiqueta...';

  @override
  String get historyCleared => 'Historial limpiado';

  @override
  String backupSavedFile(String filename) {
    return 'Respaldo guardado: $filename';
  }

  @override
  String get buttonRestore => 'Restaurar';

  @override
  String get restoreCompleted => 'Restauración completada';

  @override
  String get restoreRestartRequired =>
      'Restauración completada. La app se reiniciará para aplicar los cambios.';

  @override
  String get shortcutExpand => 'Expandir / contraer tarjeta';

  @override
  String get shortcutFocusSearch => 'Enfocar el buscador';

  @override
  String get trayShowHide => 'Mostrar/Ocultar';

  @override
  String get fileNotFound => 'No encontrado';

  @override
  String get audioFile => 'Archivo de audio';

  @override
  String get videoFile => 'Archivo de video';

  @override
  String get timeNow => 'ahora';

  @override
  String get clearAllFilters => 'Limpiar todos los filtros';

  @override
  String get colorSectionLabel => 'COLOR';

  @override
  String get colorNone => 'Ninguno';

  @override
  String get subtitlePastePreset =>
      'Velocidad de pegado automático. Normal/Seguro recomendado para la mayoría.';

  @override
  String get subtitleBackup =>
      'Crea un respaldo de tu historial, imágenes y configuración. Restaura en cualquier momento en este u otro dispositivo.';

  @override
  String get aboutDescription =>
      'Un gestor de portapapeles moderno, nativo en Windows, macOS y Linux.\nTodo local — tu historial, siempre a mano. Sin cuentas, sin telemetría, sin suscripciones.';

  @override
  String get sectionPrivacy => 'PRIVACIDAD';

  @override
  String get privacyStatement =>
      'Todo local. Nada sale de tu PC — sin telemetría, sin sincronización, sin cuentas.';

  @override
  String get privacyPolicy => 'Política de privacidad';

  @override
  String get aboutTagLocal => 'Todo local';

  @override
  String get aboutTagOpenSource => 'Código abierto';

  @override
  String get aboutTagFree => 'Gratis';

  @override
  String get sectionOtherTools => 'OTRAS HERRAMIENTAS';

  @override
  String get otherToolLinkUnbound => 'LinkUnbound';

  @override
  String get otherToolLinkUnboundDesc =>
      'Selector de navegadores de código abierto para Windows y Mac. Misma filosofía: sin anuncios, sin telemetría, todo local.';

  @override
  String get aboutLicense => 'Licencia GPL v3 — Libre y de código abierto.';

  @override
  String get permissionsTitle => 'Permiso de Accesibilidad requerido';

  @override
  String get permissionsMessage =>
      'CopyPaste necesita permiso de Accesibilidad para pegar contenido en otras apps.\n\nVe a Configuración del Sistema → Privacidad y Seguridad → Accesibilidad y activa CopyPaste.';

  @override
  String get permissionsOpenSettings => 'Abrir Configuración';

  @override
  String get permissionsDismiss => 'Después';

  @override
  String get permissionsGranted => 'Permiso concedido';

  @override
  String get permissionsResetTitle => 'Permiso de Accesibilidad perdido';

  @override
  String get permissionsResetMessage =>
      'macOS ya no reconoce el permiso de CopyPaste porque la app fue re-autorizada a través de Gatekeeper.\n\nPara solucionarlo:\n1. Abre la configuración de Accesibilidad\n2. Elimina CopyPaste de la lista (−)\n3. Vuelve a añadirlo o actívalo de nuevo';

  @override
  String get permissionsRestartMessage =>
      'Asegúrate de que CopyPaste esté activado en Privacidad y seguridad > Accesibilidad.\n\nLa app continuará automáticamente cuando detecte el permiso.';

  @override
  String get permissionsCheckAgain => 'Verificar';

  @override
  String get permissionsRestartApp => 'Reiniciar app';

  @override
  String get permissionsWaiting => 'Esperando permiso…';

  @override
  String updateBadge(String version) {
    return 'v$version disponible, por favor actualiza';
  }

  @override
  String updateAvailableWindows(String version) {
    return 'La versión $version está disponible.\n\nDescarga el instalador más reciente desde GitHub.';
  }

  @override
  String updateAvailableMac(String version) {
    return 'La versión $version está disponible.\n\nActualiza con Homebrew:\nbrew upgrade copypaste\n\nO descarga la última versión desde GitHub.';
  }

  @override
  String updateAvailableLinux(String version) {
    return 'La versión $version está disponible.\n\nDescarga la última versión desde GitHub.';
  }

  @override
  String updateAvailableStore(String version) {
    return 'La versión $version está disponible.\n\nActualiza CopyPaste desde la Microsoft Store para obtener la última versión.';
  }

  @override
  String get updateDialogTitle => 'Actualización disponible';

  @override
  String get updateViewRelease => 'Ver versión';

  @override
  String get updateDismiss => 'Después';

  @override
  String get waylandUnsupportedTitle => 'Wayland no está soportado';

  @override
  String get waylandUnsupportedBadge => 'Open source · Solo X11';

  @override
  String get waylandUnsupportedBody =>
      'El soporte en Linux está en progreso. Este proyecto lo mantiene una sola persona y necesitamos más testers para avanzar.\n\nCopyPaste funciona completamente en X11 — para usarlo, inicia sesión con X11. Lamentamos las molestias.';

  @override
  String get waylandUnsupportedGitHub => 'Ver en GitHub';

  @override
  String get waylandUnsupportedClose => 'Cerrar';

  @override
  String linuxHotkeyFallbackWarning(String requested, String fallback) {
    return 'El atajo $requested no está disponible en este escritorio X11. CopyPaste está usando temporalmente $fallback. Puedes cambiarlo en Configuración.';
  }

  @override
  String linuxHotkeyConflictWarning(String requested, String fallback) {
    return 'El atajo $requested no está disponible en este escritorio X11 y el fallback temporal $fallback también falló. Abre Configuración para elegir otro atajo.';
  }

  @override
  String get settingShowInTaskbar => 'Mantener en barra de tareas';

  @override
  String get subtitleShowInTaskbar =>
      'La app permanece visible en la barra de tareas al cerrarla. Desactívalo para ocultarla solo en la bandeja del sistema.';

  @override
  String wakeupHint(String hotkey) {
    return 'CopyPaste se ejecuta en segundo plano — presiona $hotkey o haz clic en el ícono de la bandeja para abrirlo cuando quieras.';
  }

  @override
  String taskbarOpenHint(String hotkey) {
    return 'Tip: presiona $hotkey para abrir y pegar automáticamente, sin perder el foco.';
  }

  @override
  String balloonStartupBody(String hotkey) {
    return 'Ejecutándose en segundo plano. Presiona $hotkey o haz clic en el ícono de la bandeja.';
  }

  @override
  String get balloonWakeupTitle => 'CopyPaste ya está abierto';

  @override
  String balloonWakeupBody(String hotkey) {
    return 'Presiona $hotkey o haz clic en el ícono de la bandeja para abrirlo.';
  }

  @override
  String get onboardingTitle => 'Bienvenido a CopyPaste';

  @override
  String get onboardingSubtitle => 'Todo lo que copias, guardado.';

  @override
  String get onboardingPrivacyBadge => 'Sin nube · Sin rastreo · 100% local';

  @override
  String onboardingDescription(String hotkey) {
    return 'Corre en segundo plano sin que lo notes. Presiona $hotkey cuando quieras para abrir tu historial.';
  }

  @override
  String get onboardingTrayHint =>
      'Encuéntralo junto al reloj, abajo a la derecha.';

  @override
  String get onboardingSettingsButton => 'Configuración';

  @override
  String get onboardingDismissButton => 'Empezar';
}
