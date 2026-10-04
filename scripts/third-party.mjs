import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const out = join(root, "THIRD-PARTY-BUNDLED.md");
const texts = join(root, "THIRD-PARTY-LICENSES.md");
const REPO = "https://github.com/rgdevment/CopyPaste/blob/main";

const told = (pkg) =>
  typeof pkg.license === "string"
    ? pkg.license
    : (pkg.license?.type ?? pkg.licenses?.map((one) => one.type).join(" OR ") ?? "see the package");

const MIT = (who) => `MIT License

Copyright (c) ${who}

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.`;

const ISC = (who) => `ISC License

Copyright (c) ${who}

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted, provided that the above
copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
PERFORMANCE OF THIS SOFTWARE.`;

const offered = (licence) =>
  String(licence)
    .toUpperCase()
    .split(/[()\s/]+|\bOR\b|\bAND\b/)
    .filter(Boolean);

const held = (at, pkg) => {
  for (const one of readdirSync(at)
    .filter((name) => /\.spdx$/i.test(name))
    .sort()) {
    const said = readFileSync(join(at, one), "utf8").match(/^PackageCopyrightText:\s*(.+)$/m);
    if (said) return said[1].trim();
  }
  if (typeof pkg.author === "string") return pkg.author;
  return pkg.author?.name ?? null;
};

const drafted = (at, pkg, licence) => {
  const who = held(at, pkg);
  if (!who) return null;
  const parts = offered(licence);
  if (parts.includes("MIT")) return MIT(who);
  if (parts.includes("ISC")) return ISC(who);
  return null;
};

const noticed = (at) => {
  if (!existsSync(at)) return null;
  const named = readdirSync(at)
    .filter((one) => /^(licen[cs]e|copying)/i.test(one))
    .filter((one) => !/\.spdx$/i.test(one))
    .filter((one) => statSync(join(at, one)).isFile())
    .sort();
  if (named.length > 0) {
    return named
      .map((one) => `${one}\n\n${readFileSync(join(at, one), "utf8").trim()}`)
      .join("\n\n");
  }
  const where = join(at, "package.json");
  if (!existsSync(where)) return null;
  const pkg = JSON.parse(readFileSync(where, "utf8"));
  return drafted(at, pkg, told(pkg));
};

const shipped = () => {
  const lock = JSON.parse(readFileSync(join(root, "app", "package-lock.json"), "utf8"));
  const seen = new Map();
  for (const [at, one] of Object.entries(lock.packages ?? {})) {
    if (!at || one.dev || one.devOptional || one.extraneous) continue;
    const name = one.name ?? at.slice(at.lastIndexOf("node_modules/") + 13);
    if (!name) continue;
    const version = one.version ?? "?";
    const key = `${name}@${version}`;
    if (seen.has(key)) continue;
    seen.set(key, {
      name,
      version,
      licence: one.license ?? "see the package",
      notice: noticed(join(root, "app", at)),
    });
  }
  return seen;
};

const SHIPPED = ["x86_64-pc-windows-msvc", "aarch64-apple-darwin", "x86_64-apple-darwin"];

