# Privacy Policy

**Last updated:** October 6, 2026

---

## The Short Version

**Everything stays on your computer.** CopyPaste does not collect, transmit, or share any of your data. There's no cloud, no accounts, no telemetry, no analytics, no tracking — nothing leaves your machine, ever.

**"Everything local" is not a feature — it's the foundation.** It's a technical fact you can verify yourself: the entire source code is [open and public](https://github.com/rgdevment/CopyPaste). Read the code, run a network monitor, check for yourself.

---

## Our Privacy Philosophy: Everything Local

CopyPaste was built with a **privacy-first** mindset from day one. This isn't an afterthought or a feature — it's a core design principle. **Everything stays on your machine.**

- 🔒 **Local-only by design** — Your data never leaves your computer
- 🚫 **No telemetry** — We don't measure, track, or analyze your usage
- 🚫 **No analytics** — No Google Analytics, no App Insights, no Sentry, nothing
- 🚫 **No accounts** — No sign-up, no login, no user profiles
- 🚫 **No cloud sync** — Your clipboard history is yours alone
- 🚫 **No automatic reporting** — Errors are logged locally only; nothing is sent anywhere without your action
- 🔍 **Fully auditable** — Every line of code is open source under [GPLv3](LICENSE)

**The data on your computer is yours.** I built CopyPaste to respect that boundary completely — not just in policy, but in code.

---

## What Data Does CopyPaste Store?

CopyPaste monitors your system clipboard to maintain a local history. The following data is stored **exclusively on your computer**:

### Clipboard Content

| What | What's Stored | Where |
| :--- | :--- | :--- |
| Content | The text you copied, its kind, and the formats it came with (small ones inline, larger ones as files) | `history.db`, `blobs/` |
| Images and other large data | The bytes of what you copied, stored once per distinct content | `blobs/` |
| Files and folders | Their paths; the files themselves stay where they are | `history.db` |
| Name and colour | What you named a card and the colour you gave it | `history.db` |
| Source app | The application the copy came from | `history.db` |
| Usage | When you copied it, when you last used it, how many times you pasted it, and whether it is pinned | `history.db` |
| Details | Dimensions, duration, size and similar facts read from what you copied | `history.db` |
| Text in pictures | The text the system read out of an image, so it can be searched | `history.db` |
| Thumbnails | Previews of images, video and audio | `thumbs/` |
| Settings | Your preferences | `config.toml` |
| Logs | Application events and errors, never what you copied | `logs/` |

Everything lives in one folder: `%LOCALAPPDATA%\CopyPaste\` on Windows and
`~/Library/Application Support/CopyPaste/` on macOS.

These folders are protected by your operating system's user account permissions. Other users on the same computer cannot access them under normal conditions.

---

## What CopyPaste Does NOT Do

To be absolutely clear:

- ❌ **Does not send data to any server** — No clipboard content, no metadata, no usage data
- ❌ **Does not use cookies or tracking technologies**
- ❌ **Does not create user accounts or profiles**
- ❌ **Does not share data with third parties**
- ❌ **Does not use advertising or ad networks**
- ❌ **Does not send your data to any cloud AI** — The text in your pictures is read on your machine, by the operating system itself (Windows OCR on Windows, Apple Vision on macOS)
- ❌ **Does not sync across devices**
- ❌ **Does not upload crash reports** — A crash is written to the local log beside the history, and sharing it is a file you attach yourself
- ❌ **Does not phone home** — No background network calls except the update checker described below (all platforms)

---

## Network Requests

CopyPaste makes **one type of network request** for update checking:

### Update Checker

| Detail | Value |
| :--- | :--- |
| **Purpose** | Check whether a newer version of CopyPaste is available |
| **URL** | `https://raw.githubusercontent.com/rgdevment/CopyPaste/manifest/release-manifest.json` |
| **Method** | `GET` (read-only) |
| **Data sent** | Standard HTTP headers only — **no user data** |
| **Data received** | A small JSON file naming the latest version (and the latest test version) |
| **Frequency** | At most once a day, when you open Settings → About, plus whenever you press «Check now» |
| **Cached locally** | Yes — the last answer is kept in `update.json`, next to your settings, so the app does not ask again within the day |

**Important notes:**

- This request is **read-only** — it downloads one small public file; no data is ever uploaded
- **No clipboard content, no usage data, no personal information** is ever sent
- There is **no background polling**: nothing is asked while you are not looking at Settings → About
- Only when you press «Update» does the app also read `latest.json` (or `candidate.json` for test versions) from the same branch, which says where the installer lives and carries its signature
- The installer is **cryptographically signed**, and the signature is verified against a public key built into the app before anything is installed. The download address is also checked against our own release hosts before a single byte is fetched
- **Microsoft Store version:** nothing is checked and nothing is offered. The Store delivers its own updates
- **Homebrew** (`copypaste`, or `copypaste-beta` for test versions): you are told the `brew upgrade` command for your cask. Nothing is downloaded or installed behind Homebrew's back
- **Standalone builds (Windows / macOS):** if you press «Update», and only then, the installer is downloaded and run, and CopyPaste restarts itself. Nothing is downloaded or installed without you asking for it

### User-Initiated Browser Navigation

When you explicitly click one of the links in **Settings → About**, CopyPaste opens it in your default browser:

- **Repository** and **Give it a star** → `https://github.com/rgdevment/CopyPaste`
- **AlternativeTo** → `https://alternativeto.net/software/copypaste/about/`
- **Privacy** → `https://github.com/rgdevment/CopyPaste/blob/main/PRIVACY.md`
- **Sponsor the project** → `https://github.com/sponsors/rgdevment`
- **Buy me a coffee** → `https://buymeacoffee.com/rgdevment`
- **Rate it on the Store** (Windows only) → opens the Microsoft Store app on CopyPaste's review page
- **Other tools** → `https://rgdevment.com/tisty/` and `https://rgdevment.com/linkunbound/`

The same goes for a link you open from a card in the panel. If [LinkUnbound](https://github.com/rgdevment/LinkUnbound) is installed, these web links go through it so it can pick the browser; otherwise they go straight to your default browser.

These are standard browser navigations initiated by your action — CopyPaste does not make these requests itself.

---

## Sensitive Data Protection

CopyPaste includes built-in protections for sensitive content:

### Concealed Content Exclusion

When an application marks what it copies as secret, CopyPaste **skips it entirely** — the content is never hashed, never turned into an entry, never written to the database or to disk. This is how password managers keep their output out of clipboard history.

### How It Works

Each platform has a standard marker that the copying application sets, and CopyPaste checks for it before reading anything:

| Platform | Marker it honours |
| :--- | :--- |
| Windows | `ExcludeClipboardContentFromMonitorProcessing`, or `CanIncludeInClipboardHistory` set to 0 |
| macOS | `org.nspasteboard.ConcealedType` and `org.nspasteboard.TransientType` |

The check asks only whether the marker is present, never for the secret's contents, so its bytes are never requested in the first place.

### What This Does Not Cover

The limits matter more than the promise, so plainly:

- **The marker is set by the other application, not by CopyPaste.** Whether a given password manager sets it depends on that application, its version and its own settings. Most major ones do. We do not publish a list of "supported" managers, because that would be a compatibility claim we cannot keep honest across every release of every one of them.
- **Content that merely looks sensitive is not detected.** A password you type into a text file and copy carries no marker, so it is stored like any other text.
- If you rely on this, verify it once with your own password manager: copy a credential and check that no entry appears in CopyPaste. Ten seconds, and it tells you more than any list we could publish.

### Windows Clipboard History

CopyPaste operates independently from Windows' built-in clipboard history (`Win+V`). Your CopyPaste settings do not affect Windows clipboard behavior, and vice versa.

---

## Other Things Read on Your Machine

A few features read something else on your computer. All of it stays local:

- **Importing CopyPaste 2's history** (Settings → Backup, or the welcome tour) reads the 2.x database and pictures from disk, only when you ask. Nothing of the 2.x is changed or uploaded.
- **Pasting a picture into a terminal** works by checking which application is in front when you paste, so the picture can arrive as its file path. That check is a local look at the foreground process; nothing about it is stored or sent.
- **Opening a web link** goes through [LinkUnbound](https://github.com/rgdevment/LinkUnbound) if it is installed. CopyPaste only checks whether it is there.

---

## Backup and Restore

CopyPaste can export a backup and restore from one. Both are **manual actions you start yourself** — nothing is backed up automatically, and no backup ever leaves your machine on its own.

### What the Backup Contains

A backup is a single `.cpbackup` file: a SQLite database copied from your history, with the images and other stored data inside it.

| Content | Included |
| :--- | :--- |
| Your entire clipboard history, pinned items included | Yes |
| Stored images and other data | Yes |
| Settings and configuration | No |

This is deliberate — a backup that dropped your history would not be a backup. But it means the file is as sensitive as the history itself. On macOS the file is made readable and writable by your user only. You choose where it is written; treat it accordingly, and think twice before putting it in cloud storage or attaching it to a bug report.

### Importing

Importing **adds** what the file holds to the history you already have. It never overwrites or deletes anything, and importing the same file twice does not duplicate anything.

### Leftovers from CopyPaste 2

CopyPaste 2 could leave `.pre-restore-<timestamp>` folders in its data directory after an interrupted restore. CopyPaste 3 never creates them. If you find one, it is a copy of your old history, and **Delete CopyPaste 2's data** in Settings → Backup removes it along with the rest of the 2.x data.

---

## Data Retention & Deletion

### Automatic Cleanup

- CopyPaste automatically deletes unpinned items older than your configured retention period: 7, 30 or 90 days, or Forever (default: **30 days**)
- If you set a history size (256 MB, 512 MB or 1 GB), the oldest unpinned items go first once it is exceeded
- Cleanup runs periodically in the background
- **Pinned items are preserved** regardless of the retention setting

### Manual Deletion

You can delete any clipboard item at any time: select it and press `Delete` on
Windows or `⌘⌫` on a Mac, or use the bin on the card itself.

### Emptying the History

**Settings → History → Empty the history** deletes everything you copied and
keeps what you pinned. It cannot be undone, and nothing leaves your machine
either way.

### Complete Data Removal (Uninstall)

To completely remove all CopyPaste data when uninstalling:

**Windows:**

1. Uninstall CopyPaste (via Settings → Apps or the standalone uninstaller)
2. Delete the data folder: `%LOCALAPPDATA%\CopyPaste\`

**macOS:**

1. Move CopyPaste to Trash from Applications
2. Delete the data folder: `~/Library/Application Support/CopyPaste/`

After these steps, no CopyPaste data remains on your system.

---

## Children's Privacy

CopyPaste does not knowingly collect any personal information from anyone, including children under 13. The application does not collect personal information from any user — it has no accounts, no registration, and no data transmission.

---

## Microsoft Store Distribution

CopyPaste is available through the [Microsoft Store](https://apps.microsoft.com/detail/9NBJRZF3K856). The Store version:

- **Follows the same privacy principles** as the standalone version
- **Makes no update check at all** — updates are delivered through the Microsoft Store
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
  <p><em>Everything local. Your clipboard is yours — I built CopyPaste to keep it that way.</em></p>
</div>
