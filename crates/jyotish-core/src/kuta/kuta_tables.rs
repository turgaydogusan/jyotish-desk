//! Ashtakuta'nın sabit verisi (ileri modüller spec'i §9): sınıf eşlemeleri (burç → varna ve
//! vashya; nakshatra → yoni, gana, nadi) ve üç puan tablosu (vashya 5 × 5, yoni 14 × 14,
//! gana 3 × 3). Tabloların ilk dizini damat rolünün, ikinci dizini gelin rolünün sınıfıdır.
//!
//! Kaynak: Kuzey Hint geleneğinin 36 puanlık sistemi; muhurta ve evlilik uyumu derlemelerinde
//! basılan tablolar. Değerler özgün olarak aktarılmıştır, alıntı yapılmaz.
//!
//! Aktarım notu (spec §9 "iki bağımsız basılı kaynak"): üç tablo, yaygın basılı biçimden
//! (damat sütunda, gelin satırda verilen düzen) bu dosyadaki `[damat][gelin]` dizinine
//! çevrilerek yazılmıştır. Bilinen ayrışmalar ve seçim:
//! - **Vashya:** yaygın tablo bakışımsızdır (yabanıl sınıf gelin rolündeyse diğer dört sınıfla
//!   0; damat rolündeyse dört ayaklıyla ½, suyla 1). Bazı derlemeler tabloyu bakışımlı verir
//!   (dört ayaklı–yabanıl iki yönde ½, su–yabanıl iki yönde 1). Seçim: bakışımsız, yaygın biçim.
//! - **Gana:** damat Rakshasa – gelin Deva hücresi yaygın tabloda 1, bazı derlemelerde 0'dır;
//!   damat Deva – gelin Rakshasa hücresi yaygın tabloda 0, bazı derlemelerde 1'dir. Seçim:
//!   yaygın biçim (1 ve 0). Manushya–Rakshasa her kaynakta iki yönde 0'dır.
//! - **Yoni:** yedi düşman çift ve köşegen her kaynakta aynıdır; ara değerlerde (1, 2, 3)
//!   derlemeler arasında tek tük hücre farkı görülür. Seçim: bakışımlı, yaygın tablo.
//!
//! Tabloların basılı iki kaynakla hücre hücre karşılaştırılması bir insanın işidir ve
//! `tests/golden/modules/kuta/REVIEW.md` içindeki kayıt tablosuna yazılır.

use crate::zodiac::Sign;

/// Burç sınıfı (varna); sıra üstünlük sırasıdır: Brahmin en üstte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Varna {
    Brahmin,
    Kshatriya,
    Vaishya,
    Shudra,
}

impl Varna {
    pub const ALL: [Varna; 4] = [
        Varna::Brahmin,
        Varna::Kshatriya,
        Varna::Vaishya,
        Varna::Shudra,
    ];

    /// Sınıf kimliği (`KutaScore` alanları ve i18n anahtarı).
    pub fn id(self) -> &'static str {
        match self {
            Varna::Brahmin => "brahmin",
            Varna::Kshatriya => "kshatriya",
            Varna::Vaishya => "vaishya",
            Varna::Shudra => "shudra",
        }
    }

    /// Üstünlük derecesi: Brahmin 3 … Shudra 0.
    pub fn rank(self) -> u8 {
        3 - self as u8
    }
}

/// Vashya grubu; sıra puan tablosunun dizinidir.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Vashya {
    Quadruped,
    Human,
    Water,
    Wild,
    Insect,
}

impl Vashya {
    pub const ALL: [Vashya; 5] = [
        Vashya::Quadruped,
        Vashya::Human,
        Vashya::Water,
        Vashya::Wild,
        Vashya::Insect,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Vashya::Quadruped => "quadruped",
            Vashya::Human => "human",
            Vashya::Water => "water",
            Vashya::Wild => "wild",
            Vashya::Insect => "insect",
        }
    }
}

