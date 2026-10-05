# Releasing CopyPaste

`.github/workflows/release.yml` builds, signs and publishes the 3.0 — modelled
on the pipeline in [rgdevment/Tisty](https://github.com/rgdevment/Tisty), which
is audited and in production. What follows is what it does, plus the steps that
are manual the first time and cannot be automated away.

## The pipeline

A `v*` tag on `main` fans out, in this order, with every stage gated on the one
before it:

| Stage                | What it produces                                                       |
| -------------------- | ---------------------------------------------------------------------- |
| `gate`               | Style and prose, so a tag never publishes an unformatted tree.          |
| `tested`             | Proof the commit was green when it landed.                              |
| `version`            | The version resolved from the tag, used by every stage after it.        |
| `bundle-windows`     | `cp-panel` sidecar + the Tauri app, signed with the PFX, as an NSIS installer. |
| `bundle-macos`       | The same, per architecture, signed and notarised, as a `.dmg`.          |
| `bundle-msix`        | MSIX for the Microsoft Store.                                           |
| `publish`            | The GitHub Release with every artifact, and the `manifest` branch.      |
| `verify`             | Installs what was just published and checks the update feed answers.    |
| `msstore`            | Submits the MSIX (stable tags only).                                    |
| `winget`             | Opens the pull request that adds this version to `winget-pkgs`.         |
| `homebrew`           | Rewrites `copypaste` (stable) or `copypaste-beta` (candidate) in the tap. |

`feed.yml` then watches the published manifest daily: a deleted asset, a
retired release or a force-pushed branch breaks the update feed silently, and
nothing else would notice.

### macOS is built, and the tag refuses to go out unsigned

`BUILD_MACOS` at the top of `release.yml` gates `bundle-macos` and is set to
`"true"`: the Apple secrets are the ones the 2.x already used. With it on, the
`version` job demands all five (`MACOS_CERTIFICATE_P12`,
`MACOS_CERTIFICATE_PASSWORD`, `APPLE_ID`, `APPLE_APP_PASSWORD`,
`APPLE_TEAM_ID`) and fails the tag before anything is built if one is missing.
The signing identity is read off the imported certificate, not a secret. That is the point: an unsigned, un-notarised `.dmg` is
worse than no `.dmg`.

The job has never run yet, so the first tag is also the first proof that the
sidecar is signed and the bundle notarises. Cut it with `workflow_dispatch`
before cutting it for real.

### What the 3.0 does not carry yet

Bringing the 2.x history over is done and wired: `former` finds `clipboard.db`
and weighs it, and Settings → Backup brings it over or deletes it, each behind
its own button.

What is left is publishing the bridge for whoever stays on the 2.x. It is
built — see the crossing below — and waits only on the `BRIDGE_MANIFEST`
variable and a stable tag.

### The app does check for updates

`app/src-tauri/src/update.rs` works as Tisty's does, once a day and whenever
the person asks. The `publish` job keeps three files on the `manifest` branch:
`release-manifest.json` (`latest`, `latestPrerelease`), and two channels signed
with the updater's minisign key, `latest.json` for stable and `candidate.json`
for candidates. A copy reads the manifest to decide what to offer — a stable
copy only ever a stable version, a candidate the newest of either — and then
downloads from the channels, a candidate trying `candidate.json` first. Until
the first stable exists, `latest.json` carries the newest candidate too, so a
copy that only knew that channel is not left behind.

What it does with an offer depends on where the copy came from, which it reads
off its own path: a copy under `WindowsApps` is the Store's to update and is
never offered anything; one under `Caskroom/copypaste-beta` or
`Caskroom/copypaste` is told its own `brew` command rather than handed an
installer; anything else installs its own update.

Four things guard the install, all of them borrowed from Tisty: the download
address must be this repository's releases on `github.com` (or
`objects.githubusercontent.com`) before a byte is fetched; the version is
pinned to the one the person was shown, so a feed that moves cannot hand over
another; every updater signature is bound to its version (`signer sign
--app-version`, `requireSignedVersion`), so an older release cannot be served
under a newer number, and `verify` refuses a release whose signatures do not
say it; and a copy running from the mounted `.dmg` refuses rather than failing
after the whole download. The panel is
stopped first, because on Windows an installer cannot replace a binary that is
running, and it is brought back if the install does not go through.

