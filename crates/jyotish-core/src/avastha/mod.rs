//! Avasthalar (yol haritası §4.4, Plan 24): grahanın "durum" sınıfları. Baladi (burç içi
//! dereceden beş yaş basamağı), Jagradadi (burçtan üç uyanıklık durumu), Lajjitadi (yerleşim,
//! birliktelik ve bakıştan altı durum; sonuç kümedir) ve Shayanadi (nakshatra, graha sayısı,
//! navamsa, doğum nakshatrası, ghati ve lagnadan on iki durum).
//!
//! Kaynak: BPHS, avasthalar bölümü; kurallar özgün olarak uygulanır, alıntı yapılmaz. Okuma
//! seçimleri ve sahibin onayını bekleyen varyant noktaları
//! `tests/golden/modules/avastha/REVIEW.md`'dedir. Burçlar ve evler rasi çerçevesindedir
//! (K4, K5); nakshatra sidereal boylamdandır. Rahu ve Ketu'ya avastha atanmaz. Yorum ve hüküm
//! yoktur: modül yalnızca sınıfı ve kuralı sağlayan grahaları döner.

mod baladi;

pub use baladi::{baladi, jagradadi, Baladi, Jagradadi, BALADI_SPAN_DEG};

use crate::drishti::ASPECTING_GRAHAS;
use crate::graha::Graha;

/// `AvasthaReport::method`: bu modüldeki okumaların kimliği.
pub const AVASTHA_METHOD: &str = "avastha.bphs_v1";

/// Avastha atanan grahalar: rapor satırlarının sırası.
pub const AVASTHA_GRAHAS: [Graha; 7] = ASPECTING_GRAHAS;
