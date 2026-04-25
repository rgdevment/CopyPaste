# Releasing CopyPaste

This document describes the end-to-end release process: what to change, how
to tag, what the pipeline does for you, and what (if anything) you still need
to do by hand.

## TL;DR — cutting a normal release

1. Bump the version in `app/pubspec.yaml` on `main` and merge.
2. Create and push an **annotated, signed** tag. The tag message body
   carries the release metadata through `Key: value` trailers.
3. Wait for GitHub Actions to finish — artifacts, manifest, stores and OBS
   are all updated automatically.

**You do not need to edit `release-manifest.json` for a normal release.**
The tag message is the single source of truth per release; the pipeline
rewrites the manifest from it before signing and publishing.

```sh
git checkout main && git pull
TAG=v2.4.0
git tag -s "$TAG" -m "$TAG

Recommended release. Notes go here.

Severity: recommended
Min-Supported: 2.3.0"
git push origin "$TAG"
```

That's it. Go watch the Actions tab.

## What the pipeline does on tag push

On every `v*` tag pushed to `main`, [.github/workflows/release.yml](.github/workflows/release.yml)
fans out to:

| Job                          | What it produces                                                        |
| ---------------------------- | ----------------------------------------------------------------------- |
| `build-windows`              | Signed `*_Setup.exe` + MSIX store bundle.                               |
| `build-macos`                | Universal `*.dmg`.                                                      |
| `build-linux`                | `*.AppImage` + `.zsync`, `*.deb`, `*.rpm`, `SHA256SUMS`, portable tarball.|
| `github-release`             | Publishes all artifacts to a GitHub Release on the tag.                 |
| `publish-release-manifest`   | Patches, signs (Ed25519) and uploads `release-manifest.json(.sig)`.     |
| `publish-to-store`           | Submits MSIX to the Microsoft Store (stable tags only, no `-rc`).       |
| `publish-obs`                | Commits rendered `_service`, `.spec` and `debian.tar.xz` to OBS.        |

Legacy (during the OBS transition): `release-linux.yml` still publishes to
Cloudsmith so existing users don't break. It will be removed once OBS has
produced two green releases.

## Release manifest — what the pipeline overrides vs. what you own

`release-manifest.json` is versioned in the repo but **most of it is
rewritten by the pipeline at tag time**. You only need to edit the parts
you actually want to change.

### Fields the pipeline always overrides

- `latest` ← the version extracted from the tag name.
- `releaseNotes.en.summary` / `releaseNotes.es.summary` ← auto-generated
  from tag + severity.
- `releaseNotes.en.url` / `releaseNotes.es.url` ← point to the GitHub
  Release page for the tag.
- `channels.msstore.url` ← injected from the `STORE_APP_ID` repository
  variable.
- `severity`, `minimumSupported`, `blockedVersions` ← rewritten from tag
  trailers (see below). Defaults apply when no trailer is provided.

### Trailer-driven fields (optional overrides per release)

The pipeline reads trailers from the **tag message body**. Format is
`Key: value`, one per line, anywhere in the body.

| Trailer          | Values                               | Default if missing          |
| ---------------- | ------------------------------------ | --------------------------- |
| `Severity:`      | `recommended` · `critical` · `patch` | `recommended`               |
| `Min-Supported:` | A version string, e.g. `2.3.0`       | The new version being tagged|
| `Blocked:`       | Comma-separated list, e.g. `2.2.6, 2.2.7` | `[]` (empty list)      |

The in-repo `release-manifest.json` is there for local testing and as a
static fallback; production values come from the tag.

### Fields you own (rarely change)

- `schema` — only bump if the manifest format itself changes.
- `channels.*` URLs and commands (other than `msstore.url`) — edit only
  when the install channel itself changes (e.g. adding a new OS channel).

## Severity and how the app reacts

Only three severity values are accepted. There is **no** `optional` or
`silent` — use `patch` if you don't want to bother users, or `recommended`
if you want a visible badge.

| Severity      | UI effect                                                         | When to use it                                                                                     |
| ------------- | ----------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| `patch`       | No badge, no nagging. Users only see it if they open Settings.    | Cosmetic fixes, internal refactors, test builds, releases that don't affect the user they're on.  |
| `recommended` | Blue "Update available" badge in the main screen. **Default.**    | Any normal release: new features, bug fixes, improvements to some platforms but not others.       |
| `critical`    | Full-screen block in standalone builds listing direct install URLs. MS Store builds are never blocked. | Security issue, data corruption, broken auto-update, anything where staying on the old version is unsafe. |

Mark a release `critical` **only** when staying on older versions is
unsafe. Whenever you use `critical`, consider whether to also fill
`Blocked:` and/or bump `Min-Supported:` to close the door on the bad
version(s).

