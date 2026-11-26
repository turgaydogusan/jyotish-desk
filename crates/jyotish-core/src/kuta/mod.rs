//! Uyum (ileri modüller spec'i §9): Ashtakuta (Kuzey Hint geleneğinin 36 puanlık sistemi).
//! Bu dosya modülün köküdür; sabit veri `kuta_tables`'tadır.

pub mod kuta_tables;

pub use kuta_tables::{
    gana_of, gana_points, nadi_of, varna_of, vashya_of, vashya_points, yoni_of, yoni_points, Gana,
    Nadi, Varna, Vashya, Yoni, GANA_POINTS, NADI_CYCLE, NAKSHATRA_GANA, NAKSHATRA_YONI,
    VASHYA_HALF_DEG, VASHYA_POINTS, YONI_ENEMIES, YONI_POINTS,
};
