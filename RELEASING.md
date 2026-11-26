# Releasing CoPanda

`.github/workflows/release.yml` builds, signs and publishes CoPanda. What
follows is what it does and how a release is cut.

## The pipeline

A `v*` tag on `main` fans out, in this order, with every stage gated on the one
before it:

| Stage                | What it produces                                                       |
| -------------------- | ---------------------------------------------------------------------- |
| `gate`               | Style and prose, so a tag never publishes an unformatted tree.          |
| `tested`             | Proof the commit was green when it landed.                              |
| `version`            | The version resolved from the tag, used by every stage after it.        |
| `bundle-windows`     | `cp-panel` sidecar + the Tauri app, signed, as an NSIS installer.              |
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
`"true"`. On a tag, the first step of `bundle-macos` checks that the `signing`
environment holds everything it needs to sign and notarise, and fails before
anything is built if it does not; `bundle-windows` does the same for Windows.
That is the point: an unsigned, un-notarised `.dmg` is worse than no `.dmg`.

A `workflow_dispatch` run from a branch reaches no environment, so it builds
everything unsigned and publishes nothing: it proves the build, not the
signing. A `-rc` tag is what proves that the sidecar is signed and the bundle
notarises.

### The app checks for updates

`app/src-tauri/src/update.rs` looks once a day and whenever the person asks.
The `publish` job keeps three files on the `manifest` branch:
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
own update with one click. A copy that cannot replace itself, such as one
running from the mounted `.dmg`, is told to move to Applications first.

Four things guard the install: the download address must be this repository's
releases on `github.com` (or `objects.githubusercontent.com`) before a byte is
fetched; the version is pinned to the one the person was shown, so a feed that
moves cannot hand over another; every updater signature is bound to its
version (`signer sign --app-version`, `requireSignedVersion`), so an older
release cannot be served under a newer number, and a release whose signatures
do not say it stops before anything is published (`scripts/signed_for.sh`, run
where it is signed, before `publish`, in `verify` and every morning in
`feed.yml`); and a copy running from the mounted `.dmg` refuses rather than
failing after the whole download. The panel is stopped first, because on
Windows an installer cannot replace a binary that is running, and it is brought
back if the install does not go through.

## Channels

### winget

The `winget` job adds each stable version to `rgdevment.CoPanda` in
`microsoft/winget-pkgs`, with Komac pinned by version and checksum. It runs
only while `vars.WINGET_PUBLISH` is `true`, and is skipped whole otherwise.

### Microsoft Store

The `msstore` job submits the MSIX on stable tags while `vars.STORE_PUBLISH` is
`true`. The listing appears once the Store's reviewers accept it.

### Homebrew tap

The `homebrew` job rewrites the cask in the tap itself, so there is nothing to
write by hand. Leave the Linux formulae in the tap alone: the job must not
sweep them.

The casks do not declare `auto_updates`, so `brew upgrade` updates the app as
well as its own button. After the button, Homebrew still counts the copy as
outdated and may reinstall the same version, which does no harm.

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

## Environments and variables

`bundle-windows` and `bundle-macos` use the `signing` environment,
`sign-updater` uses `updater`, behind a required reviewer, and `msstore`,
`homebrew` and `winget` use `channels`. All three take only `v*` tags.

| Variable                                                    | Purpose                                                   |
| ----------------------------------------------------------- | --------------------------------------------------------- |
| `STORE_APP_ID`, `STORE_PUBLISH`                             | Store product and its on/off switch.                      |
| `WINGET_PUBLISH`                                            | Turns the `winget` job on.                                |
| `HOMEBREW_PUBLISH`                                          | `false` keeps the tap as it is.                           |
| `BRIDGE_MANIFEST`                                           | Attaches the manifest a 2.x reads to stable releases.     |
| `MSIX_IDENTITY`, `MSIX_PUBLISHER`, `MSIX_PUBLISHER_DISPLAY` | Package identity of the Store listing.                    |

## Rolling back

A tag is never deleted or re-pointed: the signature over the manifest on the
old tag cannot be revoked retroactively. Cut a new patch tag with the fix, and
if the update manifest ends up carrying `Blocked`, that list on the new signed
manifest is the authoritative signal.
