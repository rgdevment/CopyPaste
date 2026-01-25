# ESTANDAR DE CONFIGURACION DE UI - GUIA COMPLETA

## 1. ALCANCE
Este documento describe las opciones de configuracion disponibles para personalizar el comportamiento de la interfaz de usuario de CopyPaste. Todas las configuraciones se centralizan en UIConfig.cs y se modifican en App.xaml.cs antes de la inicializacion.

## 2. TABLA DE PARAMETROS Y METRICAS
| Parametro | Default | Recomendado | Impacto |
| :--- | :--- | :--- | :--- |
| PageSize | 20 | 15 - 30 | Items a cargar por pagina |
| MaxItemsBeforeCleanup | 100 | 50 - 150 | Limite RAM antes de purgar al perder foco |
| ScrollLoadThreshold | 100px | 50 - 200px | Distancia desde el fondo para cargar mas |
| WindowWidth | 400px | 350 - 500px | Ancho de la ventana lateral en pixeles |
| WindowMarginTop | 8px | - | Margen vertical superior |
| WindowMarginBottom | 16px | - | Margen vertical inferior |
| Hotkey.VirtualKey | 0x56 (V) | - | Codigo Hex de la tecla del atajo |
| Hotkey.UseWinKey | true | - | Usar tecla Windows como modificador |
| Hotkey.UseAltKey | true | - | Incluir modificador Alt (Siempre true) |

## 3. DEFINICIONES DETALLADAS DE COMPONENTES

### 3.1 Logica de Paginacion y Memoria
* PageSize: Numero de elementos del portapapeles a cargar por pagina. Valores altos significan carga inicial mas lenta pero menos cargas frecuentes.
* MaxItemsBeforeCleanup: Maximo de elementos en memoria antes de limpieza. Se activa cuando se desactiva la ventana (blur).
* ScrollLoadThreshold: Umbral de desplazamiento (en pixeles) desde el fondo para cargar mas elementos.

### 3.2 Atajo de Teclado Global (Hotkey)
CopyPaste registra un atajo de teclado global para mostrar/ocultar la ventana.
* Por defecto: Win + Alt + V.
* Fallback automatico: Ctrl + Alt + V (si Win no se puede registrar).
* Otros codigos de tecla comunes:
    * 0x43 = C
    * 0x56 = V
    * 0x58 = X
    * 0x5A = Z

## 4. NOTAS DE RENDIMIENTO
* PageSize mas alto: Carga inicial mas lenta, menos cargas frecuentes.
* MaxItemsBeforeCleanup mas alto: Mayor uso de memoria, menos recargas.
* WindowWidth mas ancho: Mejor visualizacion de contenido largo.
* ScrollLoadThreshold mas bajo: Carga anticipada mas suave, mas llamadas al servicio.

## 5. RELACION CON THUMBNAILCONFIG
UIConfig complementa a ThumbnailConfig (del proyecto Core):
* ThumbnailConfig: Controla la generacion y calidad de miniaturas.
* UIConfig: Controla el comportamiento y apariencia de la interfaz.
* Ambos pueden configurarse independientemente en App.xaml.cs.

## 6. EJEMPLOS DE IMPLEMENTACION

### Configuracion Personalizada en App.xaml.cs (OnLaunched)
// Configuracion de UI personalizada
UIConfig.PageSize = 30;                 // Cargar mas elementos
UIConfig.WindowWidth = 450;             // Ventana mas ancha
UIConfig.Hotkey.UseWinKey = false;     // Usar Ctrl en lugar de Win
UIConfig.Hotkey.VirtualKey = 0x43;     // Cambiar a tecla C (Ctrl + Alt + C)

_window = new MainWindow(_service!);
_window.Activate();

### Desactivar el Atajo
Para desactivar el atajo de teclado, comenta estas lineas en MainWindow.xaml.cs:
// RegisterGlobalHotkey();
// HotkeyHelper.RegisterMessageHandler(this, OnHotkeyPressed);

## 7. MANTENIMIENTO Y SOLUCION DE PROBLEMAS
* Atajo: Funciona globalmente en todo el sistema. Si falla el registro con Win, intenta automaticamente con Ctrl.
* Optimizacion: Para menor uso de RAM, disminuir PageSize y MaxItemsBeforeCleanup.
