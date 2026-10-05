# Contributing to CopyPaste

Thank you for considering contributing to **CopyPaste**! 🎉

This project exists thanks to people like you. Whether it's your first open source contribution or you're an experienced developer—**everyone is welcome here**.

---

## Our Philosophy

**CopyPaste** is a personal project that grew into something bigger—a community effort. It was created by an individual developer ([@rgdevment](https://github.com/rgdevment)) with a simple idea: build the clipboard tool he always wanted to use.

We believe in:

- **Simplicity and minimalism** — Features that matter, no bloat
- **Performance and stability** — Every millisecond counts
- **Absolute privacy** — Your data is yours, always
- **Collaborative community** — We build together, not in silos
- **Valuable feedback** — Your opinion matters as much as code

**There is no premium version and no feature is ever held back** — what you install is the whole application. CopyPaste is free software, created by and for the community.

It is also dual licensed: anyone who wants to redistribute it inside a product of their own needs [separate terms](COMMERCIAL.md). Using it, deploying it across a company, or packaging it for a distribution never does.

> **Maintainers:** the full release process (tagging, manifest and store publication) lives in [RELEASING.md](RELEASING.md).

---

## 🛠 How Can I Contribute?

In many ways! Code is just one of them:

### Share Your Feedback

- **Use the app** — The most valuable feedback comes from real users
- **Report issues** — If something doesn't work, [let us know](https://github.com/rgdevment/CopyPaste/issues)
- **Suggest improvements** — Have an idea? [Open an issue](https://github.com/rgdevment/CopyPaste/issues/new)
- **Join discussions** — [GitHub Discussions](https://github.com/rgdevment/CopyPaste/discussions) is our space to talk

### Report Bugs

1. First search [Issues](https://github.com/rgdevment/CopyPaste/issues) to see if it was already reported
2. If it doesn't exist, open a new one with:
   - Clear description of the problem
   - Steps to reproduce
   - CopyPaste version and OS (with version)
   - Screenshots if applicable

### Contribute Code

1. **Fork** the repository
2. **Create a branch** from `main` (`git checkout -b feature/my-improvement`)
3. **Make your changes** following our style:
   - Clean and minimalist code
   - Code and identifiers in English
   - No comments, with no exception. CI refuses any `//` or `/*` in `crates` and
     `app/src-tauri/src`: a name that needs a comment is the wrong name
   - Tests when appropriate, and **in a file of their own** — see below
4. **Where a test goes.** Never inside the file it tests. Each file keeps its
   tests in a sibling, declared at the bottom of the file under test:

   ```rust
   #[cfg(test)]
   #[path = "watch_test.rs"]
   mod tests;
   ```

   The sibling lives in the same directory, holds the body without the
   `mod tests { }` wrapper, and is still a child module — `use super::*` and
   access to private items work exactly as if it were inline. Its name is the
   file plus `_test.rs`; when one file has several test modules, the name says
   which one it is and still ends in `_test.rs`: `store_listing_test.rs`,
   `kind_borders_test.rs`, `waking_windows_test.rs`. The suffix is not decorative:
   the ceiling refuses a declaration pointing at a name without it, so a sibling
   named anything else is measured as production code and fails at 1500 lines.
   The declaration goes **last** in the file, after the code — clippy asked for
   that while the module was inline and cannot see it any more, so
   `scripts/oversized.py --inline` does. A test module written inside the file
   fails CI, as does a `*_test.rs` that nothing declares.

5. **Make sure** all tests pass:

   ```sh
   python3 scripts/sidecar.py --debug   # the app carries the panel; build it first
   cargo clippy --workspace --all-targets
   cargo test --workspace
   cd app && npm ci && npm run lint && npm test
   ```

6. **Open a Pull Request** to `main`

### Translate

Do you speak another language? Help us bring CopyPaste to more people. Check the [localization guide](README.md#localization-help-translate-copypaste) in the README.

### Improve Documentation

Found something confusing? Missing information? Documentation is also code—PRs welcome!

### Share the Project

- Give us a star on GitHub
- Tell your friends and colleagues
- Share it on social media

---

## Style Guide

We keep the code simple and consistent:

- **Rust 2024, and the toolchain the repository pins** — `rust-toolchain.toml` decides, not your machine
- **Descriptive names** — Code should read like prose
- **No comments in the Rust** — the name says the what, and CI rejects any `//` under `crates` and `app/src-tauri/src`: whatever needs explaining goes on the record
- **`cargo fmt`, `cargo clippy` and `biome` all clean** — CI runs them on macOS and on Windows
- **KISS** — Keep It Simple, Stupid
- **DRY** — Don't Repeat Yourself

### Run the conventions before CI does

`scripts/rules.sh` holds the conventions a person can break in a second: a
comment where the code should speak for itself, `unsafe` outside the `-sys`
crates, the core printing to a terminal or reaching for a platform, Spanish in
an identifier, two crates naming an example the same, and a file grown past
what anybody reads through. `scripts/commits.sh` holds the commit convention
below. Both answer
the same whether you run them or CI does, and they say every rule that broke
rather than stopping at the first. Run either whenever you like, and if you want
them run for you:

```sh
git config core.hooksPath hooks
```

That gives you three. `pre-commit` runs the conventions, `cargo fmt --all
--check` and biome — a couple of seconds, and between them they are most of what
turns CI red. `commit-msg` weighs the message while the fix is still an
`--amend` rather than a rebase. `pre-push` runs the conventions again and the
subjects of everything you are about to send, and when what you are sending is a
tag it asks `scripts/news.sh` whether `app/src/news.json` says what changed in
that version — the screen that tells a person what is new is the only place the
app says it, and a tag is the last moment to notice it is empty. A candidate is
exempt: the screen only shows versions at or below the one running, and `3.0.0`
is above `3.0.0-rc1`. Nothing slower goes in any of
them: the suite, the build and the markdown lint are minutes, and they belong to
CI.

### Commit messages

One line, in English, and nothing else:

```text
type(scope): the concrete change
```

- **Types:** `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`,
  `build`, `ci`, `chore`, `revert`. The scope is optional.
- **At most 120 characters.**
- **No body and no trailers** — no `Co-authored-by`, no `Signed-off-by`. The
  CLA below is signed once on the pull request, not on each commit.
- **No links**, except `#123` for an issue or pull request of this repository.
- Say what changed, not how it was found.

A pull request's title follows the same rule, because the squash keeps it as
the subject on `main`.

**UI/UX:**

- Native look and feel on each platform (Windows, macOS)
- Functional minimalism
- Smooth and fluid transitions
- Respect for system Light/Dark themes

---

## License and Rights

CopyPaste is released under the **GNU General Public License v3.0 (GPL-3.0)**
and offered under separate [commercial terms](COMMERCIAL.md) to organisations
that cannot comply with it. Your contributions are licensed the same way.

### Contributor License Agreement

Because of that dual model, every contributor signs a one-time [CLA](CLA.md)
before their code can be merged. Offering commercial terms requires the right
to license the whole codebase that way, and that right has to come from each
author explicitly.

**You keep the copyright on your work.** The CLA is a licence you grant, not a
transfer of ownership.

The first time you open a Pull Request, a bot asks you to sign. Reply on that
Pull Request with exactly:

```text
I have read the CLA Document and I hereby sign the CLA
```

That is it — every later Pull Request from the same account is covered.

**Commits carry no trailers.** If your editor adds one, drop it before you
push. Section 4 of the CLA already puts the responsibility for what you submit
on you, whatever helped you write it.

**In return, the project commits that:**

- The community edition stays available under the GPL-3.0.
- Your contribution is never removed from the open source project to make it
  exclusive to a commercial edition.
- Your authorship is preserved; history is not rewritten to erase it.
- No release already published is ever retroactively withdrawn.

Read [CLA.md](CLA.md) for the full text — it is short, and worth the two
minutes before you sign it.

If you would rather not sign, you can still use CopyPaste, report bugs,
request features, discuss design, package it for your distribution, and fork
the project under the GPL-3.0. Only merging code into this repository requires
the agreement.

**Packaging CopyPaste for a distribution or package manager needs no agreement
and no commercial licence** — that is ordinary GPL redistribution, and
packagers are welcome. See [COMMERCIAL.md](COMMERCIAL.md).

---

## Code of Conduct

We want this to be a safe and welcoming space for everyone. Please read our [Code of Conduct](CODE_OF_CONDUCT.md) before participating.

**TL;DR:** Be respectful, be constructive, assume good intentions.

---

## Questions?

- **GitHub Discussions** — For general questions and conversations
- **Issues** — For specific bugs and suggestions
- **Email** — <github@apirest.cl> for sensitive matters

**Remember:** There are no stupid questions. If you have doubts, ask. We're here to help.

---

<div align="center">
  <p><strong>Thank you for being part of CopyPaste. 💙</strong></p>
  <p><em>— Mario (rgdevment)</em></p>
</div>
