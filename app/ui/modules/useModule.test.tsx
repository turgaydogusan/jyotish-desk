import { act, render, screen } from "@testing-library/react";
import { StrictMode } from "react";
import { expect, test, vi } from "vitest";
import type { Result } from "../api";
import type { CommandError } from "../bindings";
import { useLastData, useModule } from "./useModule";

const failure: CommandError = { code: "calc", params: { detail: "x" } };

/** Elle çözülen söz: yanıtın geliş sırasını test belirler. */
function deferred<T>() {
  let resolve!: (r: Result<T>) => void;
  const promise = new Promise<Result<T>>((r) => (resolve = r));
  return { promise, resolve };
}

function Probe({ k, fetch }: { k: string | null; fetch: () => Promise<Result<string>> }) {
  const m = useModule(k, fetch);
  return (
    <div>
      <span data-testid="state">{m.loading ? "loading" : m.error ? `error:${m.error.code}` : (m.data ?? "idle")}</span>
      <button type="button" onClick={m.retry}>
        retry
      </button>
    </div>
  );
}

const state = () => screen.getByTestId("state").textContent;

test("fetches once per key and shows the result", async () => {
  const d = deferred<string>();
  const fetch = vi.fn(() => d.promise);
  const { rerender } = render(<Probe k="a" fetch={fetch} />);
  expect(state()).toBe("loading");
  await act(async () => d.resolve({ status: "ok", data: "A" }));
  expect(state()).toBe("A");
  // Aynı anahtarla yeniden çizim yeni istek göndermez.
  rerender(<Probe k="a" fetch={fetch} />);
  expect(fetch).toHaveBeenCalledTimes(1);
});

test("without a key nothing is requested", () => {
  const fetch = vi.fn(async (): Promise<Result<string>> => ({ status: "ok", data: "A" }));
  render(<Probe k={null} fetch={fetch} />);
  expect(state()).toBe("idle");
  expect(fetch).not.toHaveBeenCalled();
});

test("an error is shown and retry asks again", async () => {
  const fetch = vi
    .fn<() => Promise<Result<string>>>()
    .mockResolvedValueOnce({ status: "error", error: failure })
    .mockResolvedValueOnce({ status: "ok", data: "B" });
  render(<Probe k="a" fetch={fetch} />);
  await act(async () => {});
  expect(state()).toBe("error:calc");
  await act(async () => screen.getByRole("button", { name: "retry" }).click());
  expect(fetch).toHaveBeenCalledTimes(2);
  expect(state()).toBe("B");
});

test("when the key changes the old result is hidden and a late answer for the old key is dropped", async () => {
  const first = deferred<string>();
  const second = deferred<string>();
  const fetch = vi.fn<() => Promise<Result<string>>>().mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise);
  const { rerender } = render(<Probe k="a" fetch={fetch} />);
  rerender(<Probe k="b" fetch={fetch} />);
  expect(state()).toBe("loading");
  // Eski anahtarın yanıtı sonradan gelir: gösterilmez.
  await act(async () => first.resolve({ status: "ok", data: "A" }));
  expect(state()).toBe("loading");
  await act(async () => second.resolve({ status: "ok", data: "B" }));
  expect(state()).toBe("B");
  // Önceki anahtara dönülünce eski sonuç yeniden istenir (önbellek yok).
  const third = deferred<string>();
  fetch.mockReturnValueOnce(third.promise);
  rerender(<Probe k="a" fetch={fetch} />);
  expect(state()).toBe("loading");
  await act(async () => third.resolve({ status: "ok", data: "A2" }));
  expect(state()).toBe("A2");
});

test("StrictMode's second effect run does not call the command again", async () => {
  const d = deferred<string>();
  const fetch = vi.fn(() => d.promise);
  render(
    <StrictMode>
      <Probe k="a" fetch={fetch} />
    </StrictMode>,
  );
  expect(fetch).toHaveBeenCalledTimes(1);
  await act(async () => d.resolve({ status: "ok", data: "A" }));
  expect(state()).toBe("A");
  // Yeniden deneme yeni bir istektir.
  const again = deferred<string>();
  fetch.mockReturnValueOnce(again.promise);
  await act(async () => screen.getByRole("button", { name: "retry" }).click());
  expect(fetch).toHaveBeenCalledTimes(2);
  await act(async () => again.resolve({ status: "ok", data: "A2" }));
  expect(state()).toBe("A2");
});

