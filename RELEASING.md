# Releasing CoPanda

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
| `sign-updater`       | The updater signatures over what the bundles left, and nothing else.    |
| `publish`            | The GitHub Release with every artifact, and the `manifest` branch.      |
| `verify`             | Installs what was just published and checks the update feed answers.    |
| `msstore`            | Submits the MSIX (stable tags only).                                    |
| `winget`             | Opens the pull request that adds this version to `winget-pkgs`.         |
| `homebrew`           | Rewrites `copanda` (stable) or `copanda-beta` (candidate) in the tap. |

`feed.yml` then watches the published manifest daily: a deleted asset, a
retired release or a force-pushed branch breaks the update feed silently, and
nothing else would notice.

### macOS is built, and the tag refuses to go out unsigned

`BUILD_MACOS` at the top of `release.yml` gates `bundle-macos` and is set to
`"true"`: the Apple secrets are the ones the 2.x already used. On a tag, the
first step of `bundle-macos` demands all five from the `signing` environment
(`MACOS_CERTIFICATE_P12`, `MACOS_CERTIFICATE_PASSWORD`, `APPLE_ID`,
`APPLE_APP_PASSWORD`, `APPLE_TEAM_ID`) and fails before anything is built if
one is missing; `bundle-windows` does the same with the PFX. The signing
identity is read off the imported certificate, not a secret. That is the point:
an unsigned, un-notarised `.dmg` is worse than no `.dmg`.

A `workflow_dispatch` run from a branch reaches no environment, so it builds
everything unsigned and publishes nothing: it proves the build, not the
signing. The first `-rc` tag is what proves that the sidecar is signed and the
bundle notarises.

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
never offered anything; anything else, a Homebrew copy included, installs its
own update with one click, as Tisty does. The casks say `auto_updates true`, so
Homebrew expects that, and a Homebrew copy updates itself with the button. A copy
that cannot replace itself, such as one running from the mounted `.dmg`, is told
to move to Applications first.

Four things guard the install, all of them borrowed from Tisty: the download
address must be this repository's releases on `github.com`, under its old name
or its new one, (or `objects.githubusercontent.com`) before a byte is fetched; the version is
pinned to the one the person was shown, so a feed that moves cannot hand over
another; every updater signature is bound to its version (`signer sign
--app-version`, `requireSignedVersion`), so an older release cannot be served
under a newer number, and a release whose signatures do not say it stops
before anything is published (`scripts/signed_for.sh`, run where it is signed,
before `publish`, in `verify` and every morning in `feed.yml`); and a copy running from the mounted `.dmg` refuses rather than failing
after the whole download. The panel is
stopped first, because on Windows an installer cannot replace a binary that is
running, and it is brought back if the install does not go through.

### Two updater keys, and the spare never reaches GitHub

The first updater key was lost on 2026-11-02, and every copy up to 3.4 carries
only its public half, so none of them can verify anything signed since: they
see the update, fail to install it and send the person to the website, once.
From 3.5.1 on a copy carries two public keys. The primary is in
`app/src-tauri/tauri.conf.json` and signs every release. The spare is in
`app/src-tauri/updater-spare.pub`, and `update.rs` turns to it only when a
download does not verify against the primary. Its private half lives in
1Password and an offline copy, never in a GitHub secret: it exists for the day
the primary is lost, and then it goes into the `updater` environment in the
primary's place, the next release is signed with it, and every copy since
3.5.1 takes it without anyone reinstalling. A new spare is generated that day,
and ships in that same release.

`scripts/verify_update.sh` accepts either key. `publish` refuses a release
whose signatures answer to neither, `verify` checks the same after publishing,
and `feed.yml` also accepts the retired key in `.github/updater-retired.pub`,
with a warning, so the feed 3.4.0 left is not an error until 3.5.1 replaces it.
`build.rs` refuses to build while either public key is a placeholder.

### The feed keeps the repository's old name

Every copy out there reads the feed at
`raw.githubusercontent.com/rgdevment/CopyPaste/manifest/` and downloads from
`github.com/rgdevment/CopyPaste/releases/download/`, and the 2.x reads its
bridge from `github.com/rgdevment/CopyPaste/releases/latest/download/`. Those
addresses are compiled in. So `publish`, `verify` and `feed.yml` write and read
the old name, not `github.repository`: once the repository is called CoPanda,
GitHub redirects them, and the feed keeps answering in the very URLs the copies
accept. `ours()` accepts both names, for a feed written after a new build
learns the new one.

The redirect lives only while nothing else takes the old name. **A repository
called `rgdevment/CopyPaste` must never be created again**: it would break the
update of every copy, the 2.x included.

## First time only — what a human has to do