/// Nakshatranın hayvan simgesi (yoni); sıra puan tablosunun dizinidir.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Yoni {
    Horse,
    Elephant,
    Sheep,
    Serpent,
    Dog,
    Cat,
    Rat,
    Cow,
    Buffalo,
    Tiger,
    Deer,
    Monkey,
    Mongoose,
    Lion,
}

impl Yoni {
    pub const ALL: [Yoni; 14] = [
        Yoni::Horse,
        Yoni::Elephant,
        Yoni::Sheep,
        Yoni::Serpent,
        Yoni::Dog,
        Yoni::Cat,
        Yoni::Rat,
        Yoni::Cow,
        Yoni::Buffalo,
        Yoni::Tiger,
        Yoni::Deer,
        Yoni::Monkey,
        Yoni::Mongoose,
        Yoni::Lion,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Yoni::Horse => "horse",
            Yoni::Elephant => "elephant",
            Yoni::Sheep => "sheep",
            Yoni::Serpent => "serpent",
            Yoni::Dog => "dog",
            Yoni::Cat => "cat",
            Yoni::Rat => "rat",
            Yoni::Cow => "cow",
            Yoni::Buffalo => "buffalo",
            Yoni::Tiger => "tiger",
            Yoni::Deer => "deer",
            Yoni::Monkey => "monkey",
            Yoni::Mongoose => "mongoose",
            Yoni::Lion => "lion",
        }
    }
}

/// Nakshatranın ganası; sıra puan tablosunun dizinidir.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gana {
    Deva,
    Manushya,
    Rakshasa,
}

impl Gana {
    pub const ALL: [Gana; 3] = [Gana::Deva, Gana::Manushya, Gana::Rakshasa];

    pub fn id(self) -> &'static str {
        match self {
            Gana::Deva => "deva",
            Gana::Manushya => "manushya",
            Gana::Rakshasa => "rakshasa",
        }
    }
}

/// Nakshatranın nadisi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nadi {
    Adi,
    Madhya,
    Antya,
}

impl Nadi {
    pub const ALL: [Nadi; 3] = [Nadi::Adi, Nadi::Madhya, Nadi::Antya];

    pub fn id(self) -> &'static str {
        match self {
            Nadi::Adi => "adi",
            Nadi::Madhya => "madhya",
            Nadi::Antya => "antya",
        }
    }
}

/// Burcun varnası: su burçları Brahmin, ateş burçları Kshatriya, toprak burçları Vaishya,
/// hava burçları Shudra. Koç'tan başlayarak dörtlü döngü: Kshatriya, Vaishya, Shudra, Brahmin.
pub fn varna_of(sign: Sign) -> Varna {
    match sign.index() % 4 {
        0 => Varna::Kshatriya,
        1 => Varna::Vaishya,
        2 => Varna::Shudra,
        _ => Varna::Brahmin,
    }
}

/// Yay ve Oğlak'ın vashya grubunun değiştiği derece: ilk yarı `[0°, 15°)`.
pub const VASHYA_HALF_DEG: f64 = 15.0;

/// Ay'ın vashya grubu: burç ve (Yay ile Oğlak'ta) burç içi derecenin yarısı. Yay'ın ilk
/// yarısı insan, ikinci yarısı dört ayaklı; Oğlak'ın ilk yarısı dört ayaklı, ikinci yarısı su.
pub fn vashya_of(sign: Sign, deg_in_sign: f64) -> Vashya {
    let first_half = deg_in_sign < VASHYA_HALF_DEG;
    match sign {
        Sign::Aries | Sign::Taurus => Vashya::Quadruped,
        Sign::Gemini | Sign::Virgo | Sign::Libra | Sign::Aquarius => Vashya::Human,
        Sign::Cancer | Sign::Pisces => Vashya::Water,
        Sign::Leo => Vashya::Wild,
        Sign::Scorpio => Vashya::Insect,
        Sign::Sagittarius if first_half => Vashya::Human,
        Sign::Sagittarius => Vashya::Quadruped,
        Sign::Capricorn if first_half => Vashya::Quadruped,
        Sign::Capricorn => Vashya::Water,
    }
}

