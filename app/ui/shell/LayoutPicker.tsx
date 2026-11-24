// Düzen seçici ve işlemler menüsü: farklı kaydet, yeniden adlandır, kopyala, sil, sıfırla.
// Ad girişi ve silme onayı satır içinde açılır (WKWebView'da window.prompt'a güvenilmez).
import { useState, type FormEvent, type KeyboardEvent } from "react";
import { useTranslation } from "react-i18next";
import type { TFunction } from "i18next";
import type { Layout } from "../layout/model";
import { useLayoutStore } from "../layout/store";

export function layoutLabel(t: TFunction, layout: Layout): string {
  return layout.builtin ? t(`layout.${layout.id.slice("builtin:".length)}`) : layout.name;
}

type Mode = null | "saveAs" | "rename" | "confirmDelete";

export function LayoutPicker() {
  const { t } = useTranslation();
  const store = useLayoutStore();
  const active = store.active;
  const [menuOpen, setMenuOpen] = useState(false);
  const [mode, setMode] = useState<Mode>(null);
  const [name, setName] = useState("");
  const activeLabel = layoutLabel(t, active);

  const open = (next: Mode, initialName = "") => {
    setMenuOpen(false);
    setName(initialName);
    setMode(next);
  };
  const close = () => setMode(null);

  const submitName = (e: FormEvent) => {
    e.preventDefault();
    if (name.trim() === "") return;
    if (mode === "saveAs") store.saveAs(name);
    if (mode === "rename") store.rename(active.id, name);
    close();
  };

  const onMenuKey = (e: KeyboardEvent) => {
    if (e.key === "Escape") setMenuOpen(false);
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
      <button type="button" aria-haspopup="menu" aria-expanded={menuOpen} onClick={() => setMenuOpen((o) => !o)}>
        {t("layout.actions")}
      </button>
      {menuOpen && (
        <ul className="layout-menu" role="menu" aria-label={t("layout.actions")} onKeyDown={onMenuKey}>
          <li role="none">
            <button type="button" role="menuitem" onClick={() => open("saveAs", t("layout.copyName", { name: activeLabel }))}>
              {t("layout.saveAs")}
            </button>
          </li>
          <li role="none">
            <button type="button" role="menuitem" disabled={active.builtin} onClick={() => open("rename", active.name)}>
              {t("layout.rename")}
            </button>
          </li>
          <li role="none">
            <button
              type="button"
              role="menuitem"
              onClick={() => {
                store.duplicate(active.id, t("layout.copyName", { name: activeLabel }));
                setMenuOpen(false);
              }}
            >
              {t("layout.duplicate")}
            </button>
          </li>
          <li role="none">
            <button type="button" role="menuitem" disabled={active.builtin} onClick={() => open("confirmDelete")}>
              {t("layout.delete")}
            </button>
          </li>
          <li role="none">
            <button
              type="button"
              role="menuitem"
              disabled={!active.builtin}
              onClick={() => {
                store.resetBuiltin(active.id);
                setMenuOpen(false);
              }}
            >
              {t("layout.reset")}
            </button>
          </li>
        </ul>
      )}
      {(mode === "saveAs" || mode === "rename") && (
        <form className="layout-inline" onSubmit={submitName}>
          <label>
            {t("layout.name")}
            <input autoFocus value={name} onChange={(e) => setName(e.target.value)} />
          </label>
          <button type="submit" disabled={name.trim() === ""}>
            {t("layout.save")}
          </button>
          <button type="button" onClick={close}>
            {t("layout.cancel")}
          </button>
        </form>
      )}
      {mode === "confirmDelete" && (
        <div className="layout-inline" role="group" aria-label={t("layout.delete")}>
          <span>{t("layout.confirmDelete", { name: activeLabel })}</span>
          <button
            type="button"
            onClick={() => {
              store.remove(active.id);
              close();
            }}
          >
            {t("layout.deleteNamed", { name: activeLabel })}
          </button>
          <button type="button" autoFocus onClick={close}>
            {t("layout.cancel")}
          </button>
        </div>
      )}
    </div>
  );
}
