# Jyotish Desk dışa aktarma biçimleri

Bu belge `export_clients_json` ve `export_clients_csv` komutlarının yazdığı dosyaları ve
`import_clients_json`'ın neyi kabul ettiğini anlatır. Kod: `crates/store/src/exchange.rs`,
`crates/store/src/csv.rs`, `app/src-tauri/src/store_commands.rs`.

## JSON (kayıpsız yedek, içe aktarılabilir)

```json
{
  "format": "jyotish-desk",
  "schema_version": 1,
  "exported_at": "2026-11-25T10:00:00.000Z",
  "clients": [
    {
      "uuid": "…", "name": "…", "gender": "Female" | null,
      "created_at": "…", "updated_at": "…",
      "charts": [
        {
          "uuid": "…", "label": "", "kind": "Natal" | "Event",
          "birth": { "date": "2000-01-01", "time": "17:30:00" | null, "time_certainty": "Exact",
                     "place": { "name": "…", "country": "…", "admin1": "…", "lat": 28.6139, "lon": 77.209, "geoname_id": 1261481 | null },
                     "tz": { "Iana": "Asia/Kolkata" } | { "Fixed": 19800 } | "Lmt",
                     "fold": "Earlier" | "Later" | null, "notes": "" },
          "resolved": { "utc": "2000-01-01T12:00:00Z", "jd_ut": 2451545.0, "offset_secs": 19800, "abbrev": "IST", "warnings": [], "tzdb_version": "2026f" },
          "calc_settings": { … } | null,
          "view_state": { "dasha_expanded_path": [] } | null,
          "tags": ["…"],
          "created_at": "…", "updated_at": "…"
        }
      ],
      "notes": [ { "uuid": "…", "chart_uuid": "…" | null, "body": "…", "created_at": "…", "updated_at": "…" } ] | null
    }
  ]
}
```

- **Notlar** yalnızca "Notları dahil et" işaretliyse yazılır; değilse `notes` `null`'dır (boş dizi değil).
- **`calc_settings` ve `view_state`** saklanan JSON'un aynısıdır: daha yeni bir sürümün alanları korunur.
  Saklanan JSON okunamıyorsa alan `null` yazılır. İçe aktarmada `null` ya da okunamayan
  `calc_settings` o anki genel varsayılanla değiştirilir ve bir `CalcSettingsReset` uyarısı sayılır;
  `null` `view_state` boş dasha yoluyla açılır. Düzenler (`panel_layout`) kullanıcınındır, aktarılmaz.
- **Bilinmeyen alanlar** içe aktarmada yok sayılır (reddedilmez); `format` `"jyotish-desk"` değilse ya da
  `schema_version` bu sürümden büyükse dosya reddedilir (`import_format`, `import_version`).
- **Birleştirme** UUID iledir: var olan danışan, harita ya da not atlanır ve `skipped` sayılır; üzerine yazılmaz.
  İçe aktarma tek işlemdir: bir kayıt hatalıysa hiçbir şey eklenmez (`import_record`).
- **Etiketler** virgül içeremez; virgüllü etiket içe aktarmada bölünür.
- **Kodlama** UTF-8'dir; baştaki BOM kabul edilir. UTF-8 olmayan dosya `import_format` (`utf8`) ile reddedilir.

## CSV (tek yönlü, tablo programları için)

Harita başına bir satır, RFC 4180: virgül ayırıcı, CRLF satır sonu, gerekirse çift tırnak, UTF-8 BOM ile
başlar. Notlar hiçbir zaman CSV'ye yazılmaz. Sütunlar (sırasıyla):

`client_uuid, client_name, gender, chart_uuid, label, kind, date, time, time_certainty, place_name,
country, admin1, lat, lon, tz_kind, tz_id, offset_override, fold, resolved_utc, resolved_offset_secs,
tzdb_version, tags, birth_notes, calc_settings_json`

- `tz_kind` `Iana`, `Fixed` ya da `Lmt`; `tz_id` yalnızca `Iana`'da, `offset_override` (saniye) yalnızca `Fixed`'de doludur.
- `tags` etiketleri `"; "` ile birleştirir (virgül CSV ayırıcısıdır; etiket virgül içeremez).
- `=`, `+`, `-`, `@`, sekme ya da CR ile başlayan metin alanlarının başına `'` eklenir (formül enjeksiyonu).
- CSV içe aktarılamaz; yedek için JSON kullanılır.

## Sınırlar

| Sınır | Değer | Nerede |
|---|---|---|
| İçe aktarılan JSON | 32 MiB (`MAX_IMPORT_BYTES`); dosya okunmadan önce boyutuyla denetlenir | `import_too_large` |
| Not gövdesi | 65 536 bayt UTF-8 (`MAX_NOTE_BYTES`); arayüz 16 384 karakter | `invalid_input` (`body`) |
| Etiket | `MAX_TAG_CHARS` karakter | `invalid_input` (`tags`) |
| İçe aktarma uyarıları | ilk 100 öğe listelenir, `warnings_total` hepsini sayar | `ImportReport` |
| Dışa aktarma | sınır yok; dosya 32 MiB'ı aşarsa uyarı gösterilir (bu uygulamaya olduğu gibi geri yüklenemez) | `ExportResult.over_import_limit` |

Hata bildirimlerinde yalnızca dosya adı bulunur; tam yol ve dosya içeriği gösterilmez ve loglanmaz.
