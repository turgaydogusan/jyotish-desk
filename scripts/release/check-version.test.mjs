import assert from "node:assert/strict";
import { existsSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { readVersions, ROOT, versionProblems } from "./check-version.mjs";

const same = (v) => ({ "Cargo.toml": v, "Cargo.lock": v, "app/package.json": v, "app/src-tauri/tauri.conf.json": v });

test("the repository carries one version in all four places and has its release notes", () => {
  const versions = readVersions();
  const version = versions["Cargo.toml"];
  assert.deepEqual(versionProblems(versions, { hasNotes: existsSync(join(ROOT, `docs/release/notes/v${version}.md`)) }), []);
});

test("a differing file, a malformed version, a wrong tag and missing notes are each reported", () => {
  assert.deepEqual(versionProblems(same("0.9.0"), { tag: "v0.9.0" }), []);
  assert.match(versionProblems({ ...same("0.9.0"), "app/package.json": "0.8.0" })[0], /app\/package\.json: 0\.8\.0 differs/);
  assert.match(versionProblems({ ...same("0.9.0"), "Cargo.lock": null })[0], /Cargo\.lock: version not found/);
  assert.match(versionProblems(same("0.9.0-beta.1"))[0], /not MAJOR\.MINOR\.PATCH/);
  assert.match(versionProblems(same("0.9.0"), { tag: "v0.9.1" })[0], /tag v0\.9\.1 does not match/);
  assert.match(versionProblems(same("0.9.0"), { tag: "0.9.0" })[0], /expected v0\.9\.0/);
  assert.match(versionProblems(same("0.9.0"), { hasNotes: false })[0], /notes\/v0\.9\.0\.md is missing/);
});