/// Nakshatra → yoni, Ashwini'den Revati'ye.
pub const NAKSHATRA_YONI: [Yoni; 27] = [
    Yoni::Horse,    // Ashwini
    Yoni::Elephant, // Bharani
    Yoni::Sheep,    // Krittika
    Yoni::Serpent,  // Rohini
    Yoni::Serpent,  // Mrigashira
    Yoni::Dog,      // Ardra
    Yoni::Cat,      // Punarvasu
    Yoni::Sheep,    // Pushya
    Yoni::Cat,      // Ashlesha
    Yoni::Rat,      // Magha
    Yoni::Rat,      // Purva Phalguni
    Yoni::Cow,      // Uttara Phalguni
    Yoni::Buffalo,  // Hasta
    Yoni::Tiger,    // Chitra
    Yoni::Buffalo,  // Swati
    Yoni::Tiger,    // Vishakha
    Yoni::Deer,     // Anuradha
    Yoni::Deer,     // Jyeshtha
    Yoni::Dog,      // Mula
    Yoni::Monkey,   // Purva Ashadha
    Yoni::Mongoose, // Uttara Ashadha
    Yoni::Monkey,   // Shravana
    Yoni::Lion,     // Dhanishta
    Yoni::Horse,    // Shatabhisha
    Yoni::Lion,     // Purva Bhadrapada
    Yoni::Cow,      // Uttara Bhadrapada
    Yoni::Elephant, // Revati
];

/// Nakshatra → gana, Ashwini'den Revati'ye.
pub const NAKSHATRA_GANA: [Gana; 27] = [
    Gana::Deva,     // Ashwini
    Gana::Manushya, // Bharani
    Gana::Rakshasa, // Krittika
    Gana::Manushya, // Rohini
    Gana::Deva,     // Mrigashira
    Gana::Manushya, // Ardra
    Gana::Deva,     // Punarvasu
    Gana::Deva,     // Pushya
    Gana::Rakshasa, // Ashlesha
    Gana::Rakshasa, // Magha
    Gana::Manushya, // Purva Phalguni
    Gana::Manushya, // Uttara Phalguni
    Gana::Deva,     // Hasta
    Gana::Rakshasa, // Chitra
    Gana::Deva,     // Swati
    Gana::Rakshasa, // Vishakha
    Gana::Deva,     // Anuradha
    Gana::Rakshasa, // Jyeshtha
    Gana::Rakshasa, // Mula
    Gana::Manushya, // Purva Ashadha
    Gana::Manushya, // Uttara Ashadha
    Gana::Deva,     // Shravana
    Gana::Rakshasa, // Dhanishta
    Gana::Rakshasa, // Shatabhisha
    Gana::Manushya, // Purva Bhadrapada
    Gana::Manushya, // Uttara Bhadrapada
    Gana::Deva,     // Revati
];

/// Nadinin nakshatra sırası üzerindeki altılı döngüsü (Ashwini = 0).
pub const NADI_CYCLE: [Nadi; 6] = [
    Nadi::Adi,
    Nadi::Madhya,
    Nadi::Antya,
    Nadi::Antya,
    Nadi::Madhya,
    Nadi::Adi,
];

/// Nakshatranın (1..=27) dizini; aralık dışı değer programcı hatasıdır.
fn nakshatra_index(nakshatra: u8) -> usize {
    assert!(
        (1..=27).contains(&nakshatra),
        "nakshatra {nakshatra} is outside 1..=27"
    );
    usize::from(nakshatra - 1)
}

/// Nakshatranın (1 = Ashwini … 27 = Revati) yonisi.
pub fn yoni_of(nakshatra: u8) -> Yoni {
    NAKSHATRA_YONI[nakshatra_index(nakshatra)]
}

/// Nakshatranın (1..=27) ganası.
pub fn gana_of(nakshatra: u8) -> Gana {
    NAKSHATRA_GANA[nakshatra_index(nakshatra)]
}

