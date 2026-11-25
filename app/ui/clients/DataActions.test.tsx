import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { vi } from "vitest";

vi.mock("../api", () => ({
  api: {
    exportClientsJson: vi.fn(),
    exportClientsCsv: vi.fn(),
    importClientsJson: vi.fn(),
  },
}));

import { api } from "../api";
import { useSession } from "../session/store";
import { NoticeBar } from "../shell/notices";
import { renderWithStores } from "../test/render";
import { DataActions, resetExchangeBusy } from "./DataActions";

const ok = <T,>(data: T) => ({ status: "ok" as const, data });
const report = { clients_added: 2, charts_added: 3, notes_added: 0, skipped: 1, warnings: [] };

function Revision() {
  return <output aria-label="sürüm">{useSession().revision}</output>;
}

function renderData() {
  return renderWithStores(
    <>
      <NoticeBar />
      <Revision />
      <DataActions />
    </>,
  );
}

const section = () => screen.getByRole("region", { name: "Veri" });

beforeEach(() => {
  resetExchangeBusy();
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
});

test("JSON export asks about notes first, unchecked by default, and reports the saved file name", async () => {
  vi.mocked(api.exportClientsJson).mockResolvedValue(ok("danisanlar.json"));
  renderData();
  fireEvent.click(within(section()).getByRole("button", { name: "JSON olarak dışa aktar…" }));
  const dialog = screen.getByRole("dialog", { name: "JSON olarak dışa aktar" });
  const notes = within(dialog).getByRole("checkbox", { name: "Notları dahil et" });
  expect(notes).not.toBeChecked();
  expect(notes).toHaveAccessibleDescription("Notlar hassas veridir; dosyayı kiminle paylaştığınıza dikkat edin.");
  fireEvent.click(within(dialog).getByRole("button", { name: "Dosya seç ve kaydet…" }));
  expect(await within(await screen.findByRole("region", { name: "Bildirimler" })).findByRole("status")).toHaveTextContent("“danisanlar.json” kaydedildi.");
  expect(api.exportClientsJson).toHaveBeenCalledWith(false);
  expect(screen.queryByRole("dialog")).toBeNull();
});

test("checking Include notes exports with notes; cancelling the file dialog says nothing", async () => {
  vi.mocked(api.exportClientsJson).mockResolvedValue(ok(null));
  renderData();
  fireEvent.click(within(section()).getByRole("button", { name: "JSON olarak dışa aktar…" }));
  const dialog = screen.getByRole("dialog", { name: "JSON olarak dışa aktar" });
  fireEvent.click(within(dialog).getByRole("checkbox", { name: "Notları dahil et" }));
  fireEvent.click(within(dialog).getByRole("button", { name: "Dosya seç ve kaydet…" }));
  await waitFor(() => expect(api.exportClientsJson).toHaveBeenCalledWith(true));
  await waitFor(() => expect(within(section()).getByRole("button", { name: "JSON olarak dışa aktar…" })).toBeEnabled());
  expect(screen.queryByRole("region", { name: "Bildirimler" })).toBeNull();
});

test("Include notes is off again each time the export dialog opens", async () => {
  vi.mocked(api.exportClientsJson).mockResolvedValue(ok("danisanlar.json"));
  renderData();
  const open = () => {
    fireEvent.click(within(section()).getByRole("button", { name: "JSON olarak dışa aktar…" }));
    return screen.getByRole("dialog", { name: "JSON olarak dışa aktar" });
  };
  let dialog = open();
  fireEvent.click(within(dialog).getByRole("checkbox", { name: "Notları dahil et" }));
  fireEvent.click(within(dialog).getByRole("button", { name: "Dosya seç ve kaydet…" }));
  await waitFor(() => expect(within(section()).getByRole("button", { name: "JSON olarak dışa aktar…" })).toBeEnabled());
  expect(api.exportClientsJson).toHaveBeenLastCalledWith(true);
  // İkinci açılış: önceki seçim taşınmaz (notlar hassas veridir, spec §5).
  dialog = open();
  expect(within(dialog).getByRole("checkbox", { name: "Notları dahil et" })).not.toBeChecked();
  // İşaretleyip vazgeçmek de seçimi taşımaz.
  fireEvent.click(within(dialog).getByRole("checkbox", { name: "Notları dahil et" }));
  fireEvent.click(within(dialog).getByRole("button", { name: "Vazgeç" }));
  dialog = open();
  expect(within(dialog).getByRole("checkbox", { name: "Notları dahil et" })).not.toBeChecked();
  fireEvent.click(within(dialog).getByRole("button", { name: "Dosya seç ve kaydet…" }));
  await waitFor(() => expect(api.exportClientsJson).toHaveBeenLastCalledWith(false));
});

