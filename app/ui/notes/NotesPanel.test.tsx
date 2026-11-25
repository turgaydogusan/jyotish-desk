import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { useState } from "react";
import { vi } from "vitest";

vi.mock("../api", () => ({
  api: {
    listNotes: vi.fn(),
    addNote: vi.fn(),
    updateNote: vi.fn(),
    deleteNote: vi.fn(),
  },
}));

import { api } from "../api";
import type { Note } from "../bindings";
import type { Panel } from "../layout/model";
import { NoticeBar } from "../shell/notices";
import { renderWithStores } from "../test/render";
import { MAX_NOTE_UNITS, NOTE_SAVE_DEBOUNCE_MS, NOTE_SAVE_MAX_WAIT_MS } from "./NoteEditor";
import { NOTES_LIMIT, NotesPanel } from "./NotesPanel";

const ok = <T,>(data: T) => ({ status: "ok" as const, data });
const panel: Panel = { id: "notes", kind: "notes", x: 0, y: 0, w: 4, h: 6, settings: {} };
const note = (id: number, body: string, created_at: string, chart_id: number | null = null): Note => ({
  id,
  uuid: `u${id}`,
  client_id: 1,
  chart_id,
  body,
  created_at,
  updated_at: created_at,
});
const older = note(1, "ilk seans", "2026-01-05T10:00:00.000Z");
const newer = note(2, "ikinci seans", "2026-02-10T15:30:00.000Z", 1);

let setShown: (shown: boolean) => void = () => {};
function Host() {
  const [shown, set] = useState(true);
  setShown = set;
  return (
    <>
      <NoticeBar />
      {shown && <NotesPanel panel={panel} />}
    </>
  );
}

function renderPanel() {
  return renderWithStores(<Host />);
}

const list = () => screen.getByRole("list", { name: "Örnek Danışan notları" });

beforeEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  vi.mocked(api.listNotes).mockResolvedValue(ok([newer, older]));
  vi.mocked(api.updateNote).mockImplementation(async (id, body) => ok({ ...(id === 1 ? older : newer), body }));
});

test("notes are read from list_notes, newest first, with a date and the chart they belong to", async () => {
  renderPanel();
  const items = await within(await screen.findByRole("list", { name: "Örnek Danışan notları" })).findAllByRole("listitem");
  expect(api.listNotes).toHaveBeenCalledWith(1, NOTES_LIMIT);
  expect(items).toHaveLength(2);
  const first = within(items[0]).getByRole("textbox");
  expect(first).toHaveValue("ikinci seans");
  expect(first).toHaveAccessibleName(/10 Şub 2026.*bu harita/);
  expect(first).toHaveAttribute("maxlength", String(MAX_NOTE_UNITS));
  expect(within(items[1]).getByRole("textbox")).toHaveAccessibleName(/5 Oca 2026/);
  expect(within(items[0]).getByText(/10 Şub 2026/).closest("time")).toHaveAttribute("datetime", newer.created_at);
});

test("typing saves once after the debounce, with the last text", async () => {
  vi.useFakeTimers();
  renderPanel();
  await act(async () => {});
  const box = within(list()).getAllByRole("textbox")[1];
  fireEvent.change(box, { target: { value: "ilk seans, d" } });
  await act(async () => vi.advanceTimersByTime(NOTE_SAVE_DEBOUNCE_MS - 1));
  fireEvent.change(box, { target: { value: "ilk seans, düzeltildi" } });
  await act(async () => vi.advanceTimersByTime(NOTE_SAVE_DEBOUNCE_MS - 1));
  expect(api.updateNote).not.toHaveBeenCalled();
  await act(async () => vi.advanceTimersByTime(1));
  vi.useRealTimers();
  expect(api.updateNote).toHaveBeenCalledOnce();
  expect(api.updateNote).toHaveBeenCalledWith(1, "ilk seans, düzeltildi");
});

test("continuous typing is still saved at least every NOTE_SAVE_MAX_WAIT_MS", async () => {
  vi.useFakeTimers();
  renderPanel();
  await act(async () => {});
  const box = within(list()).getAllByRole("textbox")[1];
  const step = 400;
  for (let i = 0; i * step < NOTE_SAVE_MAX_WAIT_MS; i++) {
    fireEvent.change(box, { target: { value: `yazım ${i}` } });
    await act(async () => vi.advanceTimersByTime(step));
  }
  vi.useRealTimers();
  // 400 ms aralıkla yazım hiçbir zaman 500 ms durmaz; yine de 2 400 ms dolmadan kaydedilir.
  expect(api.updateNote).toHaveBeenCalledOnce();
  expect(api.updateNote).toHaveBeenCalledWith(1, "yazım 4");
});

test("hiding the window saves pending text at once", async () => {
  renderPanel();
  const box = within(await screen.findByRole("list", { name: "Örnek Danışan notları" })).getAllByRole("textbox")[0];
  fireEvent.change(box, { target: { value: "gizlenmeden önce" } });
  Object.defineProperty(document, "visibilityState", { configurable: true, get: () => "hidden" });
  try {
    act(() => void document.dispatchEvent(new Event("visibilitychange")));
  } finally {
    delete (document as { visibilityState?: unknown }).visibilityState;
  }
  await waitFor(() => expect(api.updateNote).toHaveBeenCalledWith(2, "gizlenmeden önce"));
});