Three channels need a decision or a human before they can be switched on.

### 1. winget

The `winget` job adds a version to a package that already exists in
`microsoft/winget-pkgs`, with Komac pinned by version and checksum. The package
`rgdevment.CoPanda` was submitted by hand in microsoft/winget-pkgs#452685,
which passed validation and has waited for a moderator since October.

Once it is merged, `WINGET_TOKEN` goes into the `channels` environment and
`vars.WINGET_PUBLISH` is set to `true`. Until then the job is skipped whole, so
a release never fails over it.

### 2. Microsoft Store

Nothing manual: the SKU is the one the 2.x already publishes, so the usual
first-submission hurdle does not apply. `vars.STORE_PUBLISH` goes on with the
3.0.0 — a decision, not an oversight; see the crossing below.

### 3. Homebrew tap

The tap repository already exists and the `homebrew` job rewrites the cask
itself, so there is nothing to write by hand. Leave the frozen Linux formulae
alone: the job must not sweep them.

The cask's `homepage` is `https://www.copanda.app/`, the product page —
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
publish the GitHub Release and the `copanda-beta` cask, and skip the Store
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
`BRIDGE_MANIFEST` variable. **The bridge is frozen.** The Ed25519 seed that
signed it was lost on 2026-11-02, and the 2.x verifies only against
`.github/v2-release-pubkey.txt`, so no new bridge can ever be signed. What is
left is the pair 3.4.0 published, kept byte for byte in `.github/bridge/`
(`release-manifest.json`, which tells a 2.x about 3.4.0, and its `.sig`). On
every stable release the step verifies that pair against the 2.x key and
attaches it unchanged, so `releases/latest/download/` keeps serving it. A 2.x
is told about 3.4.0 for good and its dialog opens `releases/latest`, which is
always the newest 3.x, so the crossing works as before. `.gitattributes` keeps
git from touching those bytes: one changed byte and every 2.x drops the file
without a word.

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
missing; the feed on the branch is a three-key index, generated on every
release.

What a 2.x actually shows from it is narrower than the schema suggests:

- `latest` reaches the badge in the bottom bar and the dialog behind it.
- `severity` decides whether that badge is quiet or red. It must be one of
  `patch`, `minor`, `major`, `critical` — **anything else silently means
  `patch`**, which is what `recommended` has been doing all along. The template
  says `major`.
- `channels` and `releaseNotes` are read **only** by the blocked-version
  screen, which needs `critical` or a listed version to appear. Neither is
  used, and `feed.yml` refuses both: a working 2.x is never locked out of a
  history only that person has. With the seed gone, a 2.x can no longer be
  blocked at all.
- The dialog's own text is compiled into the 2.x and its button opens
  `releases/latest`. **The release notes on the tag are the warning that user
  reads**, so they are where the crossing gets explained — that the history is
  found and offered, and that nothing moves unless asked.

`minimumSupported` is `2.0.0` in the frozen pair: explicit and low, and it only
matters through `critical`, which the pair does not say.

A prerelease is not what `releases/latest` serves, so no 2.x would ever read a
candidate's copy. On every `-rc` the pair is verified and **deliberately not
attached**, so the path gets exercised before the run that cannot fail.
`feed.yml` checks every morning that what is served verifies against the 2.x
key and tells it about a 3.x.

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

## Secrets, environments and variables

Every secret lives in one of three environments, and each job that reads a
secret names its environment. All three allow deployments only from tags that
match `v*`, so a branch, a pull request or a `workflow_dispatch` from a branch
never reaches them; a dispatch from a branch builds unsigned. Each job checks
its own secrets in its first step and stops there, before building anything,
when one is missing or comes without its pair.

Every secret has an item in the **Apps** vault of 1Password, titled with the
secret's exact name in GitHub and tagged **CoPanda**. Other apps keep items of
the same name in that vault, so the tag is what tells them apart, and a title
alone never finds the right one. A spare that never goes to GitHub carries the
suffix `_SPARE`.

Each secret is loaded with `gh secret set <NAME> --repo rgdevment/CopyPaste
--env <environment>`, which asks for the value to paste, or with
`< <file>` at the end for a file. A secret that already exists is replaced.

### `signing` — `bundle-windows` and `bundle-macos`

