import { screen, within } from "@testing-library/react";
import type { OpenedClient } from "../bindings";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { BirthInfoPanel } from "./BirthInfoPanel";

const panel: Panel = { id: "b", kind: "birth_info", x: 0, y: 0, w: 4, h: 6, settings: {} };

function renderInfo(opened: OpenedClient | null = fixture().opened) {
  return renderWithStores(<BirthInfoPanel panel={panel} />, { opened });
}

/** `<dt>` metnine göre yanındaki `<dd>`. */
function value(term: string): string {
  const dt = screen.getAllByRole("term").find((d) => d.textContent === term);
  if (!dt) throw new Error(`no term ${term}`);
  return dt.nextElementSibling?.textContent ?? "";
}

test("shows the stored birth data and the moment Rust resolved", () => {
  renderInfo();
  expect(screen.getByRole("region", { name: "Doğum bilgisi" })).toBeInTheDocument();
  expect(value("Tarih")).toBe("2000-01-01");
  expect(value("Saat")).toBe("17:30");
  expect(value("Saat kesinliği")).toBe("Kesin");
  expect(value("Yer")).toBe("New Delhi, Delhi, India");
  expect(value("Koordinat")).toBe("28.6139, 77.2090");
  expect(value("Saat dilimi")).toBe("Asia/Kolkata");
  expect(value("UTC anı")).toBe("2000-01-01 12:00:00 UTC");
  expect(value("Ofset")).toBe("IST +05:30");
  expect(value("tzdb sürümü")).toBe("2026f");
  expect(value("Cinsiyet")).toBe("Belirtilmemiş");
  expect(value("Harita türü")).toBe("Doğum haritası");
  // `term` rolünün adı içeriğinden gelmez: ada göre sorgu her zaman boş dönerdi. Metne bakılır.
  expect(screen.getAllByRole("term").map((d) => d.textContent)).not.toContain("Kaynak notu");
  expect(screen.getByText("Uyarı yok.")).toBeInTheDocument();
});

test("an unknown time says so and its warning is explained in text, not only in a badge", () => {
  const o = unknownTimeOpened();
  o.chart.meta.warnings = ["LowPrecision"];
  renderInfo(o);
  expect(value("Saat")).toBe("Bilinmiyor");
  const warnings = within(screen.getByRole("list", { name: "Uyarılar" })).getAllByRole("listitem");
  expect(warnings.map((w) => w.textContent)).toEqual([
    "Saat bilinmiyor: Doğum saati bilinmiyor; öğlen 12:00 alındı.",
    "Düşük hassasiyet: Güneş veya Ay yedek ephemeris ile hesaplandı.",
  ]);
});

test("fixed offset, LMT, manual coordinates, gender, source note; no chart open", () => {
  const o = fixture().opened;
  o.chart_record.birth.tz = { Fixed: -17_776 };
  o.chart_record.birth.place = { ...o.chart_record.birth.place, name: "", admin1: "", country: "" };
  o.chart_record.birth.notes = "Nüfus kaydı";
  o.client.gender = "Female";
  const { unmount } = renderInfo(o);
  expect(value("Saat dilimi")).toBe("-04:56:16");
  expect(value("Yer")).toBe("Elle girilmiş koordinat");
  expect(value("Cinsiyet")).toBe("Kadın");
  expect(value("Kaynak notu")).toBe("Nüfus kaydı");
  unmount();
  o.chart_record.birth.tz = "Lmt";
  const second = renderInfo(o);
  expect(value("Saat dilimi")).toBe("Yerel ortalama saat (LMT)");
  second.unmount();
  renderInfo(null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
});
