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
| `publish`            | The GitHub Release with every artifact and the update manifest.         |
| `verify`             | Installs what was just published and checks the update feed answers.    |
| `msstore`            | Submits the MSIX (stable tags only).                                    |
| `winget`             | Opens the pull request that adds this version to `winget-pkgs`.         |
| `homebrew`           | Rewrites the cask in `rgdevment/homebrew-tap`.                          |

`feed.yml` then watches the published manifest daily: a deleted asset, a
retired release or a force-pushed branch breaks the update feed silently, and
nothing else would notice.

### macOS is built, and the tag refuses to go out unsigned

`BUILD_MACOS` at the top of `release.yml` gates `bundle-macos` and is set to
`"true"`: the Apple secrets are the ones the 2.x already used. With it on, the
`version` job demands all six (`MACOS_CERTIFICATE_P12`,
`MACOS_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`,
`APPLE_APP_PASSWORD`, `APPLE_TEAM_ID`) and fails the tag before anything is
built if one is missing. That is the point: an unsigned, un-notarised `.dmg` is
worse than no `.dmg`.

The job has never run yet, so the first tag is also the first proof that the
sidecar is signed and the bundle notarises. Cut it with `workflow_dispatch`
before cutting it for real.

### What the 3.0 does not carry yet

Two pieces, in the order they will be built, each on its own branch:

1. **Bringing the 2.x history over.** `former` finds `clipboard.db` and weighs
   it; the two buttons that would act on it are still disabled.
2. **The bridge for whoever stays on the 2.x.** See the crossing below.

### The app does check for updates

`app/src-tauri/src/update.rs` reads the feed the `publish` job writes, once a
day and whenever the person asks. What it does then depends on where the copy
came from, which it reads off its own path: a copy under `WindowsApps` is the
Store's to update and is never offered anything; one under a `Caskroom` is
told the `brew` command rather than handed an installer; anything else
installs its own update.

Three things guard the install, all of them borrowed from Tisty: the download
address must be on `github.com` or `objects.githubusercontent.com` before a
byte is fetched, the version is pinned to the one the person was shown so a
feed that moves cannot hand over another, and a copy running from the mounted
`.dmg` refuses rather than failing after the whole download. The panel is
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

Pre-releases (`v3.1.0-rc1`, `-beta1`) publish the GitHub Release and skip the
Store, winget and Homebrew.

## Crossing from the 2.x — decided 2026-09-23

Every installed 2.x copy polls
`https://github.com/rgdevment/CopyPaste/releases/latest/download/release-manifest.json`
and its `.sig`, verifies the signature against an Ed25519 public key bundled in
the app, and caches the answer for fifteen days
(`app/lib/services/release_manifest_service.dart` on `v2-stable`). Left with no
file, that user never learns the 3.0 exists. So the 3.0 keeps speaking both
languages for a while.

**The bridge is built from the 2.x, not from the 3.0** (decided 28/09/2026). A
new 2.x release is published that reads the `manifest` branch, so that whoever
stays on the 2 hears about the 3.0 through the channel they already have. It is
the last thing to be done, once the 3.0 is ready — not before.

`RELEASE_PRIVATE_KEY` is still in the repository's secrets, so anything that
has to be signed the way the 2.x expects can be. Three rules govern what that
release says:

- `severity: recommended`. Never `critical`: that paints a full-screen block
  over a working 2.x, and this user has to be free to stay.
- `Min-Supported` set explicitly and low. Left to its default it takes the
  version being tagged — `3.0.0` — which puts every 2.x below the floor.
- `releaseNotes`, in both languages, **says that the history is not migrated**.
  It is the only warning that user sees before jumping.

Each installed copy polls every fifteen days, so there is no hurry once the
bridge release is out: every one of them gets several windows to notice.

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
| `STORE_CLIENT_ID` / `STORE_CLIENT_SECRET` / `STORE_SELLER_ID` / `STORE_TENANT_ID` | secret | Partner Center. |
| `WINGET_TOKEN`                               | secret   | Opens the pull request in `winget-pkgs`. |
| `GIST_TOKEN`                                 | secret   | Pushes to the Homebrew tap.              |
| `STORE_APP_ID`, `STORE_PUBLISH`              | variable | Store product and its on/off switch.     |
| `WINGET_PUBLISH`                             | variable | Off until the package exists.            |
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