/// Nakshatranın (1..=27) nadisi: sıra mod 6 üzerinden `NADI_CYCLE`.
pub fn nadi_of(nakshatra: u8) -> Nadi {
    NADI_CYCLE[nakshatra_index(nakshatra) % 6]
}

/// Yedi düşman yoni çifti; puanları 0'dır.
pub const YONI_ENEMIES: [(Yoni, Yoni); 7] = [
    (Yoni::Horse, Yoni::Buffalo),
    (Yoni::Elephant, Yoni::Lion),
    (Yoni::Sheep, Yoni::Monkey),
    (Yoni::Serpent, Yoni::Mongoose),
    (Yoni::Dog, Yoni::Deer),
    (Yoni::Cat, Yoni::Rat),
    (Yoni::Cow, Yoni::Tiger),
];

/// Vashya puanı, `[damat][gelin]`, `Vashya::ALL` sırasında: {0, ½, 1, 2}.
pub const VASHYA_POINTS: [[f64; 5]; 5] = [
    // gelin: dört ayaklı, insan, su, yabanıl, böcek
    [2.0, 1.0, 1.0, 0.0, 1.0], // damat dört ayaklı
    [1.0, 2.0, 0.5, 0.0, 1.0], // damat insan
    [1.0, 0.5, 2.0, 0.0, 1.0], // damat su
    [0.5, 0.0, 1.0, 2.0, 0.0], // damat yabanıl
    [1.0, 1.0, 1.0, 0.0, 2.0], // damat böcek
];

/// Yoni puanı, `Yoni::ALL` sırasında (bakışımlı): aynı yoni 4, düşman çift 0, diğerleri 1–3.
pub const YONI_POINTS: [[u8; 14]; 14] = [
    // at fil koy yıl köp ked sıç ine man kap gey may fir asl
    [4, 2, 2, 3, 2, 2, 2, 1, 0, 1, 3, 3, 2, 1], // at
    [2, 4, 3, 3, 2, 2, 2, 2, 3, 1, 2, 3, 2, 0], // fil
    [2, 3, 4, 2, 1, 2, 1, 3, 3, 1, 2, 0, 3, 1], // koyun
    [3, 3, 2, 4, 2, 1, 1, 1, 1, 2, 2, 2, 0, 2], // yılan
    [2, 2, 1, 2, 4, 2, 1, 2, 2, 1, 0, 2, 1, 1], // köpek
    [2, 2, 2, 1, 2, 4, 0, 2, 2, 1, 3, 3, 2, 1], // kedi
    [2, 2, 1, 1, 1, 0, 4, 2, 2, 2, 2, 2, 1, 2], // sıçan
    [1, 2, 3, 1, 2, 2, 2, 4, 3, 0, 3, 2, 2, 1], // inek
    [0, 3, 3, 1, 2, 2, 2, 3, 4, 1, 2, 2, 2, 1], // manda
    [1, 1, 1, 2, 1, 1, 2, 0, 1, 4, 1, 1, 2, 1], // kaplan
    [3, 2, 2, 2, 0, 3, 2, 3, 2, 1, 4, 2, 2, 1], // geyik
    [3, 3, 0, 2, 2, 3, 2, 2, 2, 1, 2, 4, 3, 2], // maymun
    [2, 2, 3, 0, 1, 2, 1, 2, 2, 2, 2, 3, 4, 2], // firavun faresi
    [1, 0, 1, 2, 1, 1, 2, 1, 1, 1, 1, 2, 2, 4], // aslan
];

/// Gana puanı, `[damat][gelin]`, `Gana::ALL` sırasında.
pub const GANA_POINTS: [[u8; 3]; 3] = [
    // gelin: Deva, Manushya, Rakshasa
    [6, 6, 0], // damat Deva
    [5, 6, 0], // damat Manushya
    [1, 0, 6], // damat Rakshasa
];

