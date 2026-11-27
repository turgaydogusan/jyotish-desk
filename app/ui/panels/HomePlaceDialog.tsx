// Bulunulan yer penceresi (ileri modüller spec'i §10): `AppPrefs.home_place`'i atlastan seçer,
// açık haritanın doğum yerinden alır ya da kaldırır. Gün panchangı ve muhurta araması bunu
// varsayılan yer olarak kullanır. Konum servisi kullanılmaz; yer kişisel veridir ve loglanmaz.
import { useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import type { CommandError, HomePlace, OpenedClient } from "../bindings";
import { PlaceSearch } from "../birth/PlaceSearch";
import { errorMessage } from "../i18n/errors";
import { chartPlace, hitPlace, placeLabel } from "../modules/muhurta";
import { usePrefs } from "../settings/prefs";
import { Dialog } from "../shell/Dialog";

/** Saat diliminin kısa yazımı: IANA kimliği, sabit ofset (saniye) ya da yerel ortalama saat. */
export function tzLabel(home: HomePlace, lmt: string): string {
  if (home.tz === "Lmt") return lmt;
  // `{ Fixed } & { Iana?: never }` biçiminde `in` daraltmaz; alan değerine bakılır.
  if (home.tz.Iana !== undefined) return home.tz.Iana;
  const secs = home.tz.Fixed;
  const abs = Math.abs(secs);
  const two = (n: number) => String(n).padStart(2, "0");
  return `UTC${secs < 0 ? "−" : "+"}${two(Math.floor(abs / 3600))}:${two(Math.floor((abs % 3600) / 60))}`;
}

export function HomePlaceDialog({ opened, onClose }: { opened: OpenedClient; onClose(): void }) {
  const { t, i18n } = useTranslation();
  const { prefs, save } = usePrefs();
  const id = useId();
  const [text, setText] = useState("");
  const [error, setError] = useState<CommandError | null>(null);
  const [saving, setSaving] = useState(false);
  const useChart = useRef<HTMLButtonElement>(null);
  const home = prefs?.home_place ?? null;
  const birth = chartPlace(opened);

  const store = async (next: HomePlace | null) => {
    if (prefs === null || saving) return;
    setSaving(true);
    const failed = await save({ ...prefs, home_place: next });
    setSaving(false);
    setError(failed);
    if (failed === null) setText("");
    // "Yeri kaldır" düğmesi yer kalkınca DOM'dan gider: odak pencerede bir sonraki eyleme taşınır.
    if (failed === null && next === null) useChart.current?.focus();
  };

  return (
    <Dialog title={t("muhurta.home.title")} onClose={onClose}>
      <div className="home-place">
        <p className="home-place-current" role="status">
          {home ? t("muhurta.home.current", { place: placeLabel(home), zone: tzLabel(home, t("muhurta.home.lmt")) }) : t("muhurta.home.unset")}
        </p>
        <p className="field-hint">{t("muhurta.home.hint")}</p>
        {prefs === null ? (
          <p className="panel-notice" role="note">
            {t("muhurta.home.noPrefs")}
          </p>
        ) : (
          <>
            <label htmlFor={`${id}-search`}>{t("muhurta.home.search")}</label>
            <PlaceSearch
              inputId={`${id}-search`}
              value={text}
              onChange={setText}
              onSelect={(hit) => {
                const place = hitPlace(hit);
                if (place) void store(place);
              }}
            />
            <div className="home-place-actions">
              <button ref={useChart} type="button" data-autofocus onClick={() => void store(birth)}>
                {t("muhurta.home.useChart", { place: placeLabel(birth) })}
              </button>
              {home && (
                <button type="button" onClick={() => void store(null)}>
                  {t("muhurta.home.remove")}
                </button>
              )}
            </div>
          </>
        )}
        {error && (
          <p className="panel-error" role="alert">
            {errorMessage(t, i18n, error)}
          </p>
        )}
        <div className="dialog-actions">
          <button type="button" onClick={onClose}>
            {t("muhurta.home.close")}
          </button>
        </div>
      </div>
    </Dialog>
  );
}
