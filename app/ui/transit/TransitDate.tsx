// Transit anı seçici: duvar saati tarayıcının IANA diliminde gönderilir, Rust çözer (boşlukta
// hata ve öneri, örtüşmede önceki an; Plan 11 D7). "Şimdi" anı Rust'un saatine bırakır.
import { useId, useState, type FormEvent } from "react";
import { useTranslation } from "react-i18next";
import { browserTimeZone, formatMoment, localParts, useTransitControl } from "./store";

/** `quiet`: an sık değişirken (oynatma) gösterilen an ekran okuyucuya duyurulmaz. */
export function TransitDate({ quiet = false }: { quiet?: boolean } = {}) {
  const { t, i18n } = useTranslation();
  const { at, view, setAt } = useTransitControl();
  const id = useId();
  const zone = browserTimeZone();
  const shown = view ? localParts(view.utc, zone) : null;
  const [date, setDate] = useState(shown?.date ?? "");
  const [time, setTime] = useState(shown?.time ?? "");
  const [editing, setEditing] = useState(false);

  // Gösterilen an değişince (şimdi, şerit) kutular onu gösterir; kullanıcı yazarken değişmez.
  // Eşitleme çizim sırasında yapılır, etkide değil: etki bir sonraki işe ertelenebilir ve araya
  // giren ilk tuş vuruşunun üzerine yazardı (yük altında görülen test dalgalanmasının nedeni).
  const target = editing || !shown ? null : `${shown.date} ${shown.time}`;
  const [synced, setSynced] = useState(target);
  if (target !== synced) {
    setSynced(target);
    if (shown && target !== null) {
      setDate(shown.date);
      setTime(shown.time);
    }
  }

  const apply = (e: FormEvent) => {
    e.preventDefault();
    if (!date || !time) return;
    setEditing(false);
    setAt({ Local: { date, time, tz: zone } });
  };

  const isNow = at === "Now";
  return (
    <form className="transit-date" aria-label={t("transit.date")} onSubmit={apply}>
      <label htmlFor={`${id}-date`}>{t("transit.dateLabel")}</label>
      <input
        id={`${id}-date`}
        type="date"
        value={date}
        onChange={(e) => {
          setEditing(true);
          setDate(e.target.value);
        }}
      />
      <label htmlFor={`${id}-time`}>{t("transit.timeLabel")}</label>
      <input
        id={`${id}-time`}
        type="time"
        value={time}
        onChange={(e) => {
          setEditing(true);
          setTime(e.target.value);
        }}
      />
      <span className="transit-zone">{zone}</span>
      <button type="submit">{t("transit.apply")}</button>
      <button
        type="button"
        aria-pressed={isNow}
        onClick={() => {
          setEditing(false);
          setAt("Now");
        }}
      >
        {t("transit.now")}
      </button>
      <span className="transit-shown" role="status" aria-live={quiet ? "off" : undefined}>
        {view ? t(isNow ? "transit.shownNow" : "transit.shown", { when: formatMoment(view.utc, i18n.language, zone) }) : ""}
      </span>
    </form>
  );
}