## First time only — what a human has to do

Three channels need a decision or a human before they can be switched on.

### 1. winget

`winget-releaser` **adds a version to a package that already exists**; it
refuses a package the community repository has never seen. Today
`rgdevment.CopyPaste` is not in `microsoft/winget-pkgs` at all.

1. Cut the first release and let it publish the installer.
2. Submit the manifest by hand against that public URL:
   `wingetcreate new <url-of-the-installer>`.
3. Wait for a `winget-pkgs` moderator to merge it (days, not hours).
4. Only then set `vars.WINGET_PUBLISH=true` and `secrets.WINGET_TOKEN`.

The `winget` job starts disabled by that variable, so it is harmless to ship
the workflow before the package exists: it logs a notice and does nothing.

### 2. Microsoft Store

Nothing manual: the SKU is the one the 2.x already publishes, so the usual
first-submission hurdle does not apply. `vars.STORE_PUBLISH` goes on with the
3.0.0 — a decision, not an oversight; see the crossing below.

### 3. Homebrew tap

The tap repository already exists and the `homebrew` job rewrites the cask
itself, so there is nothing to write by hand. Leave the frozen Linux formulae
alone: the job must not sweep them.

The cask's `homepage` is `https://rgdevment.com/copypaste/`, the product page —
not the repository. The same holds anywhere a store or a package manager asks
for a website: the repository is where the code lives, not where a user is sent.

## Cutting a release

Create and push an **annotated, signed** tag on `main`. The tag body is the
release notes.

```sh
git checkout main && git pull
TAG=v3.0.0
git tag -s "$TAG" -m "$TAG

What changed, in the user's words."
git push origin "$TAG"
```

Nothing to bump by hand: the version travels from the tag name into the
binaries and the manifest.

A candidate is tagged `vX.Y.Z-rcN` with N from 1 to 9 and nothing else: semver
orders `rc.3` and `rc10` below `rc2`, so either would never be offered. The
tag is refused when X.Y.Z is not above the stable already out. Candidates
publish the GitHub Release and the `copypaste-beta` cask, and skip the Store
and winget.

## Crossing from the 2.x — decided 2026-09-29

Every installed 2.x copy polls
`https://github.com/rgdevment/CopyPaste/releases/latest/download/release-manifest.json`
and its `.sig` every 24 hours, verifies the signature against an Ed25519 public
key bundled in the app, and caches the answer for fifteen days
(`app/lib/services/release_manifest_service.dart` on `v2-stable`). Left with no
file it falls back to that cache and marks it stale; it never learns the 3.0
exists. So the 3.0 speaks the old language too, for a while.

**The 3.0 publishes the bridge itself**, as an asset on its own releases, and
no new 2.x is cut. The alternative — a bridge version of the 2.x that reads the
`manifest` branch — reaches only whoever installs it, and to hear of it they
would need this same channel anyway.

It lives in the `publish` job as `The manifest a 2.x can read`, behind the
`BRIDGE_MANIFEST` variable. `.github/bridge-manifest.json` is the template,
`scripts/sign_manifest.sh` signs it with `RELEASE_PRIVATE_KEY` — still in the
repository's secrets — and refuses to write anything unless the signature
answers to `.github/v2-release-pubkey.txt`, the very key those copies carry. A
2.x drops a manifest whose signature does not verify without a word, so that
check cannot be left to chance. With the variable on, `RELEASE_PRIVATE_KEY`
joins the secrets the `version` job demands before anything is built, because
finding out at the end costs an hour of builds and the release aborts before
the GitHub Release is even created.

