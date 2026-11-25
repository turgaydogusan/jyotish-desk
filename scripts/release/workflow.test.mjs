// Yayın işinin (release.yml) güvenlik kuralları metin düzeyinde denetlenir: sırlara dokunan iş
// yalnızca etiketle ya da elle başlar, eylemler commit SHA'sıyla sabittir, önbellek yoktur,
// hiçbir adım bir sırrı yazdırmaz ve yayın taslak olarak oluşturulur.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { ROOT } from "./check-version.mjs";

const yml = readFileSync(join(ROOT, ".github/workflows/release.yml"), "utf8");
const code = yml
  .split("\n")
  .filter((l) => !l.trim().startsWith("#"))
  .join("\n");

const SECRETS = [
  "APPLE_CERTIFICATE",
  "APPLE_CERTIFICATE_PASSWORD",
  "APPLE_SIGNING_IDENTITY",
  "APPLE_API_KEY_ID",
  "APPLE_API_ISSUER",
  "APPLE_API_KEY_P8",
  "TAURI_SIGNING_PRIVATE_KEY",
  "TAURI_SIGNING_PRIVATE_KEY_PASSWORD",
];

test("the release job starts only from a version tag or by hand", () => {
  assert.match(code, /^on:\n  push:\n    tags: \["v\*"\]\n  workflow_dispatch:\n/m);
  assert.doesNotMatch(code, /pull_request|schedule:|branches:/);
});

test("every action is pinned to a commit and nothing is cached", () => {
  const uses = [...code.matchAll(/uses: (\S+)/g)].map((m) => m[1]);
  assert.ok(uses.length >= 5);
  for (const u of uses) assert.match(u, /^[\w-]+\/[\w-]+@[0-9a-f]{40}$/, u);
  assert.doesNotMatch(code, /rust-cache|actions\/cache|cache:/);
});

test("the eight secrets are the documented ones and are never echoed", () => {
  const used = [...new Set([...code.matchAll(/secrets\.(\w+)/g)].map((m) => m[1]))].sort();
  assert.deepEqual(used, [...SECRETS].sort());
  // Bir sır yalnızca `env:` eşlemesinde geçer; `run:` satırlarında `${{ secrets… }}` yoktur.
  for (const line of code.split("\n")) {
    if (line.includes("secrets.")) assert.match(line, /^\s+\w+: \$\{\{ secrets\.\w+ \}\}$/, line);
  }
  assert.doesNotMatch(code, /set -x|echo "?\$(S[1-8]|APPLE_|TAURI_SIGNING)/);
});

test("the default token is read-only; only the release job may write, and the release is a draft", () => {
  assert.match(code, /^permissions:\n  contents: read\n/m);
  assert.equal([...code.matchAll(/contents: write/g)].length, 1);
  assert.match(code, /gh release create "\$GITHUB_REF_NAME" --draft --verify-tag/);
  assert.match(code, /if: steps\.mode\.outputs\.signed == 'true' && github\.ref_type == 'tag'/);
  assert.match(code, /persist-credentials: false/);
});

test("without the secrets the build is an unsigned dry run that is still verified", () => {
  assert.match(code, /sh scripts\/release\/build\.sh unsigned\n\s+sh scripts\/release\/verify-bundle\.sh unsigned/);
  assert.match(code, /sh scripts\/release\/build\.sh signed/);
  assert.match(code, /sh scripts\/release\/verify-bundle\.sh signed/);
  assert.match(code, /if: always\(\)\n\s+run: rm -f "\$RUNNER_TEMP\/notary-key\.p8"/);
});
