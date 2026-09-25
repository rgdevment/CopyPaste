# Releasing CopyPaste

**Status (2026-09-23): the 3.0 release pipeline does not exist yet.**
`.github/workflows/release.yml` was part of the 2.x tree and is gone; the jobs
this document used to describe (`build-windows`, `github-release`,
`publish-release-manifest`, `update-scoop-bucket`) no longer run anywhere.
What follows is the shape the 3.0 pipeline will have — modelled on the one in
[rgdevment/Tisty](https://github.com/rgdevment/Tisty), which is audited and in
production — plus the steps that are manual the first time and cannot be
automated away.

## The pipeline, once it exists

A `v*` tag on `main` fans out, in this order, with every stage gated on the one
before it:

| Stage                | What it produces                                                       |
| -------------------- | ---------------------------------------------------------------------- |
| `gate`               | Style and prose, so a tag never publishes an unformatted tree.          |
| `tested`             | Proof the commit was green when it landed.                              |
| `version`            | The version resolved from the tag, used by every stage after it.        |
| `build-windows`      | `cp-panel` sidecar + the Tauri app, signed with the PFX certificate.    |
| `build-macos`        | The same, per architecture, signed and notarised.                       |
| `bundle-windows`     | NSIS installer.                                                         |
| `bundle-macos`       | `.dmg`.                                                                 |
| `bundle-msix`        | MSIX for the Microsoft Store.                                           |
| `publish`            | The GitHub Release with every artifact and the update manifest.         |
| `verify`             | Installs what was just published and checks the update feed answers.    |
| `msstore`            | Submits the MSIX (stable tags only).                                    |
| `winget`             | Opens the pull request that adds this version to `winget-pkgs`.         |
| `homebrew`           | Rewrites the cask in `rgdevment/homebrew-tap`.                          |

`feed.yml` then watches the published manifest daily: a deleted asset, a
retired release or a force-pushed branch breaks the update feed silently, and
nothing else would notice.

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

**The 3.x publishes the old manifest alongside its own update feed, from
`v3.0.0` until `v3.1.0`.** `RELEASE_PRIVATE_KEY` is still in the repository's
secrets, so the file can be signed exactly as the 2.x expects. Three rules
govern what goes in it:

- `severity: recommended`. Never `critical`: that paints a full-screen block
  over a working 2.x, and this user has to be free to stay.
- `Min-Supported` set explicitly and low. Left to its default it takes the
  version being tagged — `3.0.0` — which puts every 2.x below the floor.
- `releaseNotes`, in both languages, **says that the history is not migrated**.
  It is the only warning that user sees before jumping.

Retirement needs no work: the file is served from `releases/latest/download/`,
so the first 3.1 release that does not attach it answers 404 and the 2.x goes
quiet. Until then each copy gets several fifteen-day windows to notice.

The Microsoft Store crosses differently, and deliberately: the 3.0.0 ships to
the **same SKU** the 2.x already publishes (`9NBJRZF3K856`), so those installs
update on their own, with no badge and no notes. The Homebrew cask behaves the
same way on the first stable tag that rewrites it.

### What the installer owes that user

The 3.0 does not read the 2.x database, and installing over it must not pretend
otherwise. On Windows the 3.0 installer:

- **Offers to uninstall the 2.x, and never deletes its data.** The old history,
  its blobs and its settings stay on disk, untouched, whatever the user picks.
- **Offers a backup before anything else** — the 2.x `.cpbackup`, written where
  the user chooses — so leaving is always reversible.
- **May offer a migration, and states its losses up front.** What crosses and
  what does not is decided when it is built; what is not allowed is a migration
  that looks complete and is not.
- **Recommends starting fresh.** That is the default and the recommended path;
  the migration is the exception for whoever asks for it.

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