### `Min-Supported` and `Blocked` — how the block works

These are two different mechanisms:

- **`Min-Supported:`** — **range-based floor.** Any version strictly below
  it is blocked. This is the normal way to drop support for old versions.
- **`Blocked:`** — **explicit, per-version.** A comma-separated list of
  specific versions that are blocked, regardless of where `Min-Supported`
  sits. Use this to surgically revoke a broken release without affecting
  anything else.

Both default to "do not change the manifest" when the trailer is absent,
so for a normal release you typically set only `Severity:` and
`Min-Supported:` (or nothing at all, and let the defaults ride).

## Examples

### Normal recommended release

```text
v2.4.0

Adds OBS repos and AppImage auto-update. Full notes: …

Severity: recommended
Min-Supported: 2.3.0
```

### Release that only improves one platform

When the new version mostly affects Linux (or any single platform), you
still tag `recommended` — the badge encourages the update without being
alarming, and users who don't care about the platform-specific changes
can ignore it. There is no `optional` severity.

```text
v2.4.0

Linux-only: new OBS apt/dnf repos and self-updating AppImage.
Windows and macOS unchanged.

Severity: recommended
Min-Supported: 2.3.0
```

If you truly don't want to surface the update at all, use `patch`:

```text
v2.4.1

Linux packaging polish. No user-facing changes on Windows/macOS.

Severity: patch
```

### Critical security release

```text
v2.4.2

Critical fix for CVE-XXXX in the clipboard listener.

Severity: critical
Min-Supported: 2.4.2
Blocked: 2.4.0, 2.4.1
```

### Pre-release / RC (tag ends with `-rc1`, `-beta1`, etc.)

```text
v2.5.0-rc1

Internal testing build. Not a real release.

Severity: patch
```

Pre-releases skip Microsoft Store publishing automatically (the pipeline
checks for a dash in the version).

## Pre-release checklist

- [ ] `main` is green on CI.
- [ ] Version bumped in `app/pubspec.yaml`.
- [ ] `CHANGELOG`/release notes drafted (they go into the tag message body).
- [ ] Smoke-tested locally on at least one platform.
- [ ] `release-manifest.json` defaults look sane on disk (severity
      `recommended`, no dangling `blockedVersions`).
- [ ] Tag is **annotated and signed** (`git tag -s`).

## Post-release checklist

- [ ] GitHub Release has all six Linux artifacts, both Windows installers
      (setup + MSIX), `.dmg`, plus `release-manifest.json(.sig)`.
- [ ] Microsoft Store submission is in "certification" within 15 min of
      the tag (stable only).
- [ ] OBS build results are green at
      `https://build.opensuse.org/package/show/home:rgdevment/copypaste`.
      First build of a new tag may take 10–20 min per target.
- [ ] Homebrew tap (`rgdevment/homebrew-tap`) updated — currently manual;
      see the tap repo for instructions.
- [ ] App started on your machine shows the right "Update available"
      badge severity (or no badge, if `patch`).

## Things that still need manual work (and why)

- **Homebrew tap** — requires a push to a separate repo. Can be automated
  later with `brew bump-formula-pr`.
- **OBS first-time project setup** — the project, package and enabled
  targets were created by hand once; see [packaging/obs/README.md](packaging/obs/README.md).
  After bootstrap, every tag flows automatically.
- **Microsoft Store first-time submission per SKU** — the Store requires
  a human to accept the submission the first time. Subsequent tags go
  through automatically.

## Rolling back

If a release turns out bad **after** the tag is out:

1. Cut a new patch tag immediately (e.g. `v2.4.1`) with the fix.
2. In the tag body, set `Severity: critical` and `Blocked: 2.4.0` to
   push every 2.4.0 user to upgrade.
3. Do **not** delete or re-point the old tag — the Ed25519 signature over
   `release-manifest.json` on the old tag cannot be revoked retroactively.
   The `Blocked` list on the new, signed manifest is the authoritative
   signal.

## Secrets and variables used by the release pipeline

| Name                    | Where                    | Purpose                                  |
| ----------------------- | ------------------------ | ---------------------------------------- |
| `RELEASE_PRIVATE_KEY`   | Actions secret           | Signs `release-manifest.json`.           |
| `STORE_APP_ID`          | Actions variable         | Microsoft Store product ID.              |
| `CLOUDSMITH_API_KEY`    | Actions secret (legacy)  | Publishes deb/rpm to Cloudsmith.         |
| `OBS_USERNAME`          | Actions secret           | OBS account for `osc`.                   |
| `OBS_PASSWORD`          | Actions secret           | OBS password / token for `osc`.          |
| `GITHUB_TOKEN`          | Built-in                 | Releases, uploads, etc.                  |

Rotating any of these does not require code changes.