test("CSV export errors are shown with the file name only", async () => {
  vi.mocked(api.exportClientsCsv).mockResolvedValue({ status: "error", error: { code: "file_io", params: { file: "haritalar.csv", detail: "permission denied" } } });
  renderData();
  fireEvent.click(within(section()).getByRole("button", { name: "CSV olarak dışa aktar…" }));
  expect(await screen.findByRole("alert")).toHaveTextContent("Dışa aktarılamadı: Dosya okunamadı ya da yazılamadı (haritalar.csv: permission denied).");
});

test("an import reports what was added and refreshes the lists", async () => {
  vi.mocked(api.importClientsJson).mockResolvedValue(ok({ file: "yedek.json", report }));
  renderData();
  expect(screen.getByRole("status", { name: "sürüm" })).toHaveTextContent("0");
  fireEvent.click(within(section()).getByRole("button", { name: "JSON içe aktar…" }));
  await waitFor(() => expect(screen.getByRole("status", { name: "sürüm" })).toHaveTextContent("1"));
  expect(screen.getByRole("region", { name: "Bildirimler" })).toHaveTextContent(
    "“yedek.json” içe aktarıldı: 2 danışan, 3 harita, 0 not eklendi; 1 kayıt zaten vardı.",
  );
});

test("an import with reset settings is a warning; a refused import adds nothing and says so", async () => {
  vi.mocked(api.importClientsJson)
    .mockResolvedValueOnce(ok({ file: "eski.json", report: { ...report, warnings: [{ item: "u", code: "CalcSettingsReset" as const }] } }))
    .mockResolvedValueOnce({ status: "error", error: { code: "import_too_large", params: { file: "dev.json", max_mb: "32" } } });
  renderData();
  const button = within(section()).getByRole("button", { name: "JSON içe aktar…" });
  fireEvent.click(button);
  expect(await screen.findByText(/1 haritanın hesap ayarı okunamadı/)).toBeInTheDocument();
  await waitFor(() => expect(button).not.toHaveAttribute("aria-disabled"));
  fireEvent.click(button);
  expect(await screen.findByRole("alert")).toHaveTextContent("İçe aktarılamadı; hiçbir kayıt eklenmedi: “dev.json” çok büyük; en çok 32 MB içe aktarılabilir.");
  expect(screen.getByRole("status", { name: "sürüm" })).toHaveTextContent("1");
});

test("while a file dialog is open the buttons say why and nothing else starts", async () => {
  let release!: () => void;
  vi.mocked(api.exportClientsCsv).mockReturnValue(new Promise((r) => (release = () => r(ok(null)))));
  renderData();
  const csv = within(section()).getByRole("button", { name: "CSV olarak dışa aktar…" });
  fireEvent.click(csv);
  await waitFor(() => expect(csv).toHaveAttribute("aria-disabled", "true"));
  expect(within(section()).getByRole("status")).toHaveTextContent("Çalışıyor…");
  fireEvent.click(csv);
  fireEvent.click(within(section()).getByRole("button", { name: "JSON içe aktar…" }));
  fireEvent.click(within(section()).getByRole("button", { name: "JSON olarak dışa aktar…" }));
  expect(api.exportClientsCsv).toHaveBeenCalledOnce();
  expect(api.importClientsJson).not.toHaveBeenCalled();
  expect(screen.queryByRole("dialog")).toBeNull();
  release();
  await waitFor(() => expect(csv).not.toHaveAttribute("aria-disabled"));
  expect(within(section()).getByRole("status")).toHaveTextContent(/^$/);
});

test("busy survives closing and reopening the sidebar while a file dialog is open", async () => {
  let release!: () => void;
  vi.mocked(api.importClientsJson).mockReturnValue(new Promise((r) => (release = () => r(ok(null)))));
  const first = renderData();
  fireEvent.click(within(section()).getByRole("button", { name: "JSON içe aktar…" }));
  await waitFor(() => expect(within(section()).getByRole("button", { name: "JSON içe aktar…" })).toHaveAttribute("aria-disabled", "true"));
  first.unmount();
  renderData();
  const again = within(section()).getByRole("button", { name: "JSON içe aktar…" });
  expect(again).toHaveAttribute("aria-disabled", "true");
  fireEvent.click(again);
  expect(api.importClientsJson).toHaveBeenCalledOnce();
  release();
  await waitFor(() => expect(again).not.toHaveAttribute("aria-disabled"));
});
