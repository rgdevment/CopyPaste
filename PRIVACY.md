# Privacy Policy

**Last updated:** February 7, 2026

---

## The Short Version

**CopyPaste does not collect, transmit, or share any of your data.** Everything stays on your computer. There's no cloud, no accounts, no telemetry, no analytics, no tracking — nothing leaves your machine.

This isn't a marketing claim. It's a technical fact you can verify yourself — the entire source code is [open and public](https://github.com/rgdevment/CopyPaste).

---

## Our Privacy Philosophy

CopyPaste was built with a **privacy-first** mindset from day one. This isn't an afterthought or a feature — it's a core design principle:

- 🔒 **Local-only by design** — Your data never leaves your computer
- 🚫 **No telemetry** — We don't measure, track, or analyze your usage
- 🚫 **No analytics** — No Google Analytics, no App Insights, no Sentry, nothing
- 🚫 **No accounts** — No sign-up, no login, no user profiles
- 🚫 **No cloud sync** — Your clipboard history is yours alone
- 🔍 **Fully auditable** — Every line of code is open source under [GPLv3](LICENSE)

---

## What Data Does CopyPaste Store?

CopyPaste monitors your system clipboard to maintain a local history. The following data is stored **exclusively on your computer**:

### Clipboard Content
| Type | What's Stored | Where |
|:---|:---|:---|
| **Text** | The copied text content | SQLite database |
| **Images** | Image files (PNG) | Local `images` folder |
| **Files & Folders** | File/folder paths (not the files themselves) | SQLite database |
| **Links** | URL text | SQLite database |
| **Audio & Video** | File paths only | SQLite database |

### Metadata
For each clipboard item, CopyPaste also stores:
- **Timestamp** — When the item was copied
- **Content type** — Text, Image, File, Folder, Link, Audio, or Video
- **Source application** — The name of the app where you copied from (*window title*)
- **User labels** — Custom labels you assign to items (optional)
- **Color tags** — Color categories you assign (optional)
- **Pin status** — Whether you pinned the item
- **Image thumbnails** — Smaller preview versions of copied images

### Configuration
Your settings are stored locally:
- Hotkey preferences
- Theme selection
- Language preference
- Panel width
- Retention period
- Filter behavior
- Startup preferences

### Logs
Application logs are stored locally for troubleshooting:
- **Location:** `%LOCALAPPDATA%\CopyPaste\logs\`
- **Content:** Application events, errors, and diagnostic information
- **No personal data:** Logs do not contain clipboard content

---

## Where Is Everything Stored?

All data is stored under your Windows user profile:

| Data | Location |
|:---|:---|
| **Database** | `%LOCALAPPDATA%\CopyPaste\clipboard.db` |
| **Images** | `%LOCALAPPDATA%\CopyPaste\images\` |
| **Thumbnails** | `%LOCALAPPDATA%\CopyPaste\thumbs\` |
| **Configuration** | `%LOCALAPPDATA%\CopyPaste\config\` |
| **Logs** | `%LOCALAPPDATA%\CopyPaste\logs\` |
| **Themes** | `%LOCALAPPDATA%\CopyPaste\themes\` |

These folders are protected by your Windows user account permissions. Other users on the same computer cannot access them under normal conditions.

---

## What CopyPaste Does NOT Do

To be absolutely clear:

- ❌ **Does not send data to any server** — No clipboard content, no metadata, no usage data
- ❌ **Does not use cookies or tracking technologies**
- ❌ **Does not create user accounts or profiles**
- ❌ **Does not share data with third parties**
- ❌ **Does not use advertising or ad networks**
- ❌ **Does not use AI or machine learning** on your data
- ❌ **Does not sync across devices**
- ❌ **Does not upload crash reports** — Errors are logged locally only

---

## Network Requests

CopyPaste makes **one type of network request**, and only in the standalone version:

### Update Checker (Standalone Version Only)

| Detail | Value |
|:---|:---|
| **Purpose** | Check if a newer version of CopyPaste is available |
| **URL** | `https://api.github.com/repos/rgdevment/CopyPaste/releases/latest` |
| **Method** | `GET` (read-only) |
| **Data sent** | `User-Agent: CopyPaste-UpdateChecker` header only — **no user data** |
| **Data received** | Version number and download URL from GitHub's public API |
| **Frequency** | 30 seconds after startup, then every 12 hours |
| **Can be disabled?** | See below |

**Important notes:**
- This request is **read-only** — it only downloads public release information from GitHub
- **No clipboard content, no usage data, no personal information** is ever sent
- The request goes to GitHub's public API, not to any server we operate
- **Microsoft Store version:** The update checker is **completely disabled**. The Store handles updates automatically through its own infrastructure

> **Standalone users:** The update checker cannot currently be disabled via settings, but it sends zero user data. If you require fully offline operation, the Microsoft Store version makes no network requests at all.

### User-Initiated Browser Navigation

When you explicitly click certain UI buttons, CopyPaste opens URLs in your default browser:
- **"Report issue"** button → Opens `https://github.com/rgdevment/CopyPaste/issues`
- **"Download update"** dialog → Opens the GitHub release page

These are standard browser navigations initiated by your action — CopyPaste does not make these requests itself.

---

## Sensitive Data Protection

CopyPaste includes built-in protections for sensitive content:

### Password Manager Exclusion
Clipboard content from recognized password managers is **automatically excluded** from history. Supported password managers include:
- 1Password
- Bitwarden
- LastPass
- KeePass
- And others that use standard clipboard security flags

### How It Works
- Password managers typically set a clipboard format flag indicating sensitive content
- CopyPaste detects these flags and **skips storing** the content entirely
- The sensitive data is never written to the database or disk

### Windows Clipboard History
CopyPaste operates independently from Windows' built-in clipboard history (`Win+V`). Your CopyPaste settings do not affect Windows clipboard behavior, and vice versa.

---

## Data Retention & Deletion

### Automatic Cleanup
- CopyPaste automatically deletes unpinned items older than your configured retention period (default: **30 days**)
- Cleanup runs periodically in the background
- **Pinned items are preserved** regardless of the retention setting

### Manual Deletion
You can delete any clipboard item at any time:
- Select an item and press `Delete`
- Right-click and choose "Delete"

### Complete Data Removal
To completely remove all CopyPaste data from your computer:

1. **Uninstall CopyPaste** (via Settings → Apps or the standalone uninstaller)
2. **Delete the data folder:** `%LOCALAPPDATA%\CopyPaste\`

After these steps, no CopyPaste data remains on your system.

---

## Children's Privacy

CopyPaste does not knowingly collect any personal information from anyone, including children under 13. The application does not collect personal information from any user — it has no accounts, no registration, and no data transmission.

---

## Microsoft Store Distribution

CopyPaste is available through the [Microsoft Store](https://apps.microsoft.com/detail/9NBJRZF3K856). The Store version:

- **Follows the same privacy principles** as the standalone version
- **Does not make network requests** — the update checker is disabled (the Store handles updates)
- **Uses MSIX packaging** — installs/uninstalls cleanly with Windows standard mechanisms
- **Microsoft Store policies** apply to distribution, but CopyPaste itself does not share any data with Microsoft beyond what the Store platform requires for installation and updates

For Microsoft's own privacy practices regarding the Store, refer to [Microsoft's Privacy Statement](https://privacy.microsoft.com/privacystatement).

---

## Open Source Transparency

The best privacy policy is one you can verify. CopyPaste is **100% open source** under the [GNU General Public License v3.0](LICENSE):

- 📂 **Full source code:** [github.com/rgdevment/CopyPaste](https://github.com/rgdevment/CopyPaste)
- 🔍 **Audit the code yourself** — every network request, every database write, every file operation
- 🐛 **Report concerns** — [open an issue](https://github.com/rgdevment/CopyPaste/issues) or [email us](mailto:github@apirest.cl)

We encourage security researchers and privacy advocates to inspect the code. See our [Security Policy](SECURITY.md) for responsible disclosure guidelines.

---

## Changes to This Policy

If we ever change this privacy policy, the changes will be:
- Committed to the public repository with a clear commit message
- Reflected in the "Last updated" date at the top
- Documented in the release notes

Since CopyPaste is open source, any change to privacy behavior would also be visible as a code change in the public repository before it reaches you.

---

## Contact

If you have questions or concerns about this privacy policy:

- 📧 **Email:** [github@apirest.cl](mailto:github@apirest.cl)
- 💬 **GitHub Discussions:** [github.com/rgdevment/CopyPaste/discussions](https://github.com/rgdevment/CopyPaste/discussions)
- 🐛 **Issues:** [github.com/rgdevment/CopyPaste/issues](https://github.com/rgdevment/CopyPaste/issues)

---

<div align="center">
  <p><em>Your clipboard is yours. We built CopyPaste to keep it that way.</em></p>
</div>