test("a fetch that rejects ends as an error instead of loading forever, and retry asks again", async () => {
  const fetch = vi
    .fn<() => Promise<Result<string>>>()
    .mockRejectedValueOnce(new Error("bridge down"))
    .mockResolvedValueOnce({ status: "ok", data: "B" });
  render(<Probe k="a" fetch={fetch} />);
  await act(async () => {});
  expect(state()).toBe("error:ipc");
  await act(async () => screen.getByRole("button", { name: "retry" }).click());
  expect(state()).toBe("B");
});

function KeptProbe({ k, scope, fetch }: { k: string | null; scope: string | null; fetch: () => Promise<Result<string>> }) {
  const m = useModule(k, fetch);
  const shown = useLastData(m, scope);
  return <span data-testid="kept">{`${shown ?? "none"}${m.loading ? " (loading)" : ""}${m.error ? " (error)" : ""}`}</span>;
}

test("the last data stays on screen while a new key loads, but not across scopes or after an error", async () => {
  const kept = () => screen.getByTestId("kept").textContent;
  const [a, b, c, d] = [deferred<string>(), deferred<string>(), deferred<string>(), deferred<string>()];
  const fetch = vi
    .fn<() => Promise<Result<string>>>()
    .mockReturnValueOnce(a.promise)
    .mockReturnValueOnce(b.promise)
    .mockReturnValueOnce(c.promise)
    .mockReturnValueOnce(d.promise);
  const { rerender } = render(<KeptProbe k="a" scope="panel" fetch={fetch} />);
  expect(kept()).toBe("none (loading)");
  await act(async () => a.resolve({ status: "ok", data: "A" }));
  expect(kept()).toBe("A");
  // Yeni anahtar: yanıt gelene dek önceki veri gösterilir.
  rerender(<KeptProbe k="b" scope="panel" fetch={fetch} />);
  expect(kept()).toBe("A (loading)");
  await act(async () => b.resolve({ status: "ok", data: "B" }));
  expect(kept()).toBe("B");
  // Hata eski veriyi göstermez.
  rerender(<KeptProbe k="c" scope="panel" fetch={fetch} />);
  expect(kept()).toBe("B (loading)");
  await act(async () => c.resolve({ status: "error", error: failure }));
  expect(kept()).toBe("none (error)");
  // Başka bir kapsam (başka harita) öncekinin verisini göstermez.
  rerender(<KeptProbe k="d" scope="other" fetch={fetch} />);
  expect(kept()).toBe("none (loading)");
  await act(async () => d.resolve({ status: "ok", data: "D" }));
  expect(kept()).toBe("D");
  // Anahtar ve kapsam yokken hiçbir şey gösterilmez.
  rerender(<KeptProbe k={null} scope={null} fetch={fetch} />);
  expect(kept()).toBe("none");
});

test("after an error, reading again in the same scope shows nothing from before the error", async () => {
  const kept = () => screen.getByTestId("kept").textContent;
  const [a, b, c, d] = [deferred<string>(), deferred<string>(), deferred<string>(), deferred<string>()];
  const fetch = vi
    .fn<() => Promise<Result<string>>>()
    .mockReturnValueOnce(a.promise)
    .mockReturnValueOnce(b.promise)
    .mockReturnValueOnce(c.promise)
    .mockReturnValueOnce(d.promise);
  const { rerender } = render(<KeptProbe k="a" scope="panel" fetch={fetch} />);
  await act(async () => a.resolve({ status: "ok", data: "A" }));
  rerender(<KeptProbe k="b" scope="panel" fetch={fetch} />);
  await act(async () => b.resolve({ status: "error", error: failure }));
  expect(kept()).toBe("none (error)");
  // The next read (a retry or another input) starts empty: "A" belongs to an input the user
  // has left, and the error came after it.
  rerender(<KeptProbe k="c" scope="panel" fetch={fetch} />);
  expect(kept()).toBe("none (loading)");
  await act(async () => c.resolve({ status: "ok", data: "C" }));
  expect(kept()).toBe("C");
  // From then on the last data is kept again.
  rerender(<KeptProbe k="d" scope="panel" fetch={fetch} />);
  expect(kept()).toBe("C (loading)");
});
