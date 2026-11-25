import assert from "node:assert/strict";
import { test } from "node:test";
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
