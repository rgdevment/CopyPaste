# CopyPaste Configuration Guide

## Overview

All configuration is centralized in a single file: `MyM.json`, located at:
```
%LocalAppData%\CopyPaste\config\MyM.json
```

The configuration model is defined in `MyMConfig.cs`. If the JSON file is missing or a property is not present, default values from the class are used.

## Configuration Properties

### Startup
| Property | Default | Description |
|----------|---------|-------------|
| `RunOnStartup` | `true` | Start CopyPaste automatically with Windows |

### Hotkey (Global Shortcut)
| Property | Default | Description |
|----------|---------|-------------|
| `UseCtrlKey` | `false` | Include Ctrl modifier |
| `UseWinKey` | `true` | Include Windows key modifier |
| `UseAltKey` | `true` | Include Alt modifier |
| `UseShiftKey` | `false` | Include Shift modifier |
| `VirtualKey` | `0x56` (V) | Virtual key code for the shortcut |
| `KeyName` | `"V"` | Display name of the key |

**Default shortcut:** `Win + Alt + V`

### User Interface
| Property | Default | Range | Description |
|----------|---------|-------|-------------|
| `WindowWidth` | `400` | 300-800 | Width of the sidebar panel in pixels |
| `WindowMarginTop` | `8` | - | Top margin from work area |
| `WindowMarginBottom` | `16` | - | Bottom margin from work area |

### Performance
| Property | Default | Range | Description |
|----------|---------|-------|-------------|
| `PageSize` | `20` | 5-100 | Items to load per page |
| `MaxItemsBeforeCleanup` | `100` | 20-500 | Max items in RAM before cleanup |
| `ScrollLoadThreshold` | `100` | 20-500 | Pixels from bottom to trigger load |

### Storage
| Property | Default | Range | Description |
|----------|---------|-------|-------------|
| `RetentionDays` | `30` | 0-365 | Days to keep history (0 = unlimited) |

### Paste Behavior
| Property | Default | Range | Description |
|----------|---------|-------|-------------|
| `DuplicateIgnoreWindowMs` | `300` | 100-1000 | Anti-duplicate window in ms |
| `DelayBeforeFocusMs` | `50` | 10-500 | Delay before restoring focus |
| `DelayBeforePasteMs` | `100` | 20-500 | Delay before simulating Ctrl+V |
| `MaxFocusVerifyAttempts` | `10` | - | Max focus verification attempts |

### Thumbnails (Advanced)
| Property | Default | Range | Description |
|----------|---------|-------|-------------|
| `ThumbnailWidth` | `170` | 100-400 | Max thumbnail width in pixels |
| `ThumbnailQualityPng` | `80` | 10-100 | PNG compression quality |
| `ThumbnailQualityJpeg` | `80` | 10-100 | JPEG compression quality |
| `ThumbnailGCThreshold` | `1000000` | - | Bytes threshold for GC trigger |
| `ThumbnailUIDecodeHeight` | `95` | - | Decode height for UI display |

## Example MyM.json

```json
{
  "RunOnStartup": true,
  "UseCtrlKey": false,
  "UseWinKey": true,
  "UseAltKey": true,
  "UseShiftKey": false,
  "VirtualKey": 86,
  "KeyName": "V",
  "WindowWidth": 400,
  "PageSize": 20,
  "MaxItemsBeforeCleanup": 100,
  "ScrollLoadThreshold": 100,
  "WindowMarginTop": 8,
  "WindowMarginBottom": 16,
  "RetentionDays": 30,
  "DuplicateIgnoreWindowMs": 300,
  "DelayBeforeFocusMs": 50,
  "DelayBeforePasteMs": 100,
  "MaxFocusVerifyAttempts": 10,
  "ThumbnailWidth": 170,
  "ThumbnailQualityPng": 80,
  "ThumbnailQualityJpeg": 80,
  "ThumbnailGCThreshold": 1000000,
  "ThumbnailUIDecodeHeight": 95
}
```

## How It Works

1. **App Start**: `ConfigLoader.Config` loads from `MyM.json` or creates defaults
2. **Missing Properties**: If JSON lacks a property, the default from `MyMConfig` is used
3. **Settings UI**: The ConfigWindow allows editing all settings visually
4. **Save**: Changes are written to `MyM.json` and require app restart

## Virtual Key Codes

Common keys for hotkey configuration:
| Key | Code (Hex) | Code (Dec) |
|-----|------------|------------|
| A | 0x41 | 65 |
| C | 0x43 | 67 |
| D | 0x44 | 68 |
| E | 0x45 | 69 |
| Q | 0x51 | 81 |
| S | 0x53 | 83 |
| V | 0x56 | 86 |
| W | 0x57 | 87 |
| X | 0x58 | 88 |
| Z | 0x5A | 90 |
