# Third-party notices

CopyPaste is GPL-3.0-only. What follows was copied into CopyPaste's own source
and carries its own licence, which allows that.

CopyPaste also bundles other people's work rather than copying it. The parts
that do the heavy lifting are [Slint](https://slint.dev) with
[Skia](https://skia.org) for the panel, [SQLite](https://sqlite.org) through
[rusqlite](https://github.com/rusqlite/rusqlite) for the history,
[image](https://github.com/image-rs/image) for the thumbnails,
[BLAKE3](https://github.com/BLAKE3-team/BLAKE3) and
[xxHash](https://github.com/Cyan4973/xxHash) for the fingerprints, and the
[windows](https://github.com/microsoft/windows-rs) and
[objc2](https://github.com/madsmtm/objc2) crates for the two platforms. Each
one is permissively licensed, and `Cargo.lock` names every last transitive one
with its version.

## Lucide Icons

The icons drawn in the panel, copied file by file into
`crates/cp-panel/assets/icons/` so that nothing is ever fetched at runtime.

- Source: <https://lucide.dev>
- Licence: ISC, in
  [crates/cp-panel/assets/icons/LICENSE](crates/cp-panel/assets/icons/LICENSE)

Lucide is itself a fork of [Feather](https://feathericons.com) (MIT), and the
ISC licence text bundled with the icons carries both notices.
