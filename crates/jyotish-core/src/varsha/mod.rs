//! Varshaphala (yol haritası §4.14, Plan 27): Tajika yıllık haritası. Yıl, Güneş'in doğumdaki
//! sidereal boylamına döndüğü anda başlar (`solar_return`); o anın haritası yıllık haritadır.
//! Muntha, doğum lagnasının burcundan yılda bir burç ilerleyen noktadır (`muntha`); mudda dasa,
//! Vimshottari paylarının bir yıla sığdırılmasıdır (`mudda`).
//!
//! Kaynak: Tajika geleneğinin yıllık harita bölümleri; kurallar özgün olarak uygulanır, alıntı
//! yapılmaz. Okuma seçimleri ve sahibin onayını bekleyen varyant noktaları
//! `tests/golden/modules/varsha/REVIEW.md`'dedir. Dönüş her zodyak kipinde sidereal boylamdandır;
//! muntha rasi çerçevesindedir (K5). Yorum ve hüküm yoktur: modül yalnızca anı, haritayı, burcu ve
//! dönemleri döner. Adlar `jyotish_core` kökünden yeniden dışa aktarılmaz.

mod collect;
mod mudda;
mod muntha;
mod solar_return;

pub use collect::{compute_varsha, VarshaReport, VarshaYear};
pub use mudda::{
    mudda_dasha, mudda_days, mudda_first_lord, mudda_remainder, MuddaDasha, MUDDA_LEVELS,
    MUDDA_OFFSET, MUDDA_ORDER, MUDDA_TOTAL_DAYS,
};
pub use muntha::{muntha_sign, year_facts, Muntha, YearFacts};
pub use solar_return::{
    completed_years_at, solar_return_jd, varsha_span, RETURN_BRACKET_DAYS, RETURN_RESOLUTION_DAYS,
    SIDEREAL_YEAR_DAYS,
};

/// `VarshaReport::method`: bu modüldeki okumaların kimliği.
pub const VARSHA_METHOD: &str = "varsha.sidereal_return_v1";

/// Yıllık harita istenebilen en büyük yaş (tamamlanan yıl): Vimshottari'nin 120 yılı.
pub const MAX_VARSHA_AGE: u8 = 120;
