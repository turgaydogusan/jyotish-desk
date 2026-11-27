//! Arama (ileri modüller spec'i §12, Alt proje 3.9).
//!
//! - `query`: harita aramasının sorgusu ve denetimi.
//! - `eval`: bir haritanın arama olguları (hafif yol, `search_facts`) ve sorgunun bu olgulara
//!   uygulanması (`evaluate`).
//! - `events`: zaman içinde gök olayı araması (`event_search`), `timeline` çekirdeğiyle.
//!
//! Hepsi olgudur: yorum, puan ya da sıralama yoktur. Sonuç saklanmaz (K1); kalıcı olgu dizini
//! kurulmaz.

#[cfg(test)]
mod crosscheck;
pub mod eval;
pub mod events;
pub mod query;

pub use eval::{evaluate, search_facts, MatchFact, Outcome, SearchFacts};
pub use events::{
    event_chunks, event_search, event_search_chunk, EventSpec, SkyEvent, SkyEventKind,
    EVENT_CHUNK_DAYS, EVENT_MAX_RANGE_DAYS, EVENT_METHOD, EVENT_MOON_CHUNK_DAYS,
    EVENT_MOON_MAX_RANGE_DAYS, HOUSE_CHANGE_GRAHAS, MAX_SKY_EVENTS,
};
pub use query::{
    date_noon_jd, ChartQuery, Condition, GrahaFlag, QueryCondition, QueryScope, ScopeKind,
    MAX_CONDITIONS, MAX_SCOPE_TAGS, MAX_SCOPE_TAG_CHARS, MAX_SEARCH_DASHA_LEVEL, MAX_YOGA_IDS,
    SEARCH_METHOD,
};
