// Muhurta panellerinin yer seçimi (ileri modüller spec'i §10 "Bulunulan yer"): bulunulan yer
// (tercihlerden), açık haritanın doğum yeri ya da panele özel bir yer (atlastan). Seçim panelin
// ayarıdır ve düzenle birlikte saklanır; bulunulan yerin kendisi tercihlerdedir. Burada hesap
// yoktur (C4).
import { useId, useState } from "react";
import { useTranslation } from "react-i18next";
import type { HomePlace, OpenedClient } from "../bindings";
import { PlaceSearch } from "../birth/PlaceSearch";
import type { Panel } from "../layout/model";
import { hitPlace, parseHomePlace, PLACE_CHOICES, placeChoice, placeLabel, resolvePlace, type PlaceChoice } from "../modules/muhurta";
import { usePrefs } from "../settings/prefs";
import { useLayoutStore } from "../view/store";
import { HomePlaceDialog } from "./HomePlaceDialog";

export interface PanelPlace {
  choice: PlaceChoice;
  /** Seçilen yer; bulunulan yer seçilmemişse ya da özel yer girilmemişse `null`. */
  place: HomePlace | null;
}

/** Panelin ayarından ve tercihlerden yer. */
export function usePanelPlace(panel: Panel, opened: OpenedClient): PanelPlace {
  const { prefs } = usePrefs();
  const choice = placeChoice(panel.settings);
  return { choice, place: resolvePlace(choice, opened, prefs, parseHomePlace(panel.settings.custom)) };
}

/** Yer seçicisi ve seçime bağlı denetimler: özel yer araması ya da bulunulan yer penceresi. */
export function PlaceControls({ panel, opened, at }: { panel: Panel; opened: OpenedClient; at: PanelPlace }) {
  const { t } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const id = useId();
  const [text, setText] = useState("");
  const [editingHome, setEditingHome] = useState(false);
  return (
    <>
      <label htmlFor={`${id}-place`}>{t("muhurta.place.label")}</label>
      <select id={`${id}-place`} value={at.choice} onChange={(e) => setPanelSettings(panel.id, { place: e.target.value })}>
        {PLACE_CHOICES.map((c) => (
          <option key={c} value={c}>
            {t(`muhurta.place.${c}`)}
          </option>
        ))}
      </select>
      {at.place && <span className="muhurta-place-name">{placeLabel(at.place)}</span>}
      {at.choice === "home" && (
        <button type="button" onClick={() => setEditingHome(true)}>
          {t(at.place ? "muhurta.place.changeHome" : "muhurta.place.pickHome")}
        </button>
      )}
      {at.choice === "custom" && (
        <span className="muhurta-place-search">
          <label htmlFor={`${id}-search`}>{t("muhurta.place.search")}</label>
          <PlaceSearch
            inputId={`${id}-search`}
            value={text}
            onChange={setText}
            onSelect={(hit) => {
              const place = hitPlace(hit);
              if (place === null) return;
              setText("");
              setPanelSettings(panel.id, { custom: place });
            }}
          />
        </span>
      )}
      {editingHome && <HomePlaceDialog opened={opened} onClose={() => setEditingHome(false)} />}
    </>
  );
}

/** Yer seçilmemişken panelin gösterdiği açıklama. */
export function PlaceMissing({ choice }: { choice: PlaceChoice }) {
  const { t } = useTranslation();
  return (
    <p className="panel-notice" role="note">
      {t(choice === "custom" ? "muhurta.place.missingCustom" : "muhurta.place.missingHome")}
    </p>
  );
}
