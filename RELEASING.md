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

None of these can be automated on the first release. All three are one-off.

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

The Store requires a human to accept the first submission per SKU. Later tags
go through the Partner Center API without intervention, gated on
`vars.STORE_PUBLISH`.

### 3. Homebrew tap

The first cask in `rgdevment/homebrew-tap` is written by hand. From then on the
`homebrew` job rewrites it on every stable tag.

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

## Update manifest — decision still open

Two models are on the table and the 3.0 has not committed to either:

- **What Tisty does:** the Tauri updater, signed with minisign, reading a
  `latest.json` published to a `manifest` branch. The app offers the update and
  installs it; `feed.yml` proves the feed still answers.
- **What the 2.x did:** `release-manifest.json`, signed with Ed25519, carrying
  `severity`, `Min-Supported` and `Blocked` per release, which let a bad
  version be revoked and let a critical release block older clients.

The second is strictly more powerful and strictly more code. The file is still
versioned in this repo, so the decision costs nothing until the pipeline is
written. Decide before `publish` is implemented — it is what the job signs.

## Secrets and variables

The 3.0 pipeline will need these, all named as Tisty names them:

| Name                                        | Kind     | Purpose                                  |
| ------------------------------------------- | -------- | ---------------------------------------- |
| `PFX_BASE` / `PFX_PASSWORD`                  | secret   | Signs the Windows binaries and installer. |
| `MACOS_CERTIFICATE_P` / `..._PASSWORD`       | secret   | Signs the macOS app.                      |
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
