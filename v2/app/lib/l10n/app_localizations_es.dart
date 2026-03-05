// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Spanish Castilian (`es`).
class AppLocalizationsEs extends AppLocalizations {
  AppLocalizationsEs([String locale = 'es']) : super(locale);

  @override
  String get appTitle => 'CopyPaste';

  @override
  String get tabRecent => 'Recientes';

  @override
  String get tabPinned => 'Anclados';

  @override
  String get searchPlaceholder => 'Buscar...';

  @override
  String get emptyState => 'No hay elementos en esta sección';

  @override
  String updateBannerLabel(String version) {
    return 'v$version disponible — click para descargar';
  }

  @override
  String get hintBannerText => 'Personaliza tu experiencia en';

  @override
  String get hintBannerAction => 'Ajustes';

  @override
  String get settingsTitle => 'Configuración';

  @override
  String get sectionGeneral => 'GENERAL';

  @override
  String get sectionHotkey => 'ATAJO DE TECLADO';

  @override
  String get sectionStorage => 'ALMACENAMIENTO';

  @override
  String get sectionBackup => 'RESPALDO';

  @override
  String get sectionShortcuts => 'ATAJOS DE TECLADO';

  @override
  String get settingRunOnStartup => 'Iniciar con Windows';

  @override
  String get settingLanguage => 'Idioma de la interfaz';

  @override
  String get languageAuto => 'Auto';

  @override
  String get languageEnglish => 'English';

  @override
  String get languageSpanish => 'Español';

  @override
  String get settingHotkeyLabel => 'Atajo global para abrir CopyPaste';

  @override
  String get hotkeyWillApply => 'El atajo se aplicará de inmediato';

  @override
  String get settingRetentionDays => 'Retención del historial';

  @override
  String get settingRetentionDaysDesc =>
      'Días para mantener historial (0 = sin límite)';

  @override
  String get settingClearHistory => 'Limpiar todo el historial';

  @override
  String get settingClearHistoryDesc =>
      'Eliminar permanentemente todos los elementos';

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
  String get backupCreateDesc => 'Exportar todos los datos a un archivo ZIP';

  @override
  String get backupRestoreLabel => 'Restaurar respaldo';

  @override
  String get backupRestoreDesc => 'Importar datos desde un archivo de respaldo';

  @override
  String get backupCreating => 'Creando respaldo...';

  @override
  String backupSuccess(int count, int images) {
    return 'Respaldo creado: $count elementos, $images imágenes.';
  }

  @override
  String get backupError =>
      'Error al crear el respaldo. Verifica los permisos.';

  @override
  String get restoreDialogTitle => 'Restaurar respaldo';

  @override
  String get restoreDialogHint => 'Ruta al archivo .zip de respaldo';

  @override
  String get restoreDialogWarning =>
      'Esto reemplazará todos los datos actuales con el contenido del respaldo. ¿Continuar?';

  @override
  String get restoreFileNotFound => 'Archivo no encontrado.';

  @override
  String get restoreInvalidFile =>
      'Archivo de respaldo inválido. Selecciona un respaldo CopyPaste válido (.zip).';

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
  String get buttonReset => 'Restablecer';

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
  String get editCardTitle => 'Editar tarjeta';

  @override
  String get editLabelPlaceholder => 'Etiqueta (opcional)';

  @override
  String get editLabelHint => 'Máximo 40 caracteres';

  @override
  String get editColorLabel => 'Color';

  @override
  String get colorNone => 'Sin color';

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
  String get filterAll => 'Todo';

  @override
  String get filterPinned => 'Anclados';

  @override
  String get fileNotAvailable => 'Archivo no disponible';

  @override
  String get trayTooltip => 'CopyPaste';

  @override
  String get trayExit => 'Salir';

  @override
  String get shortcutsTitle => 'Atajos de teclado';

  @override
  String get shortcutsSubtitle => 'Potencia tu flujo de trabajo';

  @override
  String get shortcutsGroupGeneral => 'GENERAL';

  @override
  String get shortcutsGroupNavigation => 'NAVEGACIÓN';

  @override
  String get shortcutsGroupActions => 'ACCIONES';

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
  String get shortcutExpand => 'Expandir / contraer tarjeta';

  @override
  String get shortcutFocusSearch => 'Enfocar el buscador';

  @override
  String get trayShowHide => 'Mostrar/Ocultar';
}
