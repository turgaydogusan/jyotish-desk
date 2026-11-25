// Derleme ve denetim betiklerinin ön denetimleri: hiçbiri derleme başlatmadan ve hiçbir sır
// gerektirmeden döner. Buradaki değerler sır değildir; yalnızca "boş değil" denetimini geçer.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { ROOT } from "./check-version.mjs";

const NAMES = [
  "APPLE_CERTIFICATE",
  "APPLE_CERTIFICATE_PASSWORD",
  "APPLE_SIGNING_IDENTITY",
  "APPLE_API_KEY",
  "APPLE_API_ISSUER",
  "APPLE_API_KEY_PATH",
  "TAURI_SIGNING_PRIVATE_KEY",
  "TAURI_SIGNING_PRIVATE_KEY_PASSWORD",
];

function run(script, args, extra = {}) {
  const env = { PATH: process.env.PATH, HOME: process.env.HOME, ...extra };
  return spawnSync("sh", [join(ROOT, "scripts/release", script), ...args], { env, encoding: "utf8" });
}

test("an unknown mode is refused before anything is built or inspected", () => {
  for (const script of ["build.sh", "verify-bundle.sh"]) {
    const r = run(script, ["notarized"]);
    assert.equal(r.status, 2, script);
    assert.match(r.stderr, /^usage: \S+ unsigned\|signed\n$/, script);
    assert.equal(r.stdout, "", script);
  }
});

test("a signed build stops at the first missing value, a missing key file or the empty public key", () => {
  const dir = mkdtempSync(join(tmpdir(), "jd-release-test-"));
  try {
    const keyFile = join(dir, "placeholder.txt");
    writeFileSync(keyFile, "not a key\n");
    const all = Object.fromEntries(NAMES.map((n) => [n, "placeholder"]));

    const missing = run("build.sh", ["signed"], { ...all, APPLE_API_ISSUER: "" });
    assert.equal(missing.status, 2);
    assert.match(missing.stderr, /^build\.sh: APPLE_API_ISSUER is not set/);

    const noFile = run("build.sh", ["signed"], { ...all, APPLE_API_KEY_PATH: join(dir, "absent.p8") });
    assert.equal(noFile.status, 2);
    assert.match(noFile.stderr, /^build\.sh: APPLE_API_KEY_PATH does not point to a file/);
    // Değerler ve yol yazdırılmaz.
    assert.doesNotMatch(noFile.stderr + noFile.stdout, /placeholder|absent\.p8/);

    // Depodaki açık anahtar boşken (sahibi doldurana dek) imzalı derleme başlamaz.
    const conf = JSON.parse(readFileSync(join(ROOT, "app/src-tauri/tauri.conf.json"), "utf8"));
    if (conf.plugins.updater.pubkey === "") {
      const noKey = run("build.sh", ["signed"], { ...all, APPLE_API_KEY_PATH: keyFile });
      assert.equal(noKey.status, 2);
      assert.match(noKey.stderr, /^build\.sh: plugins\.updater\.pubkey is empty/);
    }
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("each build clears the previous bundle directory only after its checks, right before building", () => {
  const sh = readFileSync(join(ROOT, "scripts/release/build.sh"), "utf8");
  assert.match(sh, /clean_bundle\(\) \{\n  rm -rf "\$BUNDLE"\n\}/);
  assert.match(sh, /BUNDLE="\$ROOT\/target\/\$TARGET\/release\/bundle"/);
  const builds = [...sh.matchAll(/^( +)clean_bundle\n(?: *#.*\n)*\1pnpm --dir "\$ROOT\/app" tauri build /gm)];
  assert.equal(builds.length, 2);
  assert.equal([...sh.matchAll(/tauri build /g)].length, 2);
  // İmzalı kipte silme, üç ön denetimden sonradır.
  assert.ok(sh.indexOf("pubkey is empty") < sh.lastIndexOf("clean_bundle"));
});
