# Thumbnail Configuration Guide

## Overview
All thumbnail generation settings are centralized in `ThumbnailConfig.cs` for easy maintenance and tuning.

## Configuration Parameters

### `Width` (default: 250)
- **Purpose**: Maximum width for generated thumbnails
- **Impact**: 
  - ↓ Lower = Less memory, faster processing, lower quality
  - ↑ Higher = More memory, slower processing, better quality
- **Recommended range**: 200-300px
- **Memory impact**: ~60KB per image @ 250px vs ~180KB @ 300px

### `QualityPng` (default: 85)
- **Purpose**: PNG encoding quality for image thumbnails
- **Impact**:
  - ↓ Lower = Smaller files, faster save, visible compression artifacts
  - ↑ Higher = Larger files, slower save, better quality
- **Recommended range**: 75-90
- **File size impact**: Quality 75 = ~40% smaller than quality 90

### `QualityJpeg` (default: 85)
- **Purpose**: JPEG encoding quality for video/audio thumbnails
- **Impact**: Same as QualityPng but for media files
- **Recommended range**: 75-90
- **Note**: Videos use JPEG instead of PNG to reduce file size

### `GarbageCollectionThreshold` (default: 1MB)
- **Purpose**: Triggers memory cleanup after processing large images
- **Impact**:
  - ↓ Lower = More frequent GC, better memory control, slight performance impact
  - ↑ Higher = Less frequent GC, faster processing, higher memory usage
- **Recommended range**: 500KB - 2MB

### `UIDecodeHeight` (default: 220)
- **Purpose**: Pixel height when decoding thumbnails for display
- **Impact**:
  - Should be slightly smaller than `Width` for optimal rendering
  - Prevents loading full resolution images in memory
- **Recommended**: Width - 30px

## Performance Profiles

### 🔋 Low Memory Profile (for 4-8GB RAM systems)
```csharp
ThumbnailConfig.Width = 200;
ThumbnailConfig.QualityPng = 75;
ThumbnailConfig.QualityJpeg = 75;
ThumbnailConfig.GarbageCollectionThreshold = 500_000;
ThumbnailConfig.UIDecodeHeight = 180;
```
**Expected RAM usage**: ~150-200MB with 50 images

### ⚖️ Balanced Profile (recommended)
```csharp
ThumbnailConfig.Width = 250;
ThumbnailConfig.QualityPng = 85;
ThumbnailConfig.QualityJpeg = 85;
ThumbnailConfig.GarbageCollectionThreshold = 1_000_000;
ThumbnailConfig.UIDecodeHeight = 220;
```
**Expected RAM usage**: ~200-250MB with 50 images

### 🎨 High Quality Profile (for 16GB+ RAM systems)
```csharp
ThumbnailConfig.Width = 300;
ThumbnailConfig.QualityPng = 90;
ThumbnailConfig.QualityJpeg = 90;
ThumbnailConfig.GarbageCollectionThreshold = 2_000_000;
ThumbnailConfig.UIDecodeHeight = 260;
```
**Expected RAM usage**: ~300-400MB with 50 images

## How to Change Configuration

### Option 1: Modify at Startup (Recommended)
In `App.xaml.cs` or `Program.cs`:
```csharp
// Apply custom configuration before initializing services
ThumbnailConfig.Width = 200;
ThumbnailConfig.QualityPng = 80;
// ... etc
```

### Option 2: Modify Defaults
Edit `CopyPaste.Core\ThumbnailConfig.cs` directly and rebuild the application.

## Troubleshooting

### App uses too much RAM
- Reduce `Width` to 200px
- Lower `QualityPng` and `QualityJpeg` to 75-80
- Decrease `GarbageCollectionThreshold` to 500KB

### Thumbnails look pixelated
- Increase `Width` to 280-300px
- Increase `QualityPng` and `QualityJpeg` to 88-90
- Increase `UIDecodeHeight` proportionally

### Thumbnail generation is slow
- Reduce `Width` to 200-220px
- Lower quality settings don't significantly impact speed
- Check if GC is being triggered too often (increase threshold)

## Technical Notes

- PNG format is used for image thumbnails (supports transparency)
- JPEG format is used for video/audio thumbnails (smaller file size)
- Garbage collection is forced only for large images to prevent memory buildup
- UI decode height prevents loading full resolution images in memory
- All thumbnails maintain aspect ratio automatically
