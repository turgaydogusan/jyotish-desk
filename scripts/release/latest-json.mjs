// Güncelleme bildirimi (`latest.json`): Tauri güncelleyicisinin GitHub Releases'ten okuduğu
// dosya. Evrensel (arm64 + x86_64) paket tek dosyadır; iki mimari anahtarı da onu gösterir.
// İmza, `tauri build`in ürettiği `.sig` dosyasının içeriğidir; burada üretilmez ve doğrulanmaz
// (doğrulamayı uygulama, gömülü açık anahtarla yapar).
//
// Kullanım: node scripts/release/latest-json.mjs <sürüm> <notlar.md> <paket.sig> <paket adı> <çıktı>
import { readFileSync, writeFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

export const RELEASE_BASE = "https://github.com/turgaydogusan/jyotish-desk/releases/download";
export const PLATFORMS = ["darwin-aarch64", "darwin-x86_64"];
const SEMVER = /^\d+\.\d+\.\d+$/;
const ASSET = /^[A-Za-z0-9._-]+\.app\.tar\.gz$/;

export function buildLatestJson({ version, notes, signature, asset, pubDate }) {
  if (!SEMVER.test(version)) throw new Error(`version is not MAJOR.MINOR.PATCH: ${version}`);
  if (!ASSET.test(asset)) throw new Error(`asset name must be a space-free .app.tar.gz: ${asset}`);
  const sig = signature.trim();
  if (sig === "" || /\s/.test(sig)) throw new Error("signature must be the single-line content of the .sig file");
  if (Number.isNaN(Date.parse(pubDate))) throw new Error(`pub_date is not a date: ${pubDate}`);
  const entry = { signature: sig, url: `${RELEASE_BASE}/v${version}/${asset}` };
  return {
    version,
    notes: notes.trim(),
    pub_date: new Date(pubDate).toISOString(),
    platforms: Object.fromEntries(PLATFORMS.map((p) => [p, entry])),
  };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  const [version, notesPath, sigPath, asset, out] = process.argv.slice(2);
  if (!out) {
    console.error("usage: latest-json.mjs <version> <notes.md> <bundle.sig> <asset name> <out>");
    process.exit(2);
  }
  const json = buildLatestJson({
    version,
    notes: readFileSync(notesPath, "utf8"),
    signature: readFileSync(sigPath, "utf8"),
    asset,
    pubDate: new Date().toISOString(),
  });
  writeFileSync(out, `${JSON.stringify(json, null, 2)}\n`);
  console.log(`latest.json: ${version}, ${PLATFORMS.join(" + ")}`);
}
