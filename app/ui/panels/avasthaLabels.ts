// Avastha adları: Sanskritçe terimlerdir ve dile göre değişmez (K11). Kimlikler Rust
// enum'larıdır; bir test bu tabloları fikstürdeki `avastha_ids` ile karşılaştırır.
import type { Baladi, Jagradadi, Lajjitadi, Shayanadi } from "../bindings";

export const BALADI_NAMES: Record<Baladi, string> = {
  Bala: "Bala",
  Kumara: "Kumara",
  Yuva: "Yuva",
  Vriddha: "Vriddha",
  Mrita: "Mrita",
};

export const JAGRADADI_NAMES: Record<Jagradadi, string> = {
  Jagrat: "Jagrat",
  Swapna: "Swapna",
  Sushupti: "Sushupti",
};

export const LAJJITADI_NAMES: Record<Lajjitadi, string> = {
  Lajjita: "Lajjita",
  Garvita: "Garvita",
  Kshudhita: "Kshudhita",
  Trushita: "Trushita",
  Mudita: "Mudita",
  Kshobhita: "Kshobhita",
};

export const SHAYANADI_NAMES: Record<Shayanadi, string> = {
  Shayana: "Shayana",
  Upaveshana: "Upaveshana",
  Netrapani: "Netrapani",
  Prakashana: "Prakashana",
  Gamana: "Gamana",
  Agamana: "Agamana",
  Sabha: "Sabha",
  Agama: "Agama",
  Bhojana: "Bhojana",
  NrityaLipsa: "Nritya Lipsa",
  Kautuka: "Kautuka",
  Nidra: "Nidra",
};
