//! Geçiş araçlarının komutları (yol haritası Plan 26): Sankranti listesi, efemeris tablosu ve
//! geçiş anının adımı.
//! Kalıp muhurta komutlarıyla aynıdır: hesap motor iş parçacığında, komut gövdesi `blocking`
//! havuzunda, hata `CommandError`; sonuç saklanmaz (K1). "Bu yıl", "bu ay", önceki ve sonraki
//! ay Rust'ta okunur; arayüz tarih hesabı yapmaz (C4).
//!
//! Yer kişisel veridir: loglanmaz ve hata parametresine girmez. Bu modülün ürettiği hatalar
//! var olan `invalid_transit_input` kodunu ve yalnızca alan adını taşır.

pub mod ephemeris;
pub mod sankranti;
pub mod step;

use crate::commands::CommandError;

/// Yıl girdisinin aralığı (yerin takvim yılı). Hesap aralığı (`JD_UT_RANGE`) daha geniştir;
/// sınır, tarihin dört basamakla yazılabildiği yıllardır.
pub const TOOLS_YEAR_RANGE: std::ops::RangeInclusive<i32> = 1..=2999;

pub(crate) fn invalid(field: &str) -> CommandError {
    CommandError::new("invalid_transit_input", &[("field", field.to_owned())])
}
