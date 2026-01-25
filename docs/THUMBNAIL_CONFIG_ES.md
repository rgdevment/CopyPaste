# ESTANDAR DE CONFIGURACION DE MINIATURAS - GUIA COMPLETA

## 1. ALCANCE
Todos los ajustes de generacion de miniaturas estan centralizados en ThumbnailConfig.cs para facilitar el mantenimiento y ajuste. Este archivo gestiona como la aplicacion procesa recursos visuales y maneja la memoria durante la generacion.

## 2. TABLA DE PARAMETROS Y METRICAS
| Parametro | Default | Recomendado | Impacto |
| :--- | :--- | :--- | :--- |
| Width | 250px | 200 - 300px | Ancho maximo. Mayor = mas memoria/mejor calidad |
| QualityPng | 85 | 75 - 90 | Calidad de codificacion PNG para imagenes |
| QualityJpeg | 85 | 75 - 90 | Calidad de codificacion JPEG para video/audio |
| GarbageCollectionThreshold | 1MB | 500KB - 2MB | Limpieza de memoria tras imagenes grandes |
| UIDecodeHeight | 220px | Width - 30px | Altura de decodificacion para visualizacion |

## 3. DEFINICIONES DETALLADAS DE COMPONENTES

### 3.1 Logica de Dimensiones y Memoria
* Width: Ancho maximo para los recursos generados.
    * Impacto 250px: ~60KB por imagen.
    * Impacto 300px: ~180KB por imagen.
* UIDecodeHeight: Altura optima para renderizado en UI. Evita cargar imagenes a resolucion completa en memoria. Recomendado: Width - 30px.

### 3.2 Codificacion y Formatos
* Formato PNG: Usado para miniaturas de imagen para soportar transparencia.
* Formato JPEG: Usado para miniaturas de video y audio para reducir tamaño.
* Calidad (75 vs 90): La calidad 75 es aproximadamente un 40% mas pequeña que la calidad 90.

### 3.3 Garbage Collection (GC)
* Proposito: Fuerza la limpieza de memoria para evitar acumulacion tras procesar archivos grandes.
* Umbral bajo: GC mas frecuente, mejor control de memoria, ligero impacto en rendimiento.
* Umbral alto: GC menos frecuente, procesamiento mas rapido, mayor uso de memoria.

## 4. PERFILES DE RENDIMIENTO

### Perfil de Memoria Baja (4-8GB RAM)
* Width: 200
* QualityPng/Jpeg: 75
* GC Threshold: 500,000 (500KB)
* UIDecodeHeight: 180

### Perfil Balanceado (Recomendado)
* Width: 250
* QualityPng/Jpeg: 85
* GC Threshold: 1,000,000 (1MB)
* UIDecodeHeight: 220

### Perfil de Alta Calidad (16GB+ RAM)
* Width: 300
* QualityPng/Jpeg: 90
* GC Threshold: 2,000,000 (2MB)
* UIDecodeHeight: 260

## 6. MANTENIMIENTO Y SOLUCION DE PROBLEMAS

### Problemas de Uso de RAM (Sobre carga de items en vista)
* Reducir Width a 200px.
* Bajar QualityPng y QualityJpeg a 75-80.
* Disminuir GarbageCollectionThreshold a 500KB.

### Problemas de Calidad Visual
* Miniaturas Pixeladas: Aumentar Width (280-300px), Quality (88-90) y UIDecodeHeight.
* Relacion de Aspecto: Se mantiene automaticamente en todas las miniaturas.

### Problemas de Rendimiento
* Generacion Lenta: Reducir Width. Revisar si el GC se dispara muy seguido (si es asi, subir el umbral).

## 7. NOTAS TECNICAS
* La relacion de aspecto se mantiene automaticamente.
* PNG es obligatorio para soporte de transparencia.
* JPEG se usa en multimedia para optimizar espacio en disco y RAM.