const crates = () => {
  const seen = new Map();
  for (const triple of SHIPPED) {
    const said = execFileSync(
      "cargo",
      [
        "tree",
        "--locked",
        "--workspace",
        "--edges",
        "normal,no-proc-macro",
        "--target",
        triple,
        "--prefix",
        "none",
        "--format",
        "{p}|{l}",
      ],
      { cwd: root, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
    );
    for (const line of said.split(/\r?\n/)) {
      const one = /^(\S+) v(\S+)(?: \((.+?)\))?\|(.*?)(?: \(\*\))?$/.exec(line);
      if (!one) continue;
      const [, name, version, from, licence] = one;
      const key = `${name}@${version}`;
      if ((from && existsSync(from)) || seen.has(key)) continue;
      seen.set(key, { name, version, licence: licence || "see the crate" });
    }
  }
  return seen;
};

const BSD3 = (who) => `BSD 3-Clause License

Copyright (c) ${who}

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

3. Neither the name of the copyright holder nor the names of its
   contributors may be used to endorse or promote products derived from
   this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.`;

const asWritten = (text) => text.replace(/\r\n/g, "\n");

const canonical = (spdx) =>
  asWritten(readFileSync(join(root, "scripts", "licences", `${spdx}.txt`), "utf8")).trim();

const manifests = () => {
  const said = execFileSync("cargo", ["metadata", "--format-version", "1", "--locked"], {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  return new Map(JSON.parse(said).packages.map((one) => [`${one.name}@${one.version}`, one]));
};

const filesIn = (at, pattern) =>
  existsSync(at) && statSync(at).isDirectory()
    ? readdirSync(at)
        .filter((one) => pattern.test(one) && statSync(join(at, one)).isFile())
        .map((one) => join(at, one))
    : [];

const carried = (pkg) => {
  const at = dirname(pkg.manifest_path);
  const found = new Set([
    ...filesIn(at, /^(licen[cs]e|copying|notice)/i).filter((one) => !/\.spdx$/i.test(one)),
    ...filesIn(join(at, "LICENSES"), /./),
  ]);
  if (pkg.license_file && existsSync(join(at, pkg.license_file))) {
    found.add(join(at, pkg.license_file));
  }
  return [...found]
    .map((one) => [one.slice(at.length), one])
    .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))
    .map(([, one]) => asWritten(readFileSync(one, "utf8")).trim());
};

const holderOf = (pkg) =>
  pkg.authors?.length
    ? pkg.authors.join(", ")
    : `the ${pkg.name} authors (${pkg.repository ?? `https://crates.io/crates/${pkg.name}`})`;

const draftedFor = (pkg) => {
  const parts = offered(pkg.license ?? "");
  if (parts.includes("APACHE-2.0")) return canonical("Apache-2.0");
  if (parts.includes("BSL-1.0")) return canonical("BSL-1.0");
  if (parts.includes("MIT")) return MIT(holderOf(pkg));
  if (parts.includes("BSD-3-CLAUSE")) return BSD3(holderOf(pkg));
  if (parts.includes("ISC")) return ISC(holderOf(pkg));
  return null;
};

const fenced = (text) => {
  const longest = Math.max(0, ...(text.match(/`+/g) ?? []).map((run) => run.length));
  const fence = "`".repeat(Math.max(3, longest + 1));
  return `${fence}text\n${text}\n${fence}`;
};

const inOrder = (a, b) =>
  a.name < b.name
    ? -1
    : a.name > b.name
      ? 1
      : a.version < b.version
        ? -1
        : a.version > b.version
          ? 1
          : 0;

const listed = (seen) =>
  [...seen.values()]
    .sort(inOrder)
    .map((one) => `| \`${one.name}\` | ${one.version} | ${one.licence} |`)
    .join("\n");

if (!existsSync(join(root, "app", "node_modules"))) {
  console.error("app/node_modules is not here, so not one notice could be read: run npm ci in app");
  process.exit(1);
}

const js = shipped();
const rs = crates();

const unread = [...js.values()].filter((one) => !one.notice);
if (unread.length > 0) {
  const said = unread.map((one) => `${one.name}@${one.version} (${one.licence})`).join(", ");
  console.error(`no notice could be read or drafted for ${said}`);
  process.exit(1);
}

const kept = [...js.values()]
  .sort(inOrder)
  .map((one) => `### \`${one.name}\` ${one.version} — ${one.licence}\n\n\`\`\`text\n${one.notice}\n\`\`\``)
  .join("\n\n");

const named = (all) => all.map((one) => `\`${one.name}\` ${one.version}`).join(", ");

const known = manifests();
const byText = new Map();
const bare = [];
for (const one of [...rs.values()].sort(inOrder)) {
  const pkg = known.get(`${one.name}@${one.version}`);
  const read = pkg ? carried(pkg) : [];
  const draft = pkg && read.length === 0 ? draftedFor(pkg) : null;
  const all = read.length > 0 ? read : draft ? [draft] : [];
  if (all.length === 0) bare.push(`${one.name}@${one.version} (${one.licence})`);
  for (const text of all) {
    const key = createHash("sha256").update(text).digest("hex");
    if (!byText.has(key)) byText.set(key, { text, carriedBy: [], writtenFor: [] });
    const entry = byText.get(key);
    const list = draft ? entry.writtenFor : entry.carriedBy;
    if (!list.includes(one)) list.push(one);
  }
}

if (bare.length > 0) {
  console.error(`no licence text could be read or written out for ${bare.join(", ")}`);
  process.exit(1);
}

const credited = (entry) =>
  [
    entry.carriedBy.length > 0 ? `Carried by ${named(entry.carriedBy)}.` : null,
    entry.writtenFor.length > 0
      ? `Written out for ${named(entry.writtenFor)}, from the licence the manifest declares: the crate ships no licence file.`
      : null,
  ]
    .filter(Boolean)
    .join(" ");

const written = [...byText.values()]
  .map((entry, at) => `## Text ${at + 1}\n\n${credited(entry)}\n\n${fenced(entry.text)}`)
  .join("\n\n");

writeFileSync(
  out,
  asWritten(`# Third-party notices — what ships inside CopyPaste

<!-- Written by \`npm run notices\`. Do not edit by hand. -->

CopyPaste is GPL-3.0-only. The binaries carry the work below, each under its own
licence. Nothing of it was copied into CopyPaste's own source; what was copied in
is in
[THIRD-PARTY.md](${REPO}/THIRD-PARTY.md).

The crates are the ones linked into the binaries for the three systems that are
published, Windows on x86-64 and macOS on Apple silicon and Intel: what only
builds, tests or generates code is left out, and a crate linked at two versions
is named once per version. Nothing here depends on the machine that wrote it: the
three systems are always the same, the order is by name and version compared by
code point, and the licence files of a package are read in full and in a fixed
order.

Every package in the window has its notice reproduced below, in full. The crates
are named with the licence each one declares, and the licence texts they carry
are in
[THIRD-PARTY-LICENSES.md](${REPO}/THIRD-PARTY-LICENSES.md), each written once
with the crates that carry it.

## In the window (${js.size} packages)

| Package | Version | Licence |
| --- | --- | --- |
${listed(js)}

## In the core (${rs.size} crates)

| Crate | Version | Licence |
| --- | --- | --- |
${listed(rs)}

## The notices themselves

${kept}
`),
);

writeFileSync(
  texts,
  asWritten(`# Licence texts — what the crates inside CopyPaste carry

<!-- Written by \`npm run notices\`. Do not edit by hand. -->

The licence texts of the ${rs.size} crates named in
[THIRD-PARTY-BUNDLED.md](${REPO}/THIRD-PARTY-BUNDLED.md), each written once with
the crates that carry it: ${byText.size} texts. A text is read in full from the
crate as it is published, from its licence, copying and notice files and its
LICENSES folder. A crate that publishes none gets the licence its manifest
declares, Apache-2.0 first where it is offered, with the holders its manifest
names.

${written}
`),
);

console.log(`${js.size} packages, ${rs.size} crates, ${byText.size} licence texts -> ${out}, ${texts}`);
