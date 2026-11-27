// Harita aramasının bir koşul satırı (ileri modüller spec'i §12.1): tür seçici, türe göre
// parametre alanları (görünür etiketli), "değil" kutusu ve satırı sil. Alanlar yalnızca seçimi
// toplar; koşulu Rust değerlendirir (C4). Retro, Rahu ve Ketu için sunulmaz.
import { useId, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import type { Condition, Graha, QueryCondition, Sign, Varga } from "../bindings";
import { signIndex } from "../chart/adapter";
import { tr } from "../i18n/tr";
import { arudhaLabel, KARAKA_ROLES, nakshatraName, tithiName, tithiShort } from "../panels/labels";
import { CheckSet } from "./CheckSet";
import {
  ASPECTING,
  CONDITION_KINDS,
  conditionIssue,
  defaultCondition,
  DIGNITIES,
  GRAHA_FLAGS,
  GRAHA_IDS,
  HOUSES,
  KARAKA_ROLE_IDS,
  kindOf,
  MAX_DASHA_LEVEL,
  NAKSHATRAS,
  PADAS,
  PAKSHAS,
  SIGNS,
  TITHIS,
  toggle,
  VARGAS,
  yogaIds,
  type ConditionKind,
} from "./model";

const YOGA_IDS = yogaIds(tr.yogas.name);
const LEVELS = Array.from({ length: MAX_DASHA_LEVEL }, (_, i) => i + 1);

export interface ConditionRowProps {
  index: number;
  value: QueryCondition;
  onChange(value: QueryCondition): void;
  onRemove(): void;
}

export function ConditionRow({ index, value, onChange, onRemove }: ConditionRowProps) {
  const { t } = useTranslation();
  const id = useId();
  const { cond } = value;
  const kind = kindOf(cond);
  const issue = conditionIssue(cond);
  const set = (next: Condition) => onChange({ ...value, cond: next });
  const count = (n: number) => t(n === 0 ? "search.noneSelected" : "search.selected", { n });

  const select = <T extends string | number>(key: string, label: string, current: T, options: readonly { value: T; label: string }[], pick: (v: T) => void): ReactNode => (
    <span className="search-field">
      <label htmlFor={`${id}-${key}`}>{label}</label>
      <select
        id={`${id}-${key}`}
        value={current}
        onChange={(e) => {
          const found = options.find((o) => String(o.value) === e.target.value);
          if (found) pick(found.value);
        }}
      >
        {options.map((o) => (
          <option key={o.value} value={o.value}>
            {o.label}
          </option>
        ))}
      </select>
    </span>
  );
  const grahaOptions = (list: readonly Graha[]) => list.map((g) => ({ value: g, label: t(`grahaName.${g}`) }));
  const grahaSelect = (key: string, label: string, current: Graha, pick: (g: Graha) => void, list: readonly Graha[] = GRAHA_IDS) => select(key, label, current, grahaOptions(list), pick);
  const vargaSelect = (current: Varga, pick: (v: Varga) => void) =>
    select(
      "varga",
      t("search.field.varga"),
      current,
      VARGAS.map((v) => ({ value: v, label: v })),
      pick,
    );
  const houseSelect = (key: string, label: string, current: number, pick: (h: number) => void, name: (h: number) => string = String) =>
    select(
      key,
      label,
      current,
      HOUSES.map((h) => ({ value: h, label: name(h) })),
      pick,
    );
  const signSet = (selected: readonly Sign[], pick: (signs: Sign[]) => void) => (
    <CheckSet
      legend={t("search.field.signs")}
      fold={`${t("search.field.signs")} · ${count(selected.length)}`}
      options={SIGNS.map((s) => ({ value: s, label: t(`sign.${signIndex(s)}`) }))}
      selected={selected}
      onToggle={(s) => pick(toggle(selected, s, SIGNS))}
    />
  );
  const houseSet = (selected: readonly number[], pick: (houses: number[]) => void) => (
    <CheckSet
      legend={t("search.field.houses")}
      fold={`${t("search.field.houses")} · ${count(selected.length)}`}
      options={HOUSES.map((h) => ({ value: h, label: String(h) }))}
      selected={selected}
      onToggle={(h) => pick(toggle(selected, h, HOUSES))}
    />
  );
  const grahaSet = (legend: string, selected: readonly Graha[], pick: (grahas: Graha[]) => void) => (
    <CheckSet
      legend={legend}
      fold={`${legend} · ${count(selected.length)}`}
      options={grahaOptions(GRAHA_IDS)}
      selected={selected}
      onToggle={(g) => pick(toggle(selected, g, GRAHA_IDS))}
    />
  );

  let fields: ReactNode = null;
  if (cond.GrahaInSign) {
    const c = cond.GrahaInSign;
    fields = (
      <>
        {grahaSelect("graha", t("search.field.graha"), c.graha, (graha) => set({ GrahaInSign: { ...c, graha } }))}
        {vargaSelect(c.varga, (varga) => set({ GrahaInSign: { ...c, varga } }))}
        {signSet(c.signs, (signs) => set({ GrahaInSign: { ...c, signs } }))}
      </>
    );
  } else if (cond.GrahaInHouse) {
    const c = cond.GrahaInHouse;
    fields = (
      <>
        {grahaSelect("graha", t("search.field.graha"), c.graha, (graha) => set({ GrahaInHouse: { ...c, graha } }))}
        {houseSet(c.houses, (houses) => set({ GrahaInHouse: { ...c, houses } }))}
      </>
    );
  } else if (cond.GrahaInNakshatra) {
    const c = cond.GrahaInNakshatra;
    fields = (
      <>
        {grahaSelect("graha", t("search.field.graha"), c.graha, (graha) => set({ GrahaInNakshatra: { ...c, graha } }))}
        <CheckSet
          legend={t("search.field.nakshatras")}
          fold={`${t("search.field.nakshatras")} · ${count(c.nakshatras.length)}`}
          options={NAKSHATRAS.map((n) => ({ value: n, label: nakshatraName(n) }))}
          selected={c.nakshatras}
          onToggle={(n) => set({ GrahaInNakshatra: { ...c, nakshatras: toggle(c.nakshatras, n, NAKSHATRAS) } })}
        />
        <CheckSet
          legend={t("search.field.padas")}
          options={PADAS.map((p) => ({ value: p, label: String(p) }))}
          selected={c.padas}
          onToggle={(p) => set({ GrahaInNakshatra: { ...c, padas: toggle(c.padas, p, PADAS) } })}
        />
      </>
    );
  } else if (cond.GrahaState) {
    const c = cond.GrahaState;
    // Düğümlerde retro sunulmaz; graha düğüme çevrilince işaretli retro kalkar.
    const flags = GRAHA_FLAGS.filter((f) => f !== "Retrograde" || (c.graha !== "Rahu" && c.graha !== "Ketu"));
    fields = (
      <>
        {grahaSelect("graha", t("search.field.graha"), c.graha, (graha) =>
          set({ GrahaState: { ...c, graha, flags: graha === "Rahu" || graha === "Ketu" ? c.flags.filter((f) => f !== "Retrograde") : c.flags } }),
        )}
        <CheckSet
          legend={t("search.field.flags")}
          options={flags.map((f) => ({ value: f, label: f === "Retrograde" ? "R" : t(`search.flag.${f}`) }))}
          selected={c.flags}
          onToggle={(f) => set({ GrahaState: { ...c, flags: toggle(c.flags, f, GRAHA_FLAGS) } })}
        />
        <CheckSet
          legend={t("search.field.dignities")}
          fold={`${t("search.field.dignities")} · ${count(c.dignities.length)}`}
          options={DIGNITIES.map((d) => ({ value: d, label: t(`dignity.${d}`) }))}
          selected={c.dignities}
          onToggle={(d) => set({ GrahaState: { ...c, dignities: toggle(c.dignities, d, DIGNITIES) } })}
        />
      </>
    );
  } else if (cond.LagnaInSign) {
    const c = cond.LagnaInSign;
    fields = (
      <>
        {vargaSelect(c.varga, (varga) => set({ LagnaInSign: { ...c, varga } }))}
        {signSet(c.signs, (signs) => set({ LagnaInSign: { ...c, signs } }))}
      </>
    );
  } else if (cond.Conjunction) {
    const c = cond.Conjunction;
    fields = grahaSet(t("search.field.grahas"), c.grahas, (grahas) => set({ Conjunction: { grahas } }));
  } else if (cond.GrahaAspects) {
    const c = cond.GrahaAspects;
    fields = (
      <>
        {grahaSelect("from", t("search.field.from"), c.from, (from) => set({ GrahaAspects: { ...c, from } }), ASPECTING)}
        {grahaSelect("to", t("search.field.to"), c.to, (to) => set({ GrahaAspects: { ...c, to } }))}
      </>
    );
  } else if (cond.HouseLordInHouse) {
    const c = cond.HouseLordInHouse;
    fields = (
      <>
        {houseSelect("lord", t("search.field.lordOf"), c.lord_of, (lord_of) => set({ HouseLordInHouse: { ...c, lord_of } }))}
        {houseSet(c.houses, (houses) => set({ HouseLordInHouse: { ...c, houses } }))}
      </>
    );
  } else if (cond.MoonPhase) {
    const c = cond.MoonPhase;
    fields = (
      <>
        {select(
          "paksha",
          t("search.field.paksha"),
          c.paksha ?? "",
          [{ value: "" as const, label: t("search.field.pakshaAny") }, ...PAKSHAS.map((p) => ({ value: p, label: p }))],
          (paksha) => set({ MoonPhase: { ...c, paksha: paksha === "" ? null : paksha } }),
        )}
        <CheckSet
          legend={t("search.field.tithis")}
          fold={`${t("search.field.tithis")} · ${count(c.tithis.length)}`}
          options={TITHIS.map((n) => ({ value: n, label: `${tithiShort(n)} ${tithiName(n)}` }))}
          selected={c.tithis}
          onToggle={(n) => set({ MoonPhase: { ...c, tithis: toggle(c.tithis, n, TITHIS) } })}
        />
      </>
    );
  } else if (cond.DashaAt) {
    const c = cond.DashaAt;
    fields = (
      <>
        <span className="search-field">
          <label htmlFor={`${id}-date`}>{t("search.field.date")}</label>
          <input id={`${id}-date`} type="date" value={c.date ?? ""} aria-describedby={`${id}-date-hint`} onChange={(e) => set({ DashaAt: { ...c, date: e.target.value === "" ? null : e.target.value } })} />
          <span id={`${id}-date-hint`} className="chart-toolbar-hint">
            {t("search.field.dateHint")}
          </span>
        </span>
        {select(
          "level",
          t("search.field.level"),
          c.level,
          LEVELS.map((l) => ({ value: l, label: t(`search.level.${l}`) })),
          (level) => set({ DashaAt: { ...c, level } }),
        )}
        {grahaSet(t("search.field.lords"), c.lords, (lords) => set({ DashaAt: { ...c, lords } }))}
      </>
    );
  } else if (cond.CharaKaraka) {
    const c = cond.CharaKaraka;
    fields = (
      <>
        {select(
          "role",
          t("search.field.role"),
          c.role,
          KARAKA_ROLE_IDS.map((r) => ({ value: r, label: `${KARAKA_ROLES[r].short} · ${KARAKA_ROLES[r].name}` })),
          (role) => set({ CharaKaraka: { ...c, role } }),
        )}
        {grahaSet(t("search.field.grahas"), c.grahas, (grahas) => set({ CharaKaraka: { ...c, grahas } }))}
      </>
    );
  } else if (cond.ArudhaInSign) {
    const c = cond.ArudhaInSign;
    fields = (
      <>
        {houseSelect("pada", t("search.field.pada"), c.pada, (pada) => set({ ArudhaInSign: { ...c, pada } }), arudhaLabel)}
        {signSet(c.signs, (signs) => set({ ArudhaInSign: { ...c, signs } }))}
      </>
    );
  } else if (cond.YogaPresent) {
    const c = cond.YogaPresent;
    fields = (
      <CheckSet
        legend={t("search.field.yogas")}
        fold={`${t("search.field.yogas")} · ${count(c.ids.length)}`}
        options={YOGA_IDS.map((y) => ({ value: y, label: t(`yogas.name.${y}`) }))}
        selected={c.ids}
        onToggle={(y) => set({ YogaPresent: { ids: toggle(c.ids, y, YOGA_IDS) } })}
      />
    );
  }

  return (
    <fieldset className="search-condition" data-condition={index} data-kind={kind}>
      <legend>{t("search.conditionN", { n: index + 1 })}</legend>
      <div className="search-condition-head">
        {select(
          "kind",
          t("search.kindLabel"),
          kind,
          CONDITION_KINDS.map((k) => ({ value: k, label: t(`search.kind.${k}`) })),
          (k: ConditionKind) => onChange({ ...value, cond: defaultCondition(k) }),
        )}
        <label>
          <input type="checkbox" checked={value.negate} onChange={(e) => onChange({ ...value, negate: e.target.checked })} />
          {t("search.negate")}
        </label>
        <button type="button" onClick={onRemove}>
          {t("search.remove")}
        </button>
      </div>
      <div className="search-condition-fields">{fields}</div>
      {issue !== null && (
        <p className="chart-toolbar-hint" data-issue={issue}>
          {t(`search.issue.${issue}`)}
        </p>
      )}
    </fieldset>
  );
}
