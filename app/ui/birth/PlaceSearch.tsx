// Yer arama kutusu (WAI-ARIA 1.2 combobox, liste açılır). Yazdıkça 200 ms sonra
// `search_places` çağrılır; eski isteğin geç gelen yanıtı yok sayılır. Atlas yoksa bilgi
// verilir ve elle koordinat girişi açık kalır (spec §12). Etkin seçenek kaydırılarak görünür
// tutulur; liste görünmüyorken Enter ve Esc tüketilmez (Esc pencereye gider); liste önceki metnin
// sonucuyken Enter seçmez.
import { useEffect, useId, useRef, useState, type KeyboardEvent } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { CommandError, PlaceHit } from "../bindings";
import { errorMessage } from "../i18n/errors";
import "./birth.css";

export const SEARCH_DEBOUNCE_MS = 200;
export const SEARCH_LIMIT = 8;

export interface PlaceSearchProps {
  value: string;
  onChange(text: string): void;
  onSelect(hit: PlaceHit): void;
  invalid?: boolean;
  describedBy?: string;
  /** Dışarıdaki `<label htmlFor>` için; verilmezse kutu kendi `aria-label`'ını taşır. */
  inputId?: string;
}

export function PlaceSearch({ value, onChange, onSelect, invalid, describedBy, inputId }: PlaceSearchProps) {
  const { t, i18n } = useTranslation();
  const id = useId();
  const [hits, setHits] = useState<PlaceHit[]>([]);
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(-1);
  const [error, setError] = useState<CommandError | null>(null);
  const [query, setQuery] = useState<string | null>(null);
  // Liste hangi metnin yanıtı: yazım sürüp yanıt gelmediyse liste eskidir.
  const [answered, setAnswered] = useState<string | null>(null);
  const stale = query !== null && query !== answered;
  const seq = useRef(0);
  const listRef = useRef<HTMLUListElement>(null);
  const shown = open && hits.length > 0;

  // Yalnızca kullanıcı yazınca aranır (`query`); seçim ya da dışarıdan değer araması tetiklemez.
  useEffect(() => {
    if (query === null) return;
    const mine = ++seq.current;
    if (query.trim() === "") {
      // Alan boşalınca eski aramanın hatası da kalkar.
      setError(null);
      setHits([]);
      setOpen(false);
      return;
    }
    const timer = setTimeout(() => {
      void api.searchPlaces(query, SEARCH_LIMIT).then((r) => {
        if (mine !== seq.current) return;
        setAnswered(query);
        if (r.status === "error") {
          setError(r.error);
          setHits([]);
          setOpen(false);
          return;
        }
        setError(null);
        setHits(r.data);
        setActive(r.data.length > 0 ? 0 : -1);
        setOpen(true);
      });
    }, SEARCH_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [query]);

  // Etkin seçenek (aria-activedescendant) kaydırılan listede görünür tutulur.
  useEffect(() => {
    if (shown && active >= 0) listRef.current?.children[active]?.scrollIntoView({ block: "nearest" });
  }, [shown, active, hits]);

  const choose = (hit: PlaceHit) => {
    seq.current++;
    setQuery(null);
    setOpen(false);
    setHits([]);
    onSelect(hit);
  };

  const onKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    // IME bileşimi sürerken oklar aday listesinde gezer, Enter adayı onaylar (WebKit bunu
    // 229 koduyla bildirir): seçenek değişmez, seçilmez.
    if (e.nativeEvent.isComposing || e.keyCode === 229) return;
    if (e.key === "ArrowDown" && hits.length > 0) {
      e.preventDefault();
      setOpen(true);
      setActive((a) => (a + 1) % hits.length);
    } else if (e.key === "ArrowUp" && hits.length > 0) {
      e.preventDefault();
      setOpen(true);
      setActive((a) => (a <= 0 ? hits.length - 1 : a - 1));
    } else if (e.key === "Enter" && shown) {
      // Görünen liste önceki metnin sonucuysa Enter tüketilir ama seçmez (eski yer seçilmez).
      e.preventDefault();
      if (!stale && active >= 0 && hits[active]) choose(hits[active]);
    } else if (e.key === "Escape" && shown) {
      // Görünen liste kapanır; pencere açık kalır. Liste görünmüyorsa (sonuç yok ya da
      // kapalı) Esc tüketilmez ve pencereyi kapatır.
      e.preventDefault();
      e.stopPropagation();
      setOpen(false);
    }
  };

  const listId = `${id}-list`;
  const statusId = `${id}-status`;
  return (
    <div className="place-search">
      <input
        id={inputId ?? `${id}-input`}
        role="combobox"
        aria-autocomplete="list"
        aria-expanded={shown}
        aria-controls={listId}
        aria-activedescendant={shown && active >= 0 ? `${id}-opt-${active}` : undefined}
        aria-invalid={invalid || undefined}
        aria-describedby={[statusId, describedBy].filter(Boolean).join(" ")}
        aria-label={inputId ? undefined : t("birth.place")}
        autoComplete="off"
        value={value}
        onChange={(e) => {
          onChange(e.target.value);
          setQuery(e.target.value);
        }}
        onKeyDown={onKeyDown}
        onBlur={() => setOpen(false)}
      />
      <ul ref={listRef} id={listId} role="listbox" aria-label={t("birth.placeResults")} className="place-results" hidden={!shown} aria-busy={stale || undefined}>
        {hits.map((h, i) => (
          <li
            key={h.geoname_id}
            id={`${id}-opt-${i}`}
            role="option"
            aria-selected={i === active}
            className={i === active ? "is-active" : undefined}
            // mousedown: input'un blur'u listeyi kapatmadan seçilir.
            onMouseDown={(e) => {
              e.preventDefault();
              choose(h);
            }}
          >
            <span>{h.name}</span>
            <span className="place-meta">
              {[h.admin1, h.country].filter(Boolean).join(", ")} · {h.timezone}
            </span>
          </li>
        ))}
      </ul>
      <p id={statusId} className="field-hint" role="status">
        {error ? errorMessage(t, i18n, error) : open && query ? t("birth.placeCount", { count: hits.length }) : ""}
      </p>
    </div>
  );
}
