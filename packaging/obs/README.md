# OBS packaging — `home:rgdevment/copypaste`

This directory holds the source files OBS (`build.opensuse.org`) consumes
to build native `.deb` and `.rpm` packages from the prebuilt portable
tarball published on each GitHub Release
(`CopyPaste-<version>-linux-x64.tar.gz`).

## Files

| File                     | Purpose                                                    |
| ------------------------ | ---------------------------------------------------------- |
| `_service`               | Tells OBS to download the upstream tarball at build time.  |
| `copypaste.spec`         | RPM spec used for Fedora and openSUSE Tumbleweed targets.  |
| `copypaste.dsc`          | Debian source description used for Debian/Ubuntu targets.  |
| `debian/`                | Debian packaging metadata (control, rules, changelog, …).  |

The literal `@VERSION@` token in `_service`, `copypaste.spec`,
`copypaste.dsc` and `debian/changelog` is substituted at release time by
the GitHub Actions job `publish-obs` in
`.github/workflows/release-linux.yml`, which then commits the rendered
files into the OBS package via `osc`.

## How the build works

1. CI publishes the GitHub Release with
   `CopyPaste-<version>-linux-x64.tar.gz` containing the Flutter bundle
   plus `LICENSE`, `packaging/com.rgdevment.copypaste.desktop` and
   `packaging/icon_app_256.png`.
2. The `publish-obs` job renders the templates and pushes them to
   `home:rgdevment/copypaste` on `build.opensuse.org`.
3. OBS downloads the tarball through `_service` and rebuilds the
   package against every enabled target. Built repositories appear at
   `https://download.opensuse.org/repositories/home:/rgdevment/<target>/`.

The tarball is **not** rebuilt by OBS — it is only repackaged. This
keeps OBS workers free of the Flutter toolchain and matches the pattern
used by other Flutter/Electron desktop apps published on OBS.

## Targets

| Family    | OBS target name              |
| --------- | ---------------------------- |
| Debian    | `Debian_12`, `Debian_13`     |
| Ubuntu    | `xUbuntu_22.04`, `xUbuntu_24.04` |
| Fedora    | `Fedora_40`, `Fedora_41`     |
| openSUSE  | `openSUSE_Tumbleweed`        |

End-user installation instructions live in the project README.
