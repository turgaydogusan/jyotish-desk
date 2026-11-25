import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { StrictMode, useState } from "react";
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
import { useSession } from "../session/store";
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
  vi.mocked(api.deleteNote).mockResolvedValue(ok(null));
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

test("under StrictMode a failed save while the panel is open is still shown at the note", async () => {
  vi.mocked(api.updateNote).mockResolvedValueOnce({ status: "error", error: { code: "store_unavailable", params: {} } });
  renderWithStores(
    <StrictMode>
      <Host />
    </StrictMode>,
  );
  const box = within(await screen.findByRole("list", { name: "Örnek Danışan notları" })).getAllByRole("textbox")[0];
  fireEvent.change(box, { target: { value: "kaydedilmeyen" } });
  fireEvent.blur(box);
  const alert = await screen.findByRole("alert");
  expect(alert).toHaveTextContent("Not kaydedilemedi: Danışan veritabanı kullanılamıyor.");
  expect(alert).not.toHaveTextContent("Panel kapanırken");
  expect(box).toHaveAccessibleDescription(/Not kaydedilemedi/);
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

test("a second New note click while the first is being added adds nothing", async () => {
  let release!: () => void;
  vi.mocked(api.addNote).mockImplementation(
    () => new Promise((resolve) => (release = () => resolve(ok(note(3, "", "2026-03-01T09:00:00.000Z", 1))))),
  );
  renderPanel();
  await screen.findByRole("list", { name: "Örnek Danışan notları" });
  const add = screen.getByRole("button", { name: "Yeni not" });
  fireEvent.click(add);
  fireEvent.click(add);
  release();
  await waitFor(() => expect(within(list()).getAllByRole("listitem")).toHaveLength(3));
  expect(api.addNote).toHaveBeenCalledOnce();
  // Ekleme bitince düğme yeniden çalışır.
  fireEvent.click(add);
  expect(api.addNote).toHaveBeenCalledTimes(2);
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

test("deleting waits for the note's queued saves, so none of them runs after the delete", async () => {
  vi.mocked(api.deleteNote).mockResolvedValue(ok(null));
  let release!: () => void;
  vi.mocked(api.updateNote).mockImplementationOnce(
    (id, body) => new Promise((resolve) => (release = () => resolve(ok({ ...older, id, body })))),
  );
  renderPanel();
  const items = await within(await screen.findByRole("list", { name: "Örnek Danışan notları" })).findAllByRole("listitem");
  const box = within(items[1]).getByRole("textbox");
  fireEvent.change(box, { target: { value: "yavaş kayıt" } });
  fireEvent.blur(box);
  fireEvent.change(box, { target: { value: "sıradaki kayıt" } });
  fireEvent.blur(box);
  fireEvent.click(within(items[1]).getByRole("button", { name: /5 Oca 2026.* tarihli notu sil/ }));
  fireEvent.click(within(screen.getByRole("dialog", { name: "Not silinsin mi?" })).getByRole("button", { name: "Notu sil" }));
  await act(async () => {});
  // İlk kayıt sürerken silme gönderilmez.
  expect(api.updateNote).toHaveBeenCalledOnce();
  expect(api.deleteNote).not.toHaveBeenCalled();
  release();
  await waitFor(() => expect(within(list()).getAllByRole("listitem")).toHaveLength(1));
  expect(api.updateNote).toHaveBeenCalledTimes(2);
  expect(api.updateNote).toHaveBeenLastCalledWith(1, "sıradaki kayıt");
  expect(vi.mocked(api.updateNote).mock.invocationCallOrder[1]).toBeLessThan(vi.mocked(api.deleteNote).mock.invocationCallOrder[0]);
  expect(screen.queryByRole("alert")).toBeNull();
});

test("a delete that fails is reported, keeps the note and saves its pending text at once", async () => {
  vi.mocked(api.deleteNote).mockResolvedValue({ status: "error", error: { code: "store_unavailable", params: {} } });
  renderPanel();
  const items = await within(await screen.findByRole("list", { name: "Örnek Danışan notları" })).findAllByRole("listitem");
  fireEvent.change(within(items[1]).getByRole("textbox"), { target: { value: "silinemeyen" } });
  fireEvent.click(within(items[1]).getByRole("button", { name: /5 Oca 2026.* tarihli notu sil/ }));
  fireEvent.click(within(screen.getByRole("dialog", { name: "Not silinsin mi?" })).getByRole("button", { name: "Notu sil" }));
  const alert = await screen.findByRole("alert");
  expect(alert).toHaveTextContent("Silinemedi: Danışan veritabanı kullanılamıyor.");
  expect(alert).not.toHaveTextContent("silinemeyen");
  expect(within(list()).getAllByRole("listitem")).toHaveLength(2);
  // Silme başlarken durdurulan otomatik kayıt, beklemeden gönderilir.
  await waitFor(() => expect(api.updateNote).toHaveBeenCalledWith(1, "silinemeyen"), { timeout: NOTE_SAVE_DEBOUNCE_MS - 100 });
});

test("with no chart open the panel says so and reads nothing", () => {
  renderWithStores(<Host />, { opened: null });
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: "Yeni not" })).toBeNull();
  expect(api.listNotes).not.toHaveBeenCalled();
});

test("a read error has a retry", async () => {
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

test("after a note is deleted focus goes to New note", async () => {
  vi.mocked(api.deleteNote).mockResolvedValue(ok(null));
  renderPanel();
  const items = await within(await screen.findByRole("list", { name: "Örnek Danışan notları" })).findAllByRole("listitem");
  fireEvent.click(within(items[1]).getByRole("button", { name: /tarihli notu sil/ }));
  fireEvent.click(within(screen.getByRole("dialog", { name: "Not silinsin mi?" })).getByRole("button", { name: "Notu sil" }));
  await waitFor(() => expect(document.activeElement).toBe(screen.getByRole("button", { name: "Yeni not" })));
});

test("a note at the length limit and a client at the list limit say so", async () => {
  const many = Array.from({ length: NOTES_LIMIT }, (_, i) => note(i + 10, "x", `2026-01-${String((i % 28) + 1).padStart(2, "0")}T10:00:00.000Z`));
  vi.mocked(api.listNotes).mockResolvedValue(ok(many));
  renderPanel();
  expect(await screen.findByText(`En yeni ${NOTES_LIMIT} not gösteriliyor.`)).toBeInTheDocument();
  const box = within(list()).getAllByRole("textbox")[0];
  fireEvent.change(box, { target: { value: "a".repeat(MAX_NOTE_UNITS) } });
  expect(box).toHaveAccessibleDescription("Not en çok 16 384 karakter olabilir; fazlası eklenmedi.");
});

test("an empty new note is removed when focus leaves it, without asking", async () => {
  vi.mocked(api.addNote).mockResolvedValue(ok(note(3, "", "2026-03-01T09:00:00.000Z", 1)));
  renderPanel();
  await screen.findByRole("list", { name: "Örnek Danışan notları" });
  fireEvent.click(screen.getByRole("button", { name: "Yeni not" }));
  await waitFor(() => expect(within(list()).getAllByRole("listitem")).toHaveLength(3));
  fireEvent.blur(within(list()).getAllByRole("textbox")[0]);
  await waitFor(() => expect(within(list()).getAllByRole("listitem")).toHaveLength(2));
  expect(api.deleteNote).toHaveBeenCalledWith(3);
  expect(screen.queryByRole("dialog")).toBeNull();
});

test("an empty new note is removed when the panel closes; a written one and an old empty one stay", async () => {
  vi.mocked(api.listNotes).mockResolvedValue(ok([note(9, "", "2026-01-01T10:00:00.000Z"), older]));
  vi.mocked(api.addNote).mockResolvedValueOnce(ok(note(3, "", "2026-03-01T09:00:00.000Z", 1)));
  renderPanel();
  await screen.findByRole("list", { name: "Örnek Danışan notları" });
  fireEvent.click(screen.getByRole("button", { name: "Yeni not" }));
  await waitFor(() => expect(within(list()).getAllByRole("listitem")).toHaveLength(3));
  act(() => setShown(false));
  await waitFor(() => expect(api.deleteNote).toHaveBeenCalledOnce());
  expect(api.deleteNote).toHaveBeenCalledWith(3);
});

test("under StrictMode a new note is not removed by the mount, cleanup, mount cycle", async () => {
  vi.mocked(api.addNote).mockResolvedValue(ok(note(3, "", "2026-03-01T09:00:00.000Z", 1)));
  renderWithStores(
    <StrictMode>
      <NotesPanel panel={panel} />
    </StrictMode>,
  );
  await screen.findByRole("list", { name: "Örnek Danışan notları" });
  fireEvent.click(screen.getByRole("button", { name: "Yeni not" }));
  await waitFor(() => expect(within(list()).getAllByRole("listitem")).toHaveLength(3));
  await act(async () => {});
  expect(api.deleteNote).not.toHaveBeenCalled();
});

test("notes are read again after the session revision changes", async () => {
  let touch: () => void = () => {};
  function Toucher() {
    touch = useSession().touch;
    return null;
  }
  renderWithStores(
    <>
      <Toucher />
      <NotesPanel panel={panel} />
    </>,
  );
  await screen.findByRole("list", { name: "Örnek Danışan notları" });
  expect(api.listNotes).toHaveBeenCalledOnce();
  vi.mocked(api.listNotes).mockResolvedValue(ok([note(4, "içe aktarılan", "2026-04-01T10:00:00.000Z"), newer, older]));
  act(() => touch());
  await waitFor(() => expect(within(list()).getAllByRole("listitem")).toHaveLength(3));
  expect(api.listNotes).toHaveBeenCalledTimes(2);
});

test("a second confirmed delete while the first is running is ignored", async () => {
  let release!: () => void;
  vi.mocked(api.deleteNote).mockReturnValue(new Promise((r) => (release = () => r(ok(null)))));
  renderPanel();
  const items = await within(await screen.findByRole("list", { name: "Örnek Danışan notları" })).findAllByRole("listitem");
  const del = within(items[1]).getByRole("button", { name: /tarihli notu sil/ });
  fireEvent.click(del);
  fireEvent.click(within(screen.getByRole("dialog", { name: "Not silinsin mi?" })).getByRole("button", { name: "Notu sil" }));
  fireEvent.click(del);
  fireEvent.click(within(screen.getByRole("dialog", { name: "Not silinsin mi?" })).getByRole("button", { name: "Notu sil" }));
  release();
  await waitFor(() => expect(within(list()).getAllByRole("listitem")).toHaveLength(1));
  expect(api.deleteNote).toHaveBeenCalledOnce();
});
