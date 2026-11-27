// İleri modül komutlarının ortak girdisi (ileri modüller spec'i K2). Değerler açık haritadan
// alınır; burada hesap yoktur (C4). Modül sonucu saklanmaz: panel açıkken istenir (K1).
import type { ChartInput, OpenedClient } from "../bindings";
import { timeUnknown } from "../session/birth";

/** Açık haritanın modül girdisi: hesaplanan haritanın anı, konumu ve ayarları. */
export function chartInput(opened: OpenedClient): ChartInput {
  const { chart } = opened;
  return { jd_ut: chart.jd_ut, lat: chart.geo_lat, lon: chart.geo_lon, settings: chart.meta.settings, time_known: !timeUnknown(opened), lagna_number: null };
}

/**
 * Sonucun ait olduğu girdinin anahtarı. Başka harita açılınca, doğum verisi düzenlenince ya da
 * hesap ayarı değişince anahtar değişir ve sonuç yeniden istenir. Kişisel veridir: loglanmaz.
 */
export function inputKey(opened: OpenedClient): string {
  return JSON.stringify([opened.chart_record.id, chartInput(opened)]);
}
