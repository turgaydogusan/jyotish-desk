// Sürüm tek değerdir ama üç dosyada yazar: çalışma alanı (`Cargo.toml`), arayüz paketi
// (`app/package.json`) ve Tauri yapılandırması (`app/src-tauri/tauri.conf.json`). Üçü de aynı
// olmalı, `Cargo.lock` çalışma alanı paketlerini o sürümle taşımalı ve sürümün yayın notu
// bulunmalıdır. Etiket verilirse (`v0.9.0`) o da aynı sürümü göstermelidir.
//
// Kullanım: node scripts/release/check-version.mjs [etiket]
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

export const ROOT = join(dirname(fileURLToPath(import.meta.url)), "../..");
const SEMVER = /^\d+\.\d+\.\d+$/;

export function readVersions(root = ROOT) {
  const read = (p) => readFileSync(join(root, p), "utf8");
  const workspace = /\[workspace\.package\][^[]*?\nversion = "([^"]+)"/.exec(read("Cargo.toml"))?.[1] ?? null;
  const lock = /\[\[package\]\]\nname = "jyotish-desk"\nversion = "([^"]+)"/.exec(read("Cargo.lock"))?.[1] ?? null;
  return {
    "Cargo.toml": workspace,
    "Cargo.lock": lock,
    "app/package.json": JSON.parse(read("app/package.json")).version ?? null,
    "app/src-tauri/tauri.conf.json": JSON.parse(read("app/src-tauri/tauri.conf.json")).version ?? null,
  };
}

/** Sorunların listesi; boşsa sürüm tutarlıdır. */
export function versionProblems(versions, { tag = null, hasNotes = true } = {}) {
  const problems = [];
  const entries = Object.entries(versions);
  const version = entries[0][1];
  for (const [file, v] of entries) {
    if (v === null) problems.push(`${file}: version not found`);
    else if (!SEMVER.test(v)) problems.push(`${file}: ${v} is not MAJOR.MINOR.PATCH`);
    else if (v !== version) problems.push(`${file}: ${v} differs from ${entries[0][0]} (${version})`);
  }
  if (tag !== null && tag !== `v${version}`) problems.push(`tag ${tag} does not match version ${version} (expected v${version})`);
  if (!hasNotes) problems.push(`docs/release/notes/v${version}.md is missing`);
  return problems;
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  const tag = process.argv[2] ?? null;
  const versions = readVersions();
  const version = versions["Cargo.toml"];
  const problems = versionProblems(versions, { tag, hasNotes: existsSync(join(ROOT, `docs/release/notes/v${version}.md`)) });
  if (problems.length > 0) {
    for (const p of problems) console.error(`version check: ${p}`);
    process.exit(1);
  }
  console.log(version);
}
