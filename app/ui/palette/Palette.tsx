// Komut paleti (`Cmd+K`, spec §4 ve UI spec §4): danışan, açık danışanın haritaları, varga,
// düzen ve komut tek arama kutusunda. WAI-ARIA combobox + listbox; ↑/↓ etkin öğeyi taşır,
// Enter çalıştırır, Esc kapatır. Danışanda `Cmd+Enter` danışanı açar ve kenar çubuğunu
// gösterir (sekme yoktur, UI spec §4). Danışan araması `search_clients` iledir; eski yanıt
// yok sayılır. Seçimden haritaya form ya da onay penceresi çıkmaz (spec §8).
import { useEffect, useId, useMemo, useRef, useState, type KeyboardEvent } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { ChartRecord, SearchHit } from "../bindings";
import { VARGA_NAMES, VARGAS } from "../chart/adapter";
import { useSession } from "../session/store";
import { Dialog } from "../shell/Dialog";
import { layoutLabel } from "../shell/LayoutPicker";
import { useLayoutStore } from "../view/store";
import { clientItems, localItems, type PaletteCommand, type PaletteItem } from "./items";
import "./palette.css";

export const PALETTE_SEARCH_DEBOUNCE_MS = 120;
/** Boş sorguda gösterilen son açılan danışan sayısı; aramada en çok danışan. */
export const PALETTE_RECENT = 5;
export const PALETTE_CLIENTS = 8;

export interface PaletteProps {
  onClose(): void;
  /** Öğe seçildi; `meta`: Cmd+Enter. Palet önce kapanır, sonra çağrılır. */
  onRun(item: PaletteItem, meta: boolean): void;
}

export function Palette({ onClose, onRun }: PaletteProps) {
  const { t, i18n } = useTranslation();
  const { opened, revision } = useSession();
  const layout = useLayoutStore();
  const id = useId();
  const [query, setQuery] = useState("");
  const [hits, setHits] = useState<SearchHit[]>([]);
  const [charts, setCharts] = useState<ChartRecord[]>([]);
  const [active, setActive] = useState(0);
  const seq = useRef(0);
  const listRef = useRef<HTMLUListElement>(null);

  useEffect(() => {
    if (!opened) {
      setCharts([]);
      return;
    }
    let live = true;
    void api.listCharts(opened.client.id).then((r) => live && r.status === "ok" && setCharts(r.data));
    return () => {
      live = false;
    };
  }, [opened, revision]);

  useEffect(() => {
    const mine = ++seq.current;
    const q = query.trim();
    const timer = setTimeout(
      () => {
        const call =
          q === ""
            ? api.listClients(PALETTE_RECENT).then((r) => (r.status === "ok" ? r.data.map((client): SearchHit => ({ client, matched: "Name", snippet: null })) : []))
            : api.searchClients(q, PALETTE_CLIENTS).then((r) => (r.status === "ok" ? r.data : []));
        void call.then((data) => {
          if (mine === seq.current) setHits(data);
        });
      },
      q === "" ? 0 : PALETTE_SEARCH_DEBOUNCE_MS,
    );
    return () => clearTimeout(timer);
  }, [query]);

  const hasChartPanel = layout.active.panels.some((p) => p.kind === "chart");
  const items = useMemo(() => {
    const commands: { command: PaletteCommand; label: string }[] = [
      { command: "newChart", label: t("palette.command.newChart") },
      ...(opened
        ? [
            { command: "addChart" as const, label: t("palette.command.addChart") },
            { command: "editChart" as const, label: t("palette.command.editChart") },
            { command: "toggleNotes" as const, label: t("palette.command.toggleNotes") },
          ]
        : []),
      { command: "sidebar", label: t("palette.command.sidebar") },
      { command: "help", label: t("palette.command.help") },
    ];
    const clients = clientItems(hits, (h) => (h.matched === "Note" && h.snippet ? t("clients.matchedNote", { snippet: h.snippet }) : h.matched === "Tag" ? t("clients.matchedTag") : null));
    const local = localItems(
      {
        charts: opened ? charts : [],
        chartLabel: (c) => c.label || t(`chart.kind.${c.kind}`),
        layouts: layout.state.layouts,
        layoutLabel: (l) => layoutLabel(t, l),
        vargas: opened && hasChartPanel ? VARGAS.map((v) => ({ varga: v, label: `${v} ${VARGA_NAMES[v]}` })) : [],
        commands,
      },
      query,
      i18n.language,
    );
    return [...clients, ...local];
  }, [hits, charts, opened, layout.state.layouts, hasChartPanel, query, t, i18n.language]);

  useEffect(() => setActive(0), [query]);
  const current = Math.min(active, items.length - 1);

  // Etkin seçenek (aria-activedescendant) kaydırılan listede görünür tutulur (PlaceSearch gibi).
  useEffect(() => {
    listRef.current?.children[current]?.scrollIntoView({ block: "nearest" });
  }, [current, items]);

  const run = (item: PaletteItem, meta: boolean) => {
    onClose();
    onRun(item, meta);
  };

  const onKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.nativeEvent.isComposing) return;
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      if (items.length === 0) return;
      const step = e.key === "ArrowDown" ? 1 : -1;
      setActive((current + step + items.length) % items.length);
    } else if (e.key === "Enter") {
      e.preventDefault();
      const item = items[current];
      if (item) run(item, e.metaKey);
    }
  };

  const kindLabel = (item: PaletteItem) => t(`palette.kind.${item.kind}`);
  const optionId = (i: number) => `${id}-opt-${i}`;
  return (
    <Dialog title={t("palette.title")} onClose={onClose}>
      <div className="palette">
        <label htmlFor={`${id}-q`} className="visually-hidden">
          {t("palette.search")}
        </label>
        <input
          id={`${id}-q`}
          data-autofocus
          role="combobox"
          aria-expanded="true"
          aria-controls={`${id}-list`}
          aria-autocomplete="list"
          aria-activedescendant={items.length > 0 ? optionId(current) : undefined}
          aria-describedby={`${id}-hint`}
          placeholder={t("palette.placeholder")}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={onKeyDown}
        />
        <p id={`${id}-hint`} className="palette-hint">
          {t("palette.hint")}
        </p>
        <ul ref={listRef} id={`${id}-list`} role="listbox" aria-label={t("palette.results")} className="palette-list">
          {items.map((item, i) => (
            <li
              key={item.key}
              id={optionId(i)}
              role="option"
              aria-selected={i === current}
              className={i === current ? "is-active" : undefined}
              onMouseDown={(e) => e.preventDefault()}
              onClick={(e) => run(item, e.metaKey)}
            >
              <span className="palette-kind">{kindLabel(item)}</span>
              <span className="palette-label">{item.label}</span>
              {"detail" in item && item.detail && <span className="palette-detail">{item.detail}</span>}
            </li>
          ))}
        </ul>
        {items.length === 0 && <p className="palette-empty">{t("palette.empty")}</p>}
        <p className="visually-hidden" role="status">
          {t("palette.count", { n: items.length })}
        </p>
      </div>
    </Dialog>
  );
}
