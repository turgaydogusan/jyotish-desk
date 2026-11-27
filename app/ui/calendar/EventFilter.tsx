// Takvimin olay süzgeci (Plan 29): olay türleri ve Ay'ın kendi olayları. Seçim panelin ayarıdır
// ve düzenle birlikte saklanır; burada hesap yoktur (C4): seçim `month_calendar`'a gider ve
// olayları Rust arar. Doğum boylamına gelişler açık haritanın doğum verisini ister; veri yalnızca
// o tür seçiliyken gönderilir.
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { CalendarFilter } from "../bindings";
import type { Panel } from "../layout/model";
import { toggleMember } from "../modules/muhurta";
import { CheckSet } from "../search/CheckSet";
import "../search/search.css";
import { useLayoutStore } from "../view/store";
import { CALENDAR_KINDS } from "./model";

export function EventFilter({ panel, filter, natalApprox }: { panel: Panel; filter: CalendarFilter; natalApprox: boolean }) {
  const { t } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const id = useId();
  return (
    <div className="calendar-filter">
      <CheckSet
        legend={t("calendar.filter.kinds")}
        fold={t("calendar.filter.summary", { n: filter.kinds.length, of: CALENDAR_KINDS.length })}
        options={CALENDAR_KINDS.map((k) => ({ value: k, label: t(`eventSearch.kind.${k}`) }))}
        selected={filter.kinds}
        onToggle={(k) => setPanelSettings(panel.id, { kinds: toggleMember(filter.kinds, k, CALENDAR_KINDS) })}
      />
      <label htmlFor={`${id}-moon`}>
        <input id={`${id}-moon`} type="checkbox" checked={filter.moon} onChange={() => setPanelSettings(panel.id, { moon: !filter.moon })} />
        {t("calendar.filter.moon")}
      </label>
      {natalApprox && (
        <p className="panel-notice" role="note">
          {t("calendar.natalApprox")}
        </p>
      )}
    </div>
  );
}
