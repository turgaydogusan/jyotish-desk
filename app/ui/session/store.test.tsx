import { act, fireEvent, screen, waitFor } from "@testing-library/react";
import { vi } from "vitest";

vi.mock("../api", () => ({ api: { openClient: vi.fn() } }));

import { api } from "../api";
import type { OpenedClient } from "../bindings";
import { NoticeBar, useNotices } from "../shell/notices";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useViewStore } from "../view/store";
import { useSession } from "./store";

/** İkinci danışan: aynı harita verisi, başka kimlikler. */
function other(): OpenedClient {
  const o = fixture().opened;
  o.client = { ...o.client, id: 2, name: "İkinci Danışan" };
  o.chart_record = { ...o.chart_record, id: 9, client_id: 2, tags: ["kariyer"] };
  o.view_state = {
    ...o.view_state,
    dasha_expanded_path: ["Venus"],
    chara_expanded_path: { cycle: 2, signs: ["Gemini"] },
    compat_partner_uuid: "4b0f2a8e-0000-4000-8000-00000000000a",
    compat_roles: "OpenIsBride",
  };
  o.warnings = ["LastChartMissing"];
  return o;
}

function Probe() {
  const session = useSession();
  const view = useViewStore();
  const { push } = useNotices();
  return (
    <div>
      <output aria-label="client">{session.opened?.client.name ?? "-"}</output>
      <output aria-label="chart">{String(view.model.chartId)}</output>
      <output aria-label="path">{view.model.dashaPath.join(">")}</output>
      <output aria-label="chara">{`${view.model.charaPath.cycle}:${view.model.charaPath.signs.join(">")}`}</output>
      <output aria-label="compat">{`${view.model.compat.partnerUuid ?? "none"}:${view.model.compat.roles}`}</output>
      <output aria-label="tags">{session.opened?.chart_record.tags.join(",") ?? "-"}</output>
      <output aria-label="revision">{session.revision}</output>
      <button onClick={() => view.setDashaPath(["Rahu"])}>path</button>
      <button onClick={() => void session.openClient(2)}>open2</button>
      <button onClick={() => session.close()}>close</button>
      <button onClick={() => session.setTags(1, ["araştırma"])}>tags</button>
      <button onClick={() => push({ id: "boot", type: "error", messageKey: "notice.restoreFailed", error: { code: "ipc", params: {} } })}>boot</button>
      <NoticeBar />
    </div>
  );
}

const out = (name: string) => screen.getByRole("status", { name });

beforeEach(() => vi.mocked(api.openClient).mockReset());

test("openClient flushes the previous chart, opens the reply and swaps the notices", async () => {
  vi.mocked(api.openClient).mockResolvedValue({ status: "ok", data: other() });
  const { persistence } = renderWithStores(<Probe />, { debounceMs: 10_000 });
  fireEvent.click(screen.getByText("boot"));
  fireEvent.click(screen.getByText("path"));
  fireEvent.click(screen.getByText("open2"));
  await waitFor(() => expect(out("client")).toHaveTextContent("İkinci Danışan"));
  expect(api.openClient).toHaveBeenCalledWith(2, null);
  // Bekleyen kayıt önceki haritaya (1) yazıldı; açılan harita kirli değildir.
  expect(persistence.saved.map((s) => [s.chartId, s.viewState.dasha_expanded_path])).toEqual([[1, ["Rahu"]]]);
  expect(out("chart")).toHaveTextContent("9");
  expect(out("path")).toHaveTextContent("Venus");
  // Açılan haritanın chara dasha yolu da yüklenir.
  expect(out("chara")).toHaveTextContent("2:Gemini");
  // Eş bağı da açılan haritanınkidir (önceki harita eşsizdi) ve yeniden yazılmaz.
  expect(out("compat")).toHaveTextContent("4b0f2a8e-0000-4000-8000-00000000000a:OpenIsBride");
  expect(persistence.saved).toHaveLength(1);
  expect(screen.queryByText(/Son oturum açılamadı/)).toBeNull();
  expect(screen.getByText("Son açılan harita silinmiş; en son güncellenen harita açıldı.")).toBeInTheDocument();
  expect(out("revision")).toHaveTextContent("1");
});

test("only the latest openClient reply is opened and an error leaves the chart as it is", async () => {
  let first!: (v: { status: "ok"; data: OpenedClient }) => void;
  vi.mocked(api.openClient)
    .mockReturnValueOnce(new Promise((r) => (first = r)))
    .mockResolvedValueOnce({ status: "error", error: { code: "not_found", params: { what: "client", id: "2" } } });
  renderWithStores(<Probe />);
  fireEvent.click(screen.getByText("open2"));
  fireEvent.click(screen.getByText("open2"));
  await act(async () => first({ status: "ok", data: other() }));
  expect(out("client")).toHaveTextContent("Örnek Danışan");
  expect(out("chart")).toHaveTextContent("1");
});

test("close empties the desk and saves the pending change to the closed chart", () => {
  const { persistence } = renderWithStores(<Probe />, { debounceMs: 10_000 });
  fireEvent.click(screen.getByText("path"));
  fireEvent.click(screen.getByText("close"));
  expect(out("client")).toHaveTextContent("-");
  expect(out("chart")).toHaveTextContent("null");
  expect(persistence.saved.map((s) => s.chartId)).toEqual([1]);
});

test("setTags updates the open chart's tags and bumps the revision", () => {
  renderWithStores(<Probe />);
  fireEvent.click(screen.getByText("tags"));
  expect(out("tags")).toHaveTextContent("araştırma");
  expect(out("revision")).toHaveTextContent("1");
});

test("setPrashna rewrites the side record of the open prashna only", () => {
  function Side() {
    const session = useSession();
    const meta = session.opened?.prashna;
    return (
      <div>
        <output aria-label="prashna">{meta ? `${meta.focus_house ?? "-"}|${meta.number ?? "-"}|${meta.question}` : "none"}</output>
        <output aria-label="revision">{session.revision}</output>
        <button onClick={() => meta && session.setPrashna(session.opened!.chart_record.id, { ...meta, question: "Yeni", focus_house: null })}>mine</button>
        <button onClick={() => session.setPrashna(9999, { question: "Başka", number: 3, focus_house: 2, created_at: "t" })}>other</button>
        <button onClick={() => session.setPrashna(session.opened!.chart_record.id, { question: "Başka", number: 3, focus_house: 2, created_at: "t" })}>force</button>
      </div>
    );
  }
  const first = renderWithStores(<Side />, { opened: fixture().prashna.opened_number });
  expect(out("prashna")).toHaveTextContent("7|54|Deneme sorusu: yolculuk bu ay olur mu?");
  // Başka haritanın geç gelen kaydı açık haritaya yazılmaz.
  fireEvent.click(screen.getByText("other"));
  expect(out("prashna")).toHaveTextContent("7|54|Deneme sorusu");
  fireEvent.click(screen.getByText("mine"));
  expect(out("prashna")).toHaveTextContent("-|54|Yeni");
  // Listeleri etkilemez: revizyon artmaz.
  expect(out("revision")).toHaveTextContent("0");
  first.unmount();
  // Prashna olmayan harita bir kayıtla prashna olmaz.
  renderWithStores(<Side />);
  fireEvent.click(screen.getByText("force"));
  expect(out("prashna")).toHaveTextContent("none");
});
