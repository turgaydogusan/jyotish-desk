// Düzen seçici ve işlemler menüsü: farklı kaydet, yeniden adlandır, kopyala, sil, sıfırla.
// `MAX_LAYOUTS` düzende "farklı kaydet" ve "kopyala" devre dışıdır (ipucu sınırı söyler).
// Ad girişi ve silme onayı satır içinde açılır (WKWebView'da window.prompt'a güvenilmez).
// Menü ARIA menü düzenini izler: açılınca ilk etkin öğe odaklanır, oklar/Home/End gezdirir,
// Esc kapatır (preventDefault: vurgu sabitlemesi korunur), Tab ve dış tıklama kapatır.
// Menü, form ve onay kapanınca odak tetik düğmesine döner.
import { useEffect, useId, useRef, useState, type FormEvent, type KeyboardEvent } from "react";
import { useTranslation } from "react-i18next";
import type { TFunction } from "i18next";
import { MAX_LAYOUTS, type Layout } from "../layout/model";
import { useLayoutStore } from "../view/store";

export function layoutLabel(t: TFunction, layout: Layout): string {
  return layout.builtin ? t(`layout.${layout.id.slice("builtin:".length)}`) : layout.name;
}

type Mode = null | "saveAs" | "rename" | "confirmDelete";
type NameError = null | "nameEmpty" | "nameTaken";

