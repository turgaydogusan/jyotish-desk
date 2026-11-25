import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { ROOT } from "./check-version.mjs";
import { buildLatestJson, PLATFORMS } from "./latest-json.mjs";

const input = {
  version: "0.9.1",
  notes: "## 0.9.1\n\n- Düzeltme\n",
  signature: "dW50cnVzdGVkIGNvbW1lbnQ=\n",
  asset: "Jyotish-Desk_0.9.1_universal.app.tar.gz",
  pubDate: "2026-12-01T10:00:00+03:00",
};

test("both architectures point at the one universal bundle of that tag", () => {
  const json = buildLatestJson(input);
  assert.deepEqual(Object.keys(json), ["version", "notes", "pub_date", "platforms"]);
  assert.equal(json.version, "0.9.1");
  assert.equal(json.notes, "## 0.9.1\n\n- Düzeltme");
  assert.equal(json.pub_date, "2026-12-01T07:00:00.000Z");
  assert.deepEqual(Object.keys(json.platforms), PLATFORMS);
  for (const p of PLATFORMS) {
    assert.deepEqual(json.platforms[p], {
      signature: "dW50cnVzdGVkIGNvbW1lbnQ=",
      url: "https://github.com/turgaydogusan/jyotish-desk/releases/download/v0.9.1/Jyotish-Desk_0.9.1_universal.app.tar.gz",
    });
  }
});

test("a malformed version, asset name, signature or date is refused", () => {
  assert.throws(() => buildLatestJson({ ...input, version: "v0.9.1" }), /version/);
  assert.throws(() => buildLatestJson({ ...input, version: "0.9" }), /version/);
  assert.throws(() => buildLatestJson({ ...input, asset: "Jyotish Desk.app.tar.gz" }), /asset/);
  assert.throws(() => buildLatestJson({ ...input, asset: "Jyotish-Desk.dmg" }), /asset/);
  assert.throws(() => buildLatestJson({ ...input, signature: " \n" }), /signature/);
  assert.throws(() => buildLatestJson({ ...input, signature: "a\nb" }), /signature/);
  assert.throws(() => buildLatestJson({ ...input, pubDate: "yesterday" }), /pub_date/);
});

test("the command line package.sh uses writes latest.json from the notes and signature files", () => {
  const dir = mkdtempSync(join(tmpdir(), "jd-latest-json-test-"));
  const cli = (...args) => spawnSync(process.execPath, [join(ROOT, "scripts/release/latest-json.mjs"), ...args], { encoding: "utf8" });
  try {
    const notes = join(dir, "notes.md");
    const sig = join(dir, "bundle.sig");
    const out = join(dir, "latest.json");
    writeFileSync(notes, input.notes);
    // İmza değil, yalnızca base64 biçimli bir yer tutucu.
    writeFileSync(sig, input.signature);

    const ok = cli(input.version, notes, sig, input.asset, out);
    assert.equal(ok.status, 0, ok.stderr);
    const json = JSON.parse(readFileSync(out, "utf8"));
    const expected = buildLatestJson({ ...input, pubDate: json.pub_date });
    assert.deepEqual(json, expected);
    assert.ok(!Number.isNaN(Date.parse(json.pub_date)));

    // Eksik bağımsız değişken: kullanım iletisi, dosya yazılmaz.
    const short = cli(input.version, notes, sig, input.asset);
    assert.equal(short.status, 2);
    assert.match(short.stderr, /^usage: latest-json\.mjs /);

    // Geçersiz sürüm: dosya yazılmaz.
    const bad = join(dir, "bad.json");
    assert.notEqual(cli("0.9", notes, sig, input.asset, bad).status, 0);
    assert.ok(!existsSync(bad));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
