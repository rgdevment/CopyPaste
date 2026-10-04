import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const out = join(root, "THIRD-PARTY-BUNDLED.md");

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

const licenceFiles = (at) =>
  existsSync(at)
    ? readdirSync(at)
        .filter((one) => /^(licen[cs]e|copying)/i.test(one))
        .filter((one) => !/\.spdx$/i.test(one))
        .filter((one) => statSync(join(at, one)).isFile())
        .sort()
    : [];

const noticed = (at) => {
  if (!existsSync(at)) return null;
  const named = licenceFiles(at);
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

const carried = (one) => {
  const at = dirname(one.manifest_path);
  const found = new Set(licenceFiles(at).map((name) => join(at, name)));
  const reuse = join(at, "LICENSES");
  if (existsSync(reuse) && statSync(reuse).isDirectory()) {
    for (const name of readdirSync(reuse).sort()) {
      if (statSync(join(reuse, name)).isFile()) found.add(join(reuse, name));
    }
  }
  if (one.license_file && existsSync(join(at, one.license_file))) found.add(join(at, one.license_file));
  for (const name of readdirSync(at).sort()) {
    if (/^notice/i.test(name) && statSync(join(at, name)).isFile()) found.add(join(at, name));
  }
  return [...found].sort().map((path) => asWritten(readFileSync(path, "utf8")).trim());
};

const holderOf = (one) => {
  if (one.authors?.length) return one.authors.join(", ");
  return `the ${one.name} authors (${one.repository ?? `https://crates.io/crates/${one.name}`})`;
};

const draftedFor = (one) => {
  const parts = offered(one.license ?? "");
  if (parts.includes("APACHE-2.0")) return canonical("Apache-2.0");
  if (parts.includes("MPL-2.0")) return canonical("MPL-2.0");
  if (parts.includes("BSL-1.0")) return canonical("BSL-1.0");
  const who = holderOf(one);
  if (parts.includes("MIT")) return MIT(who);
  if (parts.includes("BSD-3-CLAUSE")) return BSD3(who);
  if (parts.includes("ISC")) return ISC(who);
  return null;
};

const crates = () => {
  execFileSync("cargo", ["fetch", "--locked"], { cwd: root, stdio: ["ignore", "ignore", "inherit"] });
  const said = execFileSync("cargo", ["metadata", "--format-version", "1", "--locked"], {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  const meta = JSON.parse(said);
  const byId = new Map(meta.packages.map((one) => [one.id, one]));
  const nodes = new Map((meta.resolve?.nodes ?? []).map((one) => [one.id, one]));
  const ours = new Set(meta.workspace_members);

  const walked = new Set(ours);
  const queue = [...ours];
  while (queue.length) {
    for (const dep of nodes.get(queue.shift())?.deps ?? []) {
      const kinds = dep.dep_kinds ?? [];
      if (kinds.length > 0 && kinds.every((one) => one.kind === "dev")) continue;
      if (walked.has(dep.pkg)) continue;
      walked.add(dep.pkg);
      queue.push(dep.pkg);
    }
  }

  const seen = new Map();
  for (const id of walked) {
    if (ours.has(id)) continue;
    const one = byId.get(id);
    if (!one) continue;
    const key = `${one.name}@${one.version}`;
    if (seen.has(key)) continue;
    const texts = carried(one);
    const draft = texts.length > 0 ? null : draftedFor(one);
    seen.set(key, {
      name: one.name,
      version: one.version,
      licence: one.license ?? "see the crate",
      texts: draft ? [draft] : texts,
      drafted: Boolean(draft),
    });
  }
  return seen;
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

const fenced = (text) => {
  const longest = Math.max(0, ...(text.match(/`+/g) ?? []).map((run) => run.length));
  const fence = "`".repeat(Math.max(3, longest + 1));
  return `${fence}text\n${text}\n${fence}`;
};

const named = (all) => all.map((one) => `\`${one.name}\` ${one.version}`).join(", ");

const grouped = (seen) => {
  const byText = new Map();
  for (const one of [...seen.values()].sort(inOrder)) {
    for (const text of one.texts) {
      const key = createHash("sha256").update(text).digest("hex");
      if (!byText.has(key)) byText.set(key, { text, carriedBy: [], draftedFor: [] });
      const entry = byText.get(key);
      const list = one.drafted ? entry.draftedFor : entry.carriedBy;
      if (!list.includes(one)) list.push(one);
    }
  }
  return [...byText.values()];
};

const credited = (entry) =>
  [
    entry.carriedBy.length > 0 ? `Carried by ${named(entry.carriedBy)}.` : null,
    entry.draftedFor.length > 0
      ? `Written out for ${named(entry.draftedFor)}, from the licence the manifest declares: the crate ships no licence file.`
      : null,
  ]
    .filter(Boolean)
    .join(" ");

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

const bare = [...rs.values()].filter((one) => one.texts.length === 0);
if (bare.length > 0) {
  const said = bare.map((one) => `${one.name}@${one.version} (${one.licence})`).join(", ");
  console.error(`no licence text could be read or written out for ${said}`);
  process.exit(1);
}

const kept = [...js.values()]
  .sort(inOrder)
  .map((one) => `### \`${one.name}\` ${one.version} — ${one.licence}\n\n${fenced(asWritten(one.notice))}`)
  .join("\n\n");

const texts = grouped(rs);

const crateTexts = texts
  .map((entry, at) => `### Text ${at + 1}\n\n${credited(entry)}\n\n${fenced(entry.text)}`)
  .join("\n\n");

writeFileSync(
  out,
  asWritten(`# Third-party notices — what ships inside CopyPaste

<!-- Written by \`npm run notices\`. Do not edit by hand. -->

CopyPaste is GPL-3.0-only. The binaries carry the work below, each under its own
licence. Nothing of it was copied into CopyPaste's own source; what was copied in
is in [THIRD-PARTY.md](THIRD-PARTY.md).

The crates are every one the build resolves on any system, which is what
Cargo.lock holds; what only the tests use is left out, and a crate resolved at
two versions is named once per version. Nothing here depends on the machine that
wrote it: no platform is named, the order is by name and version compared by code
point, and the licence files
of a package are read in full and in a fixed order.

Every package in the window has its notice reproduced below, in full. The crates'
licence texts follow them, each one written once with the crates that carry it.
Where a crate offers a choice, the text is the one it ships; a crate that ships
none gets the licence its manifest declares, Apache-2.0 first where it is offered.

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

## The crates' licence texts (${texts.length} texts)

${crateTexts}
`),
);

console.log(`${js.size} packages, ${rs.size} crates, ${texts.length} crate texts -> ${out}`);