export function LayoutPicker() {
  const { t, i18n } = useTranslation();
  const store = useLayoutStore();
  const active = store.active;
  const [menuOpen, setMenuOpen] = useState(false);
  const [focusOnOpen, setFocusOnOpen] = useState<"first" | "last">("first");
  const [mode, setMode] = useState<Mode>(null);
  const [name, setName] = useState("");
  // Yeniden adlandırma/silme formu açıldığı andaki düzeni hedefler (etkin düzen sonradan değişebilir).
  const [targetId, setTargetId] = useState<string | null>(null);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const menuRef = useRef<HTMLUListElement>(null);
  const menuId = useId();
  const errorId = useId();
  const activeLabel = layoutLabel(t, active);
  const target = store.state.layouts.find((l) => l.id === targetId) ?? active;
  const targetLabel = layoutLabel(t, target);
  /** Düzen sınırı (`MAX_LAYOUTS`): yeni düzen eklenemez; düğmeler görünür biçimde kapanır. */
  const atCap = store.state.layouts.length >= MAX_LAYOUTS;

  const sameName = (a: string, b: string) =>
    a.trim().localeCompare(b.trim(), i18n.language, { sensitivity: "accent" }) === 0;
  const isTaken = (candidate: string, exceptId?: string) =>
    store.state.layouts.some((l) => l.id !== exceptId && sameName(layoutLabel(t, l), candidate));
  /** "X kopyası", doluysa "X kopyası 2", "X kopyası 3"… */
  const uniqueCopyName = () => {
    const first = t("layout.copyName", { name: activeLabel });
    if (!isTaken(first)) return first;
    for (let n = 2; ; n++) {
      const next = t("layout.copyNameN", { name: activeLabel, n });
      if (!isTaken(next)) return next;
    }
  };

  const nameError: NameError =
    name.trim() === "" ? "nameEmpty" : isTaken(name, mode === "rename" ? target.id : undefined) ? "nameTaken" : null;

  const focusTrigger = () => triggerRef.current?.focus();
  const enabledItems = () =>
    [...(menuRef.current?.querySelectorAll<HTMLButtonElement>('[role="menuitem"]') ?? [])].filter((b) => !b.disabled);

  const openMenu = (focus: "first" | "last") => {
    setFocusOnOpen(focus);
    setMenuOpen(true);
  };
  const closeMenu = (returnFocus: boolean) => {
    setMenuOpen(false);
    if (returnFocus) focusTrigger();
  };

  useEffect(() => {
    if (!menuOpen) return;
    const items = enabledItems();
    (focusOnOpen === "first" ? items[0] : items[items.length - 1])?.focus();
    const onPointerDown = (e: PointerEvent) => {
      const target = e.target as Node | null;
      if (menuRef.current?.contains(target) || triggerRef.current?.contains(target)) return;
      setMenuOpen(false);
    };
    document.addEventListener("pointerdown", onPointerDown);
    return () => document.removeEventListener("pointerdown", onPointerDown);
    // Yalnızca açılışta çalışır; focusOnOpen açılıştan önce ayarlanır.
  }, [menuOpen]);

  // Etkin düzen değişirse (Cmd+rakam, seçici) açık form kapanır; yanlış düzene işlem yapılmaz.
  useEffect(() => {
    setMode(null);
  }, [active.id]);

  const startMode = (next: Mode, initialName = "") => {
    setMenuOpen(false);
    setTargetId(active.id);
    setName(initialName);
    setMode(next);
  };
  const endMode = () => {
    setMode(null);
    focusTrigger();
  };

  const submitName = (e: FormEvent) => {
    e.preventDefault();
    if (nameError !== null) return;
    if (mode === "saveAs") store.saveAs(name.trim());
    if (mode === "rename") store.rename(target.id, name.trim());
    endMode();
  };

  const onTriggerKey = (e: KeyboardEvent) => {
    // Enter ve Boşluk yerel tıklamayla açar (onClick); oklar da açar.
    if (e.key === "ArrowDown") {
      e.preventDefault();
      openMenu("first");
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      openMenu("last");
    }
  };

  const onMenuKey = (e: KeyboardEvent) => {
    const items = enabledItems();
    const current = items.indexOf(document.activeElement as HTMLButtonElement);
    const move = (index: number) => {
      e.preventDefault();
      items[(index + items.length) % items.length]?.focus();
    };
    switch (e.key) {
      case "Escape":
        e.preventDefault();
        closeMenu(true);
        break;
      case "Tab":
        closeMenu(false);
        break;
      case "ArrowDown":
        move(current + 1);
        break;
      case "ArrowUp":
        move(current < 0 ? items.length - 1 : current - 1);
        break;
      case "Home":
        move(0);
        break;
      case "End":
        move(items.length - 1);
        break;
    }
  };

  const onInlineKey = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      endMode();
    }
  };

  const runItem = (action: () => void) => {
    action();
    closeMenu(true);
  };

  return (
    <div className="layout-picker">
      <label>
        {t("layout.picker")}
        <select value={active.id} onChange={(e) => store.select(e.target.value)}>
          {store.state.layouts.map((l) => (
            <option key={l.id} value={l.id}>
              {layoutLabel(t, l)}
            </option>
          ))}
        </select>
      </label>
      <button
        ref={triggerRef}
        type="button"
        aria-haspopup="menu"
        aria-expanded={menuOpen}
        aria-controls={menuOpen ? menuId : undefined}
        onClick={() => (menuOpen ? closeMenu(false) : openMenu("first"))}
        onKeyDown={onTriggerKey}
      >
        {t("layout.actions")}
      </button>
      {menuOpen && (
        <ul
          ref={menuRef}
          id={menuId}
          className="layout-menu"
          role="menu"
          aria-label={t("layout.actions")}
          onKeyDown={onMenuKey}
        >
          <li role="none">
            <button
              type="button"
              role="menuitem"
              tabIndex={-1}
              disabled={atCap}
              title={atCap ? t("layout.limit", { max: MAX_LAYOUTS }) : undefined}
              onClick={() => startMode("saveAs", uniqueCopyName())}
            >
              {t("layout.saveAs")}
            </button>
          </li>
          <li role="none">
            <button
              type="button"
              role="menuitem"
              tabIndex={-1}
              disabled={active.builtin}
              onClick={() => startMode("rename", active.name)}
            >
              {t("layout.rename")}
            </button>
          </li>
          <li role="none">
            <button
              type="button"
              role="menuitem"
              tabIndex={-1}
              disabled={atCap}
              title={atCap ? t("layout.limit", { max: MAX_LAYOUTS }) : undefined}
              onClick={() => runItem(() => store.duplicate(active.id, uniqueCopyName()))}
            >
              {t("layout.duplicate")}
            </button>
          </li>
          <li role="none">
            <button
              type="button"
              role="menuitem"
              tabIndex={-1}
              disabled={active.builtin}
              onClick={() => startMode("confirmDelete")}
            >
              {t("layout.delete")}
            </button>
          </li>
          <li role="none">
            <button
              type="button"
              role="menuitem"
              tabIndex={-1}
              disabled={!active.builtin}
              onClick={() => runItem(() => store.resetBuiltin(active.id))}
            >
              {t("layout.reset")}
            </button>
          </li>
        </ul>
      )}
      {(mode === "saveAs" || mode === "rename") && (
        <form className="layout-inline" onSubmit={submitName} onKeyDown={onInlineKey}>
          <label>
            {t("layout.name")}
            <input
              autoFocus
              value={name}
              aria-invalid={nameError !== null}
              aria-describedby={errorId}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={(e) => {
                // Kaydet devre dışıyken tarayıcı Enter ile göndermez; hata metni zaten görünür.
                if (e.key === "Enter" && nameError !== null) e.preventDefault();
              }}
            />
          </label>
          <span id={errorId} className="layout-error" aria-live="polite">
            {nameError !== null ? t(`layout.${nameError}`) : ""}
          </span>
          <button type="submit" disabled={nameError !== null}>
            {t("layout.save")}
          </button>
          <button type="button" onClick={endMode}>
            {t("layout.cancel")}
          </button>
        </form>
      )}
      {mode === "confirmDelete" && (
        <div className="layout-inline" role="group" aria-label={t("layout.delete")} onKeyDown={onInlineKey}>
          <span>{t("layout.confirmDelete", { name: targetLabel })}</span>
          <button
            type="button"
            onClick={() => {
              store.remove(target.id);
              endMode();
            }}
          >
            {t("layout.deleteNamed", { name: targetLabel })}
          </button>
          <button type="button" autoFocus onClick={endMode}>
            {t("layout.cancel")}
          </button>
        </div>
      )}
    </div>
  );
}
