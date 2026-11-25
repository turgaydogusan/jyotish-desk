// Yer arama kutusu (WAI-ARIA 1.2 combobox, liste açılır). Yazdıkça 200 ms sonra
// `search_places` çağrılır; eski isteğin geç gelen yanıtı yok sayılır. Atlas yoksa bilgi
// verilir ve elle koordinat girişi açık kalır (spec §12).
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
}

export function PlaceSearch({ value, onChange, onSelect, invalid, describedBy }: PlaceSearchProps) {
  const { t, i18n } = useTranslation();
  const id = useId();
  const [hits, setHits] = useState<PlaceHit[]>([]);
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(-1);
  const [error, setError] = useState<CommandError | null>(null);
  const [query, setQuery] = useState<string | null>(null);
  const seq = useRef(0);

  // Yalnızca kullanıcı yazınca aranır (`query`); seçim ya da dışarıdan değer araması tetiklemez.
  useEffect(() => {
    if (query === null) return;
    const mine = ++seq.current;
    if (query.trim() === "") {
      setHits([]);
      setOpen(false);
      return;
    }
    const timer = setTimeout(() => {
      void api.searchPlaces(query, SEARCH_LIMIT).then((r) => {
        if (mine !== seq.current) return;
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

  const choose = (hit: PlaceHit) => {
    seq.current++;
    setQuery(null);
    setOpen(false);
    setHits([]);
    onSelect(hit);
  };

  const onKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "ArrowDown" && hits.length > 0) {
      e.preventDefault();
      setOpen(true);
      setActive((a) => (a + 1) % hits.length);
    } else if (e.key === "ArrowUp" && hits.length > 0) {
      e.preventDefault();
      setOpen(true);
      setActive((a) => (a <= 0 ? hits.length - 1 : a - 1));
    } else if (e.key === "Enter" && open && active >= 0 && hits[active]) {
      e.preventDefault();
      choose(hits[active]);
    } else if (e.key === "Escape" && open) {
      // Liste kapanır; pencere açık kalır.
      e.preventDefault();
      e.stopPropagation();
      setOpen(false);
    }
  };

  const listId = `${id}-list`;
  const statusId = `${id}-status`;
  const shown = open && hits.length > 0;
  return (
    <div className="place-search">
      <input
        id={`${id}-input`}
        role="combobox"
        aria-autocomplete="list"
        aria-expanded={shown}
        aria-controls={listId}
        aria-activedescendant={shown && active >= 0 ? `${id}-opt-${active}` : undefined}
        aria-invalid={invalid || undefined}
        aria-describedby={[statusId, describedBy].filter(Boolean).join(" ")}
        aria-label={t("birth.place")}
        autoComplete="off"
        value={value}
        onChange={(e) => {
          onChange(e.target.value);
          setQuery(e.target.value);
        }}
        onKeyDown={onKeyDown}
        onBlur={() => setOpen(false)}
      />
      <ul id={listId} role="listbox" aria-label={t("birth.placeResults")} className="place-results" hidden={!shown}>
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
