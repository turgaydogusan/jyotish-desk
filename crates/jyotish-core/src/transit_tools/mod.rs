//! Geçiş araçları (yol haritası Plan 26; §4.7–§4.9): nakshatra lordları tablosu, Sankranti
//! listesi ve efemeris tablosu. Hepsi var olan konum hesabının yeni görünümüdür: yeni astronomi
//! yoktur. Sonuçlar `Chart`'a eklenmez ve saklanmaz; panel açıkken istenir (K1).

pub mod lords;

pub use lords::NAKSHATRA_LORDS;