**Switching the variable off is not a quiet retirement.** The asset is served
from `releases/latest/download/`, so the first stable tag published without it
hands a 404 to every copy still on the 2.x, all at once, and they fall back to
their own cache and go quiet for good — the very outcome this exists to
prevent. So the watchdog in `feed.yml` does _not_ hide when the variable is
off: it keeps looking, and while the release that is out still carries the
manifest it warns, every morning, that the next stable tag will cut it. Turning
it off is a decision to be made after that warning, not before.

This file has nothing to do with the `release-manifest.json` on the `manifest`
branch beyond the name, which the URL above forces. The 2.x wants `schema`,
`latest`, `minimumSupported`, `blockedVersions`, `severity`, `channels` and
`releaseNotes`, and rejects the whole document if `minimumSupported` is
missing; the feed on the branch is a three-key index. Both are generated, and
neither is the other.

What a 2.x actually shows from it is narrower than the schema suggests:

- `latest` reaches the badge in the bottom bar and the dialog behind it.
- `severity` decides whether that badge is quiet or red. It must be one of
  `patch`, `minor`, `major`, `critical` — **anything else silently means
  `patch`**, which is what `recommended` has been doing all along. The template
  says `major`.
- `channels` and `releaseNotes` are read **only** by the blocked-version
  screen, which needs `critical` or a listed version to appear. Neither is
  used, and the step refuses both: a working 2.x is never locked out of a
  history only that person has. Should a 2.x ever have to be blocked for real,
  it is a deliberate edit here, not a default.
- The dialog's own text is compiled into the 2.x and its button opens
  `releases/latest`. **The release notes on the tag are the warning that user
  reads**, so they are where the crossing gets explained — that the history is
  found and offered, and that nothing moves unless asked.

`minimumSupported` stays at `2.0.0`: explicit and low. It only matters through
`critical`, and that combination is refused, but a floor at the version being
tagged would put every 2.x below it the day someone changes one line.

A prerelease is not what `releases/latest` serves, so no 2.x would ever read a
candidate's copy. It is therefore built, validated and signed on every `-rc`
and **deliberately not attached**: the path gets exercised before the run that
cannot fail, and the release carries no asset whose bytes depend on what a
server answered that morning — which would collide with the Publish step's
refusal to replace a published asset with different bytes on a rerun. Exposure
begins with the first stable 3.0, whose asset every installed copy reads within
a day. On a stable tag the live manifest is fetched for one purpose only, to
refuse a version that walks it backwards, and a reply that is neither 200 nor
404 stops the release rather than guessing.

The Microsoft Store crosses differently, and deliberately: the 3.0.0 ships to
the **same SKU** the 2.x already publishes (`9NBJRZF3K856`), so those installs
update on their own, with no badge and no notes. The Homebrew cask behaves the
same way on the first stable tag that rewrites it.

### What the installer owes that user

The 3.0 reads the 2.x database when the person asks it to, and never otherwise.
On Windows the 3.0 installer:

- **Offers to uninstall the 2.x, and never deletes its data.** The old history,
  its pictures and its settings stay on disk, untouched, whatever the user
  picks. The uninstaller is explicit about it: it deletes the files the 3.0
  owns one by one — `history.db`, `blobs`, `thumbs`, `config.toml`,
  `update.json`, its two logs — and then tries `RmDir` without `/r`, so the
  shared folder survives for as long as anything of the 2.x is still in it.
- **Finds the 2.x however it was installed.** The Inno setup answers to
  `HKCU\…\Uninstall\{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}_is1`; the Store
  copy is the `rgdevment.CopyPaste-ClipboardManager` package below 3.0, asked
  of `Get-AppxPackage`. Either way the installer closes `CopyPaste.exe` before
  it removes anything, since the 2.x left running holds the shortcut the 3.0
  needs.