| Secret                       | 1Password item (Apps, tag CoPanda) | How to regenerate it                                                                                                      |
| ---------------------------- | ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `PFX_BASE64`                 | `PFX_BASE64`                       | Export the code-signing certificate with its private key as `.pfx`, then `base64 < cert.pfx \| tr -d '\n' > cert.b64`; load `< cert.b64`. |
| `PFX_PASSWORD`               | `PFX_PASSWORD`                     | The password chosen on that export.                                                                                       |
| `MACOS_CERTIFICATE_P12`      | `MACOS_CERTIFICATE_P12`            | Export the Developer ID Application certificate and its key from Keychain as `.p12`, then base64 it the same way.        |
| `MACOS_CERTIFICATE_PASSWORD` | `MACOS_CERTIFICATE_PASSWORD`       | The password chosen on that export.                                                                                       |
| `APPLE_ID`                   | `APPLE_ID`                         | The Apple account that notarises.                                                                                         |
| `APPLE_APP_PASSWORD`         | `APPLE_APP_PASSWORD`               | A new app-specific password at account.apple.com → Sign-In and Security.                                                  |
| `APPLE_TEAM_ID`              | `APPLE_TEAM_ID`                    | The team ID at developer.apple.com → Membership.                                                                          |

The certificates and the Apple account are the same other apps sign with, so a
new certificate or password changes in their items too.

`bundle-msix` reads no secret: it packs binaries `bundle-windows` already
signed, and the Store signs the package.

### `updater` — `sign-updater` only, behind a required reviewer

| Secret                               | 1Password item (Apps, tag CoPanda)   | How to regenerate it                                                                       |
| ------------------------------------ | ------------------------------------ | ------------------------------------------------------------------------------------------ |
| `TAURI_SIGNING_PRIVATE_KEY`          | `TAURI_SIGNING_PRIVATE_KEY`          | Not regenerated: when it is lost, the spare takes its place (see the updater keys above). |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | The password the key was made with.                                                        |

The key is the file `tauri signer generate` writes, already text: it goes in
with `< <key file>`, without `base64`.

The spare is `TAURI_SIGNING_PRIVATE_KEY_SPARE` and
`TAURI_SIGNING_PRIVATE_KEY_PASSWORD_SPARE`, and it is **never loaded into
GitHub** while the primary works. On the day it replaces the primary, its two
values go into `TAURI_SIGNING_PRIVATE_KEY` and
`TAURI_SIGNING_PRIVATE_KEY_PASSWORD` in `updater`, and a new spare is made with
`npm run tauri signer generate` in `app` that same day.

`sign-updater` builds nothing and runs nobody else's scripts; it is the only
job that ever holds the key. It runs after the bundles, so a missing key
stops the release there, before anything is published.

### `channels` — `msstore`, `homebrew` and `winget`

| Secret                | 1Password item (Apps, tag CoPanda) | How to regenerate it                                                                                       |
| --------------------- | ---------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| `STORE_TENANT_ID`     | `STORE_TENANT_ID`                  | Partner Center → Account settings → User management → Microsoft Entra applications.                       |
| `STORE_SELLER_ID`     | `STORE_SELLER_ID`                  | Partner Center → Account settings → Legal info → Seller ID.                                               |
| `STORE_CLIENT_ID`     | `STORE_CLIENT_ID`                  | The Entra application added there.                                                                        |
| `STORE_CLIENT_SECRET` | `STORE_CLIENT_SECRET`              | A new key on that application; it expires, so note the date.                                              |
| `HOMEBREW_TAP_TOKEN`  | `HOMEBREW_TAP_TOKEN`               | A fine-grained token limited to `rgdevment/homebrew-tap`, with Contents read and write.                   |
| `WINGET_TOKEN`        | `WINGET_TOKEN`                     | A classic token of `rgdevment-bot` with `public_repo`, which Komac uses to push its fork and open the PR.  |

The frozen bridge has no secret. Its two files are kept in the same vault with
the same tag, the same bytes as `.github/bridge/`.

### Add before you remove

A secret is replaced by adding the new value first and removing the old one
only after a `-rc` tag has gone green using the new one. Removing first leaves
a window in which a tag fails halfway, and a secret removed by mistake cannot
be read back: GitHub never shows a secret again, so 1Password is the only copy.
The same goes for environments: one is deleted only after its secrets are in
1Password and the jobs that used it have moved.

### Variables

| Name                                                        | Purpose                                                      |
| ----------------------------------------------------------- | ------------------------------------------------------------ |
| `STORE_APP_ID`, `STORE_PUBLISH`                             | Store product and its on/off switch.                         |
| `WINGET_PUBLISH`                                            | Off until the package exists.                                |
| `HOMEBREW_PUBLISH`                                          | `false` keeps the tap as it is.                              |
| `BRIDGE_MANIFEST`                                           | Attaches the frozen bridge. Off once nobody is left on the 2. |
| `MSIX_IDENTITY`, `MSIX_PUBLISHER`, `MSIX_PUBLISHER_DISPLAY` | Package identity, frozen from the 2.x.                       |

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
