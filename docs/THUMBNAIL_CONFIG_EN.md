# THUMBNAIL CONFIGURATION STANDARD - COMPLETE GUIDE

## 1. SCOPE
All thumbnail generation settings are centralized in ThumbnailConfig.cs for easy maintenance and tuning. This file manages how the application processes visual assets and manages memory during generation.

## 2. PARAMETERS AND METRICS TABLE
| Parameter | Default | Recommended | Impact |
| :--- | :--- | :--- | :--- |
| Width | 250px | 200 - 300px | Max width. Higher = more memory/better quality |
| QualityPng | 85 | 75 - 90 | PNG encoding quality for image thumbnails |
| QualityJpeg | 85 | 75 - 90 | JPEG encoding quality for video/audio |
| GarbageCollectionThreshold | 1MB | 500KB - 2MB | Triggers memory cleanup after large images |
| UIDecodeHeight | 220px | Width - 30px | Pixel height used when decoding for display |

## 3. DETAILED COMPONENT DEFINITIONS

### 3.1 Dimension and Memory Logic
* Width: Maximum width for generated assets.
    * 250px impact: ~60KB per image.
    * 300px impact: ~180KB per image.
* UIDecodeHeight: Optimal height for UI rendering. It prevents loading full resolution images into memory. Recommended setting: Width - 30px.

### 3.2 Encoding and Formats
* PNG Format: Used for image thumbnails to support transparency.
* JPEG Format: Used for video and audio thumbnails to reduce file size.
* Quality (75 vs 90): Quality 75 is approximately 40% smaller than quality 90.

### 3.3 Garbage Collection (GC)
* Purpose: Forces memory cleanup to prevent buildup after processing large files.
* Lower Threshold: More frequent GC, better memory control, slight performance hit.
* Higher Threshold: Less frequent GC, faster processing, higher memory usage.

## 4. PERFORMANCE PROFILES

### Low Memory (4-8GB RAM)
* Width: 200
* QualityPng/Jpeg: 75
* GC Threshold: 500,000 (500KB)
* UIDecodeHeight: 180

### Balanced (Recommended)
* Width: 250
* QualityPng/Jpeg: 85
* GC Threshold: 1,000,000 (1MB)
* UIDecodeHeight: 220

### High Quality (16GB+ RAM)
* Width: 300
* QualityPng/Jpeg: 90
* GC Threshold: 2,000,000 (2MB)
* UIDecodeHeight: 260

## 5. IMPLEMENTATION EXAMPLES

### Option 1: Modify at Startup (Recommended)
In App.xaml.cs or Program.cs:
// Apply custom configuration before initializing services
ThumbnailConfig.Width = 200;
ThumbnailConfig.QualityPng = 80;
ThumbnailConfig.GarbageCollectionThreshold = 500000;

### Option 2: Modify Defaults
Edit CopyPaste.Core\ThumbnailConfig.cs directly and rebuild the application.

## 6. MAINTENANCE AND TROUBLESHOOTING

### RAM Usage Issues
* Reduce Width to 200px.
* Lower QualityPng and QualityJpeg to 75-80.
* Decrease GarbageCollectionThreshold to 500KB.

### Visual Quality Issues
* Pixelated Thumbnails: Increase Width (280-300px), Quality (88-90), and UIDecodeHeight.
* Aspect Ratio: All thumbnails maintain aspect ratio automatically.

### Performance Issues
* Slow Generation: Reduce Width. Check if GC triggers too often (if so, increase threshold).

## 7. TECHNICAL NOTES
* Aspect ratio is automatically maintained.
* PNG is mandatory for transparency support.
* JPEG is used for media to optimize disk/RAM space.
