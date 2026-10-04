# Commercial licensing

CopyPaste is dual licensed.

- **GPL-3.0** — free for everyone, forever. See [LICENSE](LICENSE).
- **Commercial licence** — for organisations that cannot accept the GPL's
  terms. Contact <github@apirest.cl>.

Same software, same features. The only thing you buy is a different set of
obligations.

## Do you actually need one?

Almost certainly not. The GPL asks something of you when you **distribute**
the software or a work derived from it. Using it does not trigger anything, no
matter how many people use it or for how long.

**You do not need a commercial licence to:**

- Install and use CopyPaste at home or at work, on any number of machines.
- Deploy it across an entire organisation, including internal IT rollouts.
- Read, audit, or fork the source.
- Modify it for your own internal use, without publishing those changes.
- Run modified code as an internal service — the GPL has no network clause.
- Contribute changes back.

If you are a company wondering whether rolling this out to your staff needs a
licence: it does not. Internal use is not distribution.

### Packaging it for a distribution or package manager

**You do not need a commercial licence, and you never will.** Building
CopyPaste for any distribution channel — a package manager, a software
repository, or your own build pipeline — is exactly the redistribution the GPL
is designed to permit. Ship the package, keep the licence notices and make the
source available as the GPL requires, and you are done.

Packagers are welcome here. If something about the build makes your life
harder, open an issue — that is a bug worth fixing.

**You likely do need one to:**

- Ship CopyPaste, or code derived from it, inside a **product you distribute
  to others**, without licensing that product under the GPL.
- Redistribute it under **your own brand** without the GPL's source disclosure
  requirement. The notices of the third-party work inside it still go with it;
  see [Third-party components](#third-party-components).
- Preinstall it on computers you sell. A dedicated device — a kiosk, a
  terminal, a machine built for one job — also needs a licence for Slint; see
  below.
- Satisfy a policy or contract that **forbids copyleft** dependencies in what
  you ship.

If you are unsure which side you fall on, ask. A short description of what you
intend to do is usually enough to answer it, and the answer is often "you are
fine, carry on".

## What a commercial licence gives you

- The right to use, modify, and redistribute CopyPaste **without the GPL's
  copyleft obligations** — no requirement to publish your modifications or to
  license your own product under the GPL.
- Written permission you can hand to your legal or procurement team.

It covers the code the project owns, which is all of CopyPaste's own source.
It cannot cover the work of others that CopyPaste is built on.

Terms, scope, and price are agreed per case rather than published, because a
single-seat integration and an OEM redistribution are not the same deal.
Contact <github@apirest.cl> with what you want to do and the scale of it.

## Third-party components

CopyPaste is built on other people's open source work, listed with its licence
in [THIRD-PARTY-BUNDLED.md](THIRD-PARTY-BUNDLED.md). Those licences come with it
whichever licence you hold for CopyPaste itself.

- **Permissive licences** — MIT, Apache-2.0, BSD, ISC, Zlib, BSL-1.0 and
  Unicode-3.0, which is nearly all of it. They allow a closed product, and they
  ask for their notices to ship with it. CopyPaste already shows them under
  **About → Third-party notices**; keep that, or carry the file another way.
- **MPL-2.0** — the `selectors` crate, which Tauri brings in. Its copyleft stops
  at its own files: change them and you publish those files, and either way you
  tell recipients where its source is.
- **Slint** — the toolkit the panel is drawn with. CopyPaste uses it under
  GPL-3.0 and cannot pass it on under other terms. A closed desktop product can
  use it at no cost under the
  [Slint Royalty-free Desktop, Mobile, and Web Applications License 2.0](https://github.com/slint-ui/slint/blob/master/LICENSES/LicenseRef-Slint-Royalty-free-2.0.md),
  which asks for attribution in your product: the `AboutSlint` widget in its
  About screen, or the Made with Slint badge on the page it is downloaded from.
  That licence does not cover embedded systems, an application that exposes
  Slint's APIs, or Slint on its own; those need a licence from SixtyFPS GmbH,
  who make Slint.

Every commercial agreement for CopyPaste says this in writing: the licensee
distributes Slint under the Slint Royalty-free License 2.0 and meets its
attribution, or holds a Slint licence of their own, and ships the third-party
notices with the product.

## Store distribution

The GPL's section 10 forbids imposing further restrictions on recipients,
which conflicts with the terms of some application stores — Apple's App Store
being the well-known case. For the code the project owns, that conflict binds
**licensees**, not the copyright holder: a third party cannot publish CopyPaste
there, and the project itself can, under separate terms it grants to itself.

That reaches only the project's own code. Slint comes to CopyPaste under the
GPL as well, so a build for such a store takes Slint under its royalty-free
licence instead, whose attribution the README already carries.

Third-party package repositories and the Microsoft Store are unaffected: both
defer to the software's own licence.

## Why the project is set up this way

The dual model exists so CopyPaste can stay genuinely free for the people who
use it, while commercial redistribution that would otherwise contribute
nothing back has a way to support it. Whichever side you are on, the GPL
edition is not a crippled version — it is the same application, and it stays
that way.

## For contributors

Dual licensing only works if the project can license every merged line under
both sets of terms, which is why contributions require a signed
[CLA](CLA.md). You keep the copyright on your work, and the commitments in
section 6 of that document bind the project in return. See
[CONTRIBUTING.md](CONTRIBUTING.md).