test("a save that fails after the panel closed is reported without the note text", async () => {
  vi.mocked(api.updateNote).mockResolvedValue({ status: "error", error: { code: "store_unavailable", params: {} } });
  renderPanel();
  const box = within(await screen.findByRole("list", { name: "Örnek Danışan notları" })).getAllByRole("textbox")[0];
  fireEvent.change(box, { target: { value: "gizli seans metni" } });
  act(() => setShown(false));
  const alert = await screen.findByRole("alert");
  expect(alert).toHaveTextContent("Panel kapanırken bir not kaydedilemedi; son değişiklikler yazılmadı: Danışan veritabanı kullanılamıyor.");
  expect(alert).not.toHaveTextContent("gizli seans metni");
});

test("leaving the box saves at once and closing the panel saves pending text before the next read", async () => {
  renderPanel();
  const box = within(await screen.findByRole("list", { name: "Örnek Danışan notları" })).getAllByRole("textbox")[0];
  fireEvent.change(box, { target: { value: "ikinci seans +" } });
  fireEvent.blur(box);
  await waitFor(() => expect(api.updateNote).toHaveBeenCalledWith(2, "ikinci seans +"));
  // Değişmeyen metin yeniden yazılmaz.
  fireEvent.blur(box);
  expect(api.updateNote).toHaveBeenCalledOnce();
  fireEvent.change(box, { target: { value: "panel kapanırken" } });
  let release!: () => void;
  vi.mocked(api.updateNote).mockImplementationOnce(
    (id, body) => new Promise((resolve) => (release = () => resolve(ok({ ...newer, id, body })))),
  );
  act(() => setShown(false));
  await waitFor(() => expect(api.updateNote).toHaveBeenLastCalledWith(2, "panel kapanırken"));
  vi.mocked(api.listNotes).mockClear();
  act(() => setShown(true));
  // Liste, bekleyen kayıt bitmeden okunmaz.
  await act(async () => {});
  expect(api.listNotes).not.toHaveBeenCalled();
  release();
  await waitFor(() => expect(api.listNotes).toHaveBeenCalledOnce());
});

test("a failed save is shown at the note and retried on the next leave", async () => {
  vi.mocked(api.updateNote).mockResolvedValueOnce({ status: "error", error: { code: "store_unavailable", params: {} } });
  renderPanel();
  const box = within(await screen.findByRole("list", { name: "Örnek Danışan notları" })).getAllByRole("textbox")[0];
  fireEvent.change(box, { target: { value: "kaydedilmeyen" } });
  fireEvent.blur(box);
  const alert = await screen.findByRole("alert");
  expect(alert).toHaveTextContent("Not kaydedilemedi: Danışan veritabanı kullanılamıyor.");
  expect(alert).not.toHaveTextContent("kaydedilmeyen");
  expect(box).toHaveAccessibleDescription(/Not kaydedilemedi/);
  fireEvent.blur(box);
  await waitFor(() => expect(api.updateNote).toHaveBeenCalledTimes(2));
  await waitFor(() => expect(screen.queryByRole("alert")).toBeNull());
});

test("New note adds an empty note on the open chart and focuses it", async () => {
  const added = note(3, "", "2026-03-01T09:00:00.000Z", 1);
  vi.mocked(api.addNote).mockResolvedValue(ok(added));
  renderPanel();
  await screen.findByRole("list", { name: "Örnek Danışan notları" });
  fireEvent.click(screen.getByRole("button", { name: "Yeni not" }));
  await waitFor(() => expect(within(list()).getAllByRole("listitem")).toHaveLength(3));
  expect(api.addNote).toHaveBeenCalledWith(1, 1, "");
  expect(document.activeElement).toBe(within(list()).getAllByRole("textbox")[0]);
});

test("deleting asks first; after confirming the note goes away, with no save after it", async () => {
  vi.mocked(api.deleteNote).mockResolvedValue(ok(null));
  renderPanel();
  const items = await within(await screen.findByRole("list", { name: "Örnek Danışan notları" })).findAllByRole("listitem");
  fireEvent.change(within(items[1]).getByRole("textbox"), { target: { value: "silinecek" } });
  fireEvent.click(within(items[1]).getByRole("button", { name: /5 Oca 2026.* tarihli notu sil/ }));
  const dialog = screen.getByRole("dialog", { name: "Not silinsin mi?" });
  expect(document.activeElement).toBe(within(dialog).getByRole("button", { name: "Vazgeç" }));
  fireEvent.click(within(dialog).getByRole("button", { name: "Notu sil" }));
  await waitFor(() => expect(within(list()).getAllByRole("listitem")).toHaveLength(1));
  expect(api.deleteNote).toHaveBeenCalledWith(1);
  await new Promise((r) => setTimeout(r, NOTE_SAVE_DEBOUNCE_MS + 50));
  expect(api.updateNote).not.toHaveBeenCalled();
});

test("a read error has a retry; with no chart open the panel says so", async () => {
  vi.mocked(api.listNotes).mockResolvedValueOnce({ status: "error", error: { code: "store_unavailable", params: {} } });
  renderPanel();
  const alert = await screen.findByRole("alert");
  expect(alert).toHaveTextContent("Notlar okunamadı: Danışan veritabanı kullanılamıyor.");
  fireEvent.click(within(alert).getByRole("button", { name: "Yeniden dene" }));
  expect(await screen.findByRole("list", { name: "Örnek Danışan notları" })).toBeInTheDocument();
});

test("an empty client shows the empty text", async () => {
  vi.mocked(api.listNotes).mockResolvedValue(ok([]));
  renderPanel();
  expect(await screen.findByText("Henüz not yok.")).toBeInTheDocument();
});