- **Never removes a Store copy that keeps its history inside its package.** A
  packaged app's writes to a `LocalAppData` folder it created itself land in
  `Packages\rgdevment.CopyPaste-ClipboardManager_kdjgfdc2rb3gc\LocalCache\Local\CopyPaste`,
  and `Remove-AppxPackage` deletes that folder with the package. When a
  `clipboard.db` is there, the installer only offers to close the 2.x. The 3.0
  looks for the 2.x history there too when the plain folder has none, and finds
  each picture by its name under that folder's `images`, since the path the 2.x
  wrote down is one only a packaged process could open. Not yet tried against a
  machine that only ever had the Store copy.
- **Puts back the 3.0's own start with the session.** The Inno uninstaller
  deletes the `CopyPaste` value under `HKCU\…\Run`, which is the name the 3.0
  uses too; if it pointed at `cp-gui.exe` before, it is written again.
- **Recommends starting fresh.** That is the default; bringing the history over
  is there for whoever asks.

Bringing it over happens inside the app, in Settings → Backup, not in the
installer, and it states its losses before anything is touched: how many items
there are, how many keep the styling the 2.x did store and that the rest arrive
plain, how many pictures are no longer on disk, and that nothing of the 2.x is
read more than once. Deleting the 2.x data is a separate button, behind a
confirmation, and it takes `clipboard.db`, `images`, `config`, `.initialized`,
`crash.log`, `last_cleanup.txt` and the `copypaste_*.log` files in `logs` — never
the logs the 3.0 writes beside them, and never the files the history merely
pointed at, which belong to the person.

The same applies on macOS, where the 2.x app bundle and its Application Support
folder are separate things: removing the app never touches the folder.

## Secrets and variables

The 3.0 pipeline will need these, all named as Tisty names them:

| Name                                        | Kind     | Purpose                                  |
| ------------------------------------------- | -------- | ---------------------------------------- |
| `PFX_BASE64` / `PFX_PASSWORD`                  | secret   | Signs the Windows binaries and installer. |
| `MACOS_CERTIFICATE_P12` / `MACOS_CERTIFICATE_PASSWORD`       | secret   | Signs the macOS app.                      |
| `APPLE_ID` / `APPLE_APP_PASSWORD` / `APPLE_TEAM_ID` / `APPLE_SIGNING_IDENTITY` | secret | Notarisation. |
| `TAURI_SIGNING_PRIVATE_KEY` / `..._PASSWORD` | secret   | Signs the update artifacts.              |
| `RELEASE_PRIVATE_KEY`                        | secret   | Signs the bridge manifest the 2.x reads. |
| `STORE_CLIENT_ID` / `STORE_CLIENT_SECRET` / `STORE_SELLER_ID` / `STORE_TENANT_ID` | secret | Partner Center. |
| `WINGET_TOKEN`                               | secret   | Opens the pull request in `winget-pkgs`. |
| `GIST_TOKEN`                                 | secret   | Pushes to the Homebrew tap.              |
| `STORE_APP_ID`, `STORE_PUBLISH`              | variable | Store product and its on/off switch.     |
| `WINGET_PUBLISH`                             | variable | Off until the package exists.            |
| `BRIDGE_MANIFEST`                            | variable | The manifest a 2.x reads. Off once nobody is left on the 2. |
| `MSIX_IDENTITY`, `MSIX_PUBLISHER`, `MSIX_PUBLISHER_DISPLAY` | variable | Package identity, frozen from the 2.x. |

Rotating any of these needs no code change.

## Rolling back

A tag is never deleted or re-pointed: the signature over the manifest on the
old tag cannot be revoked retroactively. Cut a new patch tag with the fix, and
if the update manifest ends up carrying `Blocked`, that list on the new signed
manifest is the authoritative signal.

## What the 2.x process left behind

- **Scoop** — `rgdevment/scoop-bucket` is not part of the 3.0 plan; winget
  covers Windows.
- **Linux formulae** — frozen at v2.11.0 in the tap. Mark them `deprecate!`
  rather than deleting them, so anyone who installed them keeps a reinstall
  path.
- **`app/pubspec.yaml`** — gone with Flutter; there is no version to pin.
