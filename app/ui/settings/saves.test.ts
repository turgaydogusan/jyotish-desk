import { expect, test, vi } from "vitest";
import type { CommandError } from "../bindings";
import { createSaver } from "./saves";

const tick = () => new Promise((r) => setTimeout(r, 0));

function harness(results: (CommandError | null)[] = []) {
  const gates: (() => void)[] = [];
  const written: number[] = [];
  const onError = vi.fn();
  const onSaved = vi.fn();
  const saver = createSaver<number>(
    (value) =>
      new Promise((resolve) => {
        written.push(value);
        gates.push(() => resolve(results.shift() ?? null));
      }),
    { debounceMs: 10_000, onError, onSaved },
  );
  return { saver, gates, written, onError, onSaved };
}

test("flush writes the pending value at once and an empty flush writes nothing", async () => {
  const h = harness();
  h.saver.flush();
  expect(h.written).toEqual([]);
  h.saver.schedule(1);
  h.saver.schedule(2);
  h.saver.flush();
  expect(h.written).toEqual([2]);
  h.gates[0]();
  await tick();
  expect(h.onSaved).toHaveBeenCalledExactlyOnceWith(2);
});

test("values that arrive during a write wait for it and only the last one is written", async () => {
  const h = harness();
  h.saver.schedule(1);
  h.saver.flush();
  h.saver.schedule(2);
  h.saver.flush();
  h.saver.schedule(3);
  h.saver.flush();
  expect(h.written).toEqual([1]);
  h.gates[0]();
  await tick();
  expect(h.written).toEqual([1, 3]);
  h.gates[1]();
  await tick();
  expect(h.onSaved.mock.calls).toEqual([[1], [3]]);
  expect(h.onError).not.toHaveBeenCalled();
});

test("a failed write is reported and the value queued behind it is still written", async () => {
  const error: CommandError = { code: "store_sqlite", params: { detail: "disk" } };
  const h = harness([error]);
  h.saver.schedule(1);
  h.saver.flush();
  h.saver.schedule(2);
  h.saver.flush();
  h.gates[0]();
  await tick();
  expect(h.onError).toHaveBeenCalledExactlyOnceWith(error);
  expect(h.written).toEqual([1, 2]);
  h.gates[1]();
  await tick();
  expect(h.onSaved).toHaveBeenCalledExactlyOnceWith(2);
});
