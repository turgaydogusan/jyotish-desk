# jyotish-desk (kod adı)

Profesyonel astrologlar için açık kaynak Jyotish (Vedik astroloji) masaüstü yazılımı.
Lisans: AGPL-3.0-or-later. Ayrıntılar: `docs/superpowers/specs/`.

## Geliştirme

    cargo test --workspace

Vendor edilmiş Swiss Ephemeris kaynakları ve veri dosyaları depoda commit'lidir;
`./scripts/vendor-swisseph.sh` yalnızca Swiss Ephemeris'i yükseltmek için gerekir.

### Atlas

`data/atlas/atlas.sqlite` GeoNames `cities1000` dökümünden üretilir ve commit'lidir.
Yeniden üretmek için (ağ gerekir):

```bash
./scripts/fetch-geonames.sh
```

### Uygulama

    cd app
    pnpm install
    pnpm tauri dev      # geliştirme penceresi
    pnpm test           # arayüz testleri

Rust komut imzaları değişince TS bağlamalarını yeniden üretin:

    UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings

`app/ui/bindings.ts` elle düzenlenmez.
