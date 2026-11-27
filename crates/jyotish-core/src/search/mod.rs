//! Arama (ileri modüller spec'i §12, Alt proje 3.9).
//!
//! - `query`: harita aramasının sorgusu ve denetimi.
//! - `eval`: bir haritanın arama olguları (hafif yol, `search_facts`) ve sorgunun bu olgulara
//!   uygulanması (`evaluate`).
//!
//! Hepsi olgudur: yorum, puan ya da sıralama yoktur. Sonuç saklanmaz (K1); kalıcı olgu dizini
//! kurulmaz.

#[cfg(test)]
mod crosscheck;
pub mod eval;
pub mod query;

pub use eval::{evaluate, search_facts, MatchFact, Outcome, SearchFacts};
pub use query::{
    date_noon_jd, ChartQuery, Condition, GrahaFlag, QueryCondition, QueryScope, ScopeKind,
    MAX_CONDITIONS, MAX_SCOPE_TAGS, MAX_SCOPE_TAG_CHARS, MAX_SEARCH_DASHA_LEVEL, MAX_YOGA_IDS,
    SEARCH_METHOD,
};