/// Vashya puanı: damat ve gelin rolünün grupları.
pub fn vashya_points(groom: Vashya, bride: Vashya) -> f64 {
    VASHYA_POINTS[groom as usize][bride as usize]
}

/// Yoni puanı (rol değişiminde değişmez).
pub fn yoni_points(groom: Yoni, bride: Yoni) -> u8 {
    YONI_POINTS[groom as usize][bride as usize]
}

/// Gana puanı: damat ve gelin rolünün ganaları.
pub fn gana_points(groom: Gana, bride: Gana) -> u8 {
    GANA_POINTS[groom as usize][bride as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use Sign::*;

    /// Nakshatra adları, 1..=27 sırasında (yalnızca testlerin el listeleri için).
    const NAMES: [&str; 27] = [
        "Ashwini",
        "Bharani",
        "Krittika",
        "Rohini",
        "Mrigashira",
        "Ardra",
        "Punarvasu",
        "Pushya",
        "Ashlesha",
        "Magha",
        "Purva Phalguni",
        "Uttara Phalguni",
        "Hasta",
        "Chitra",
        "Swati",
        "Vishakha",
        "Anuradha",
        "Jyeshtha",
        "Mula",
        "Purva Ashadha",
        "Uttara Ashadha",
        "Shravana",
        "Dhanishta",
        "Shatabhisha",
        "Purva Bhadrapada",
        "Uttara Bhadrapada",
        "Revati",
    ];

    fn no(name: &str) -> u8 {
        let at = NAMES
            .iter()
            .position(|n| *n == name)
            .unwrap_or_else(|| panic!("unknown nakshatra {name}"));
        at as u8 + 1
    }

    // Bağımsız ikinci yazım (§13 katman 3): spec §9'un ad listeleri. Motor dizin sırasıyla
    // yazılmış tablolardan, bu listeler sınıf sırasıyla adlardan okur.
    const HAND_YONI: [(Yoni, &[&str]); 14] = [
        (Yoni::Horse, &["Ashwini", "Shatabhisha"]),
        (Yoni::Elephant, &["Bharani", "Revati"]),
        (Yoni::Sheep, &["Krittika", "Pushya"]),
        (Yoni::Serpent, &["Rohini", "Mrigashira"]),
        (Yoni::Dog, &["Ardra", "Mula"]),
        (Yoni::Cat, &["Punarvasu", "Ashlesha"]),
        (Yoni::Rat, &["Magha", "Purva Phalguni"]),
        (Yoni::Cow, &["Uttara Phalguni", "Uttara Bhadrapada"]),
        (Yoni::Buffalo, &["Hasta", "Swati"]),
        (Yoni::Tiger, &["Chitra", "Vishakha"]),
        (Yoni::Deer, &["Anuradha", "Jyeshtha"]),
        (Yoni::Monkey, &["Purva Ashadha", "Shravana"]),
        (Yoni::Mongoose, &["Uttara Ashadha"]),
        (Yoni::Lion, &["Dhanishta", "Purva Bhadrapada"]),
    ];

    const HAND_DEVA: [&str; 9] = [
        "Ashwini",
        "Mrigashira",
        "Punarvasu",
        "Pushya",
        "Hasta",
        "Swati",
        "Anuradha",
        "Shravana",
        "Revati",
    ];
    const HAND_MANUSHYA: [&str; 9] = [
        "Bharani",
        "Rohini",
        "Ardra",
        "Purva Phalguni",
        "Uttara Phalguni",
        "Purva Ashadha",
        "Uttara Ashadha",
        "Purva Bhadrapada",
        "Uttara Bhadrapada",
    ];

    /// Nadi, nakshatra sırasıyla elle yazılmış (A = Adi, M = Madhya, N = Antya).
    const HAND_NADI: &str = "AMNNMAAMNNMAAMNNMAAMNNMAAMN";

    #[test]
    fn every_nakshatra_has_the_yoni_of_the_hand_list() {
        let mut seen = [false; 27];
        for (yoni, names) in HAND_YONI {
            for name in names {
                assert_eq!(yoni_of(no(name)), yoni, "{name}");
                assert!(!seen[usize::from(no(name)) - 1], "{name} listed twice");
                seen[usize::from(no(name)) - 1] = true;
            }
        }
        assert!(seen.iter().all(|s| *s), "all 27 nakshatras are listed");
    }

    #[test]
    fn every_nakshatra_has_the_gana_of_the_hand_list() {
        let mut counts = [0; 3];
        for (i, name) in NAMES.iter().enumerate() {
            let want = if HAND_DEVA.contains(name) {
                Gana::Deva
            } else if HAND_MANUSHYA.contains(name) {
                Gana::Manushya
            } else {
                Gana::Rakshasa
            };
            assert_eq!(gana_of(i as u8 + 1), want, "{name}");
            counts[want as usize] += 1;
        }
        assert_eq!(counts, [9, 9, 9]);
        assert!(HAND_DEVA.iter().all(|n| !HAND_MANUSHYA.contains(n)));
    }

    #[test]
    fn every_nakshatra_has_the_nadi_of_the_hand_list() {
        assert_eq!(HAND_NADI.len(), 27);
        let mut counts = [0; 3];
        for (i, letter) in HAND_NADI.chars().enumerate() {
            let want = match letter {
                'A' => Nadi::Adi,
                'M' => Nadi::Madhya,
                _ => Nadi::Antya,
            };
            assert_eq!(nadi_of(i as u8 + 1), want, "{}", NAMES[i]);
            counts[want as usize] += 1;
        }
        assert_eq!(counts, [9, 9, 9]);
    }

    #[test]
    fn varna_follows_the_sign_lists() {
        for (varna, signs) in [
            (Varna::Brahmin, [Cancer, Scorpio, Pisces]),
            (Varna::Kshatriya, [Aries, Leo, Sagittarius]),
            (Varna::Vaishya, [Taurus, Virgo, Capricorn]),
            (Varna::Shudra, [Gemini, Libra, Aquarius]),
        ] {
            for sign in signs {
                assert_eq!(varna_of(sign), varna, "{sign:?}");
            }
        }
        // Üstünlük sırası: Brahmin > Kshatriya > Vaishya > Shudra.
        assert_eq!(Varna::ALL.map(Varna::rank), [3, 2, 1, 0]);
    }

    #[test]
    fn vashya_follows_the_sign_lists_and_the_two_split_signs() {
        // Bölünmeyen on burç derecenin her yerinde aynı gruptadır.
        for (vashya, signs) in [
            (Vashya::Quadruped, vec![Aries, Taurus]),
            (Vashya::Human, vec![Gemini, Virgo, Libra, Aquarius]),
            (Vashya::Water, vec![Cancer, Pisces]),
            (Vashya::Wild, vec![Leo]),
            (Vashya::Insect, vec![Scorpio]),
        ] {
            for sign in signs {
                for deg in [0.0, 14.999, 15.0, 29.999] {
                    assert_eq!(vashya_of(sign, deg), vashya, "{sign:?} {deg}");
                }
            }
        }
        // Yay: ilk yarı insan, ikinci yarı dört ayaklı. Oğlak: ilk yarı dört ayaklı, ikinci
        // yarı su. Sınır 15°'dir ve ikinci yarıya aittir.
        for (sign, deg, vashya) in [
            (Sagittarius, 0.0, Vashya::Human),
            (Sagittarius, 14.999_999, Vashya::Human),
            (Sagittarius, 15.0, Vashya::Quadruped),
            (Sagittarius, 29.999, Vashya::Quadruped),
            (Capricorn, 0.0, Vashya::Quadruped),
            (Capricorn, 14.999_999, Vashya::Quadruped),
            (Capricorn, 15.0, Vashya::Water),
            (Capricorn, 29.999, Vashya::Water),
        ] {
            assert_eq!(vashya_of(sign, deg), vashya, "{sign:?} {deg}");
        }
    }

    #[test]
    fn vashya_table_keeps_its_invariants() {
        for g in Vashya::ALL {
            for b in Vashya::ALL {
                let p = vashya_points(g, b);
                assert!([0.0, 0.5, 1.0, 2.0].contains(&p), "{g:?} {b:?}: {p}");
                // Köşegen en yüksek puandır ve yalnızca köşegendedir.
                assert_eq!(p == 2.0, g == b, "{g:?} {b:?}");
            }
        }
        // Yabanıl sınıf gelin rolündeyse diğer dört sınıfla 0; damat rolündeyse dört
        // ayaklıyla ½, suyla 1 (dosya başındaki aktarım notu).
        for g in [
            Vashya::Quadruped,
            Vashya::Human,
            Vashya::Water,
            Vashya::Insect,
        ] {
            assert_eq!(vashya_points(g, Vashya::Wild), 0.0, "{g:?}");
        }
        assert_eq!(vashya_points(Vashya::Wild, Vashya::Quadruped), 0.5);
        assert_eq!(vashya_points(Vashya::Wild, Vashya::Water), 1.0);
        // Yabanıl dışındaki dört sınıf arasında tablo bakışımlıdır.
        for g in Vashya::ALL {
            for b in Vashya::ALL {
                if g != Vashya::Wild && b != Vashya::Wild {
                    assert_eq!(vashya_points(g, b), vashya_points(b, g), "{g:?} {b:?}");
                }
            }
        }
    }

    // Yoni tablosunun ikinci yazımı: puan sınıfına göre elle listelenmiş çiftler (15 çift 3,
    // 26 çift 1). Listede olmayan ve düşman olmayan 43 farklı çift 2'dir (91 çift = 7 + 15 +
    // 26 + 43).
    const HAND_YONI_THREE: [(Yoni, Yoni); 15] = [
        (Yoni::Horse, Yoni::Serpent),
        (Yoni::Horse, Yoni::Deer),
        (Yoni::Horse, Yoni::Monkey),
        (Yoni::Elephant, Yoni::Sheep),
        (Yoni::Elephant, Yoni::Serpent),
        (Yoni::Elephant, Yoni::Buffalo),
        (Yoni::Elephant, Yoni::Monkey),
        (Yoni::Sheep, Yoni::Cow),
        (Yoni::Sheep, Yoni::Buffalo),
        (Yoni::Sheep, Yoni::Mongoose),
        (Yoni::Cat, Yoni::Deer),
        (Yoni::Cat, Yoni::Monkey),
        (Yoni::Cow, Yoni::Buffalo),
        (Yoni::Cow, Yoni::Deer),
        (Yoni::Monkey, Yoni::Mongoose),
    ];
    const HAND_YONI_ONE: [(Yoni, Yoni); 26] = [
        (Yoni::Horse, Yoni::Cow),
        (Yoni::Horse, Yoni::Tiger),
        (Yoni::Horse, Yoni::Lion),
        (Yoni::Elephant, Yoni::Tiger),
        (Yoni::Sheep, Yoni::Dog),
        (Yoni::Sheep, Yoni::Rat),
        (Yoni::Sheep, Yoni::Tiger),
        (Yoni::Sheep, Yoni::Lion),
        (Yoni::Serpent, Yoni::Cat),
        (Yoni::Serpent, Yoni::Rat),
        (Yoni::Serpent, Yoni::Cow),
        (Yoni::Serpent, Yoni::Buffalo),
        (Yoni::Dog, Yoni::Rat),
        (Yoni::Dog, Yoni::Tiger),
        (Yoni::Dog, Yoni::Mongoose),
        (Yoni::Dog, Yoni::Lion),
        (Yoni::Cat, Yoni::Tiger),
        (Yoni::Cat, Yoni::Lion),
        (Yoni::Rat, Yoni::Mongoose),
        (Yoni::Cow, Yoni::Lion),
        (Yoni::Buffalo, Yoni::Tiger),
        (Yoni::Buffalo, Yoni::Lion),
        (Yoni::Tiger, Yoni::Deer),
        (Yoni::Tiger, Yoni::Monkey),
        (Yoni::Tiger, Yoni::Lion),
        (Yoni::Deer, Yoni::Lion),
    ];

    fn hand_yoni_points(a: Yoni, b: Yoni) -> u8 {
        let listed = |pairs: &[(Yoni, Yoni)]| {
            pairs
                .iter()
                .any(|&(x, y)| (x, y) == (a, b) || (x, y) == (b, a))
        };
        if a == b {
            4
        } else if listed(&YONI_ENEMIES) {
            0
        } else if listed(&HAND_YONI_THREE) {
            3
        } else if listed(&HAND_YONI_ONE) {
            1
        } else {
            2
        }
    }

    #[test]
    fn yoni_table_keeps_its_invariants_and_matches_the_hand_lists() {
        let mut zeros = 0;
        for a in Yoni::ALL {
            for b in Yoni::ALL {
                let p = yoni_points(a, b);
                assert_eq!(p, yoni_points(b, a), "symmetry {a:?} {b:?}");
                assert_eq!(p, hand_yoni_points(a, b), "{a:?} {b:?}");
                if a == b {
                    assert_eq!(p, 4);
                } else if p == 0 {
                    zeros += 1;
                } else {
                    assert!((1..=3).contains(&p), "{a:?} {b:?}: {p}");
                }
            }
        }
        // Yedi düşman çift, iki yönde: 14 sıfır hücre; başka sıfır yok.
        assert_eq!(zeros, 14);
        let twos = Yoni::ALL
            .into_iter()
            .flat_map(|a| Yoni::ALL.map(|b| (a, b)))
            .filter(|&(a, b)| (a as usize) < (b as usize) && yoni_points(a, b) == 2)
            .count();
        assert_eq!(twos, 43);
        for (a, b) in YONI_ENEMIES {
            assert_eq!(yoni_points(a, b), 0, "{a:?} {b:?}");
        }
        // Her yoni tam bir düşman çiftte yer alır.
        for y in Yoni::ALL {
            let n = YONI_ENEMIES
                .iter()
                .filter(|(a, b)| *a == y || *b == y)
                .count();
            assert_eq!(n, 1, "{y:?}");
        }
    }

    #[test]
    fn gana_table_keeps_its_invariants() {
        for g in Gana::ALL {
            for b in Gana::ALL {
                let p = gana_points(g, b);
                assert!(p <= 6, "{g:?} {b:?}");
                // Köşegen en yüksek puandır.
                if g == b {
                    assert_eq!(p, 6);
                }
            }
        }
        // Manushya–Rakshasa iki yönde de 0.
        assert_eq!(gana_points(Gana::Manushya, Gana::Rakshasa), 0);
        assert_eq!(gana_points(Gana::Rakshasa, Gana::Manushya), 0);
        // Dokuz hücrenin tamamı, damat rolü önce (dosya başındaki aktarım notu).
        let cells: Vec<u8> = Gana::ALL
            .into_iter()
            .flat_map(|g| Gana::ALL.map(|b| gana_points(g, b)))
            .collect();
        assert_eq!(cells, [6, 6, 0, 5, 6, 0, 1, 0, 6]);
    }

    #[test]
    fn class_ids_are_unique_within_each_kuta() {
        fn unique(ids: &[&str]) {
            for (i, id) in ids.iter().enumerate() {
                assert!(!ids[..i].contains(id), "{id} twice");
                assert!(id.chars().all(|c| c.is_ascii_lowercase()), "{id}");
            }
        }
        unique(&Varna::ALL.map(Varna::id));
        unique(&Vashya::ALL.map(Vashya::id));
        unique(&Yoni::ALL.map(Yoni::id));
        unique(&Gana::ALL.map(Gana::id));
        unique(&Nadi::ALL.map(Nadi::id));
    }

    #[test]
    #[should_panic(expected = "outside 1..=27")]
    fn a_nakshatra_outside_the_range_is_a_programming_error() {
        yoni_of(28);
    }
}
