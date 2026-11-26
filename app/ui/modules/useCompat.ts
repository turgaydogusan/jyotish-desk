// Uyum raporunu (`compute_compat`) isteyen kanca (ileri modüller spec'i §9). Eş harita açık
// haritanın görünüm kaydındaki bağdır (`ViewState.compat_partner_uuid`); eşin girdisi ve kısa
// kimliği store'dan okunur (`chart_input`, `chart_brief`), rapor iki girdiden Rust'ta hesaplanır.
// Arayüzde hesap yoktur (C4) ve rapor saklanmaz (K1). Eş harita silinmişse bağ kaldırılır:
// silinen bir danışanı gösteren ilişki bilgisi geç gelen bir kayıtla geri yazılmaz. Girdiler ve
// eş bağı kişisel veridir: loglanmaz.
import { useEffect } from "react";
import { api, type Result } from "../api";
import type { ChartBrief, ChartInput, CommandError, CompatRoles, Gender, KutaReport, KutaRole, OpenedClient } from "../bindings";
import { useSession } from "../session/store";
import { useViewStore } from "../view/store";
import { chartInput, inputKey } from "./input";
import { useModule } from "./useModule";

/** Eş haritanın store'dan okunan kısmı. */
export interface CompatPartner {
  uuid: string;
  brief: ChartBrief;
  input: ChartInput;
}

export interface CompatState {
  /** Bağ var ve eş harita okundu. */
  partner: CompatPartner | null;
  /** Açık haritanın rolü. */
  openRole: KutaRole;
  report: KutaReport | null;
  error: CommandError | null;
  loading: boolean;
  retry(): void;
  /** Eşi atar; ilk roller iki danışanın cinsiyetinden (`initialRoles`). */
  setPartner(chartUuid: string, partnerGender: Gender | null): void;
  clearPartner(): void;
  swapRoles(): void;
}

/**
 * İlk rol ataması: açık haritanın danışanı kadın ve eşin danışanı erkekse açık harita gelin
 * rolünü alır; diğer her durumda (cinsiyet boş, aynı ya da "diğer") açık harita "1. kişi (damat
 * rolü)" olur. Kullanıcı rolleri her zaman değiştirebilir.
 */
export function initialRoles(open: Gender | null, partner: Gender | null): CompatRoles {
  return open === "Female" && partner === "Male" ? "OpenIsBride" : "OpenIsGroom";
}

/** Rollerin iki girdisi: `[damat rolü, gelin rolü]`. */
export function compatInputs(open: ChartInput, partner: ChartInput, roles: CompatRoles): [ChartInput, ChartInput] {
  return roles === "OpenIsGroom" ? [open, partner] : [partner, open];
}

async function loadPartner(uuid: string): Promise<Result<CompatPartner | null>> {
  const [brief, input] = await Promise.all([api.chartBrief(uuid), api.chartInput(uuid)]);
  if (brief.status === "error") return brief;
  if (input.status === "error") return input;
  // Harita silinmişse ikisi de `null` döner.
  if (brief.data === null || input.data === null) return { status: "ok", data: null };
  return { status: "ok", data: { uuid, brief: brief.data, input: input.data } };
}

/** `enabled: false` iken hiçbir komut çağrılmaz (ör. eş halkası kapalı harita paneli). */
export function useCompat(opened: OpenedClient | null, enabled = true): CompatState {
  const view = useViewStore();
  const { revision } = useSession();
  const { partnerUuid, roles } = view.model.compat;
  const active = enabled && opened !== null && view.model.chartId === opened.chart_record.id;
  // `revision` kayıt değişikliklerinde artar (silme, düzenleme): eş yeniden okunur.
  const partnerKey = active && partnerUuid !== null ? JSON.stringify([opened.chart_record.id, partnerUuid, revision]) : null;
  const lookup = useModule(partnerKey, () => loadPartner(partnerUuid as string));
  const partner = lookup.data ?? null;
  const missing = partnerKey !== null && !lookup.loading && lookup.error === null && lookup.data === null;

  const { setCompatPartner } = view;
  useEffect(() => {
    if (missing) setCompatPartner(null, "OpenIsGroom");
  }, [missing, setCompatPartner]);

  const reportKey = active && partner !== null ? JSON.stringify([inputKey(opened), partner.input, roles]) : null;
  const report = useModule(reportKey, () => {
    const [groom, bride] = compatInputs(chartInput(opened as OpenedClient), (partner as CompatPartner).input, roles);
    return api.computeCompat(groom, bride);
  });

  return {
    partner,
    openRole: roles === "OpenIsGroom" ? "Groom" : "Bride",
    report: report.data,
    error: lookup.error ?? report.error,
    loading: lookup.loading || report.loading,
    retry: lookup.error ? lookup.retry : report.retry,
    setPartner: (chartUuid, partnerGender) => setCompatPartner(chartUuid, initialRoles(opened?.client.gender ?? null, partnerGender)),
    clearPartner: () => setCompatPartner(null, "OpenIsGroom"),
    swapRoles: view.swapCompatRoles,
  };
}
