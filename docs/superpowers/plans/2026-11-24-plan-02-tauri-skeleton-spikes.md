# Plan 02 — Tauri 2 İskeleti, Ephemeris İş Parçacığı ve Print Spike'ı

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `ephemeris` motorunu kendi iş parçacığında çalıştıran, tauri-specta ile tip güvenli komutlar sunan, React/TS arayüzlü çalışır bir Tauri 2 masaüstü iskeleti ve WKWebView print → PDF spike'ı kurmak.

**Architecture:** `SwissEph` `!Send` olduğu için `ephemeris` crate'ine bir `Engine` aktörü eklenir. Bu aktör SE örneğine tek bir iş parçacığında sahip olur, işleri bir kanal (channel) üzerinden alır ve sonucu çağırana geri gönderir. `Engine` `Send + Sync` olduğu için Tauri `State` olarak paylaşılır. `app/src-tauri` ince bir komut katmanıdır. TS bağlamaları (`app/ui/bindings.ts`) tauri-specta ile üretilir; güncel olup olmadıkları bir Rust testiyle doğrulanır. Arayüz yalnızca çizer, hesap yapmaz.

**Tech Stack:** Rust 1.90+, Tauri 2.12 (`tauri` 2.12.x, `tauri-build` 2.7.x), `specta` =2.0.0-rc.28, `tauri-specta` =2.0.0-rc.29, `specta-typescript` 0.0.15, React 19, Vite 8, TypeScript 7, Vitest 5, pnpm 11.

**Spec:** `docs/superpowers/specs/2026-11-24-jyotish-desk-v1-design.md` (§4 Mimari, §7 Hata yönetimi, §8 CI, §11 M0)

## Global Constraints

- Lisans: AGPL-3.0-or-later. Bu lisans tüm crate'lerin `license` alanında ve `app/package.json` dosyasında yer alır.
- Kod adı `jyotish-desk`. "Kala", "Kāla", "Kaala", "Cala" ve türevleri hiçbir yerde kullanılmaz. Geçici ürün adı `Jyotish Desk`, tanımlayıcı `io.github.turgaydogusan.jyotishdesk`.
- SE global durumu: süreçte tek bir `SwissEph` örneği bulunur ve tek bir iş parçacığı (`Engine`) ona sahip olur.
- Veri yolu parametre olarak verilir. Uygulamada yol, Tauri resource dizininden (`ephe/`) çözülür; sabit kodlanmış yol yok.
- Eksik ephemeris dosyası ölümcül hatadır; sessizce yedeğe düşülmez.
- FFI sınırında ve iş parçacığı sınırında panik yayılmaz.
- `src-tauri` ince bir katmandır; iş mantığı içermez. Arayüz hesap yapmaz.
- Hatalar arayüze `{code, params}` biçiminde gider; çeviri i18n katmanında yapılır (i18n Plan 07'de gelir).
- MSRV: workspace `rust-version = "1.90"` (Tauri 2.12'nin tabanı). MSRV CI işi `cargo +1.90 check -p ephemeris --all-targets` komutunu çalıştırır.
- CI: macOS (`macos-15`) tam seti çalıştırır: fmt, clippy `-D warnings`, test, vitest, build. `ubuntu` ve `windows` yalnızca çekirdek crate'leri (`-p ephemeris`) kapsar. Bir de `cargo deny` işi var.
- Minimum macOS: 13 (Ventura). `bundle.macOS.minimumSystemVersion = "13.0"`.
- Tauri spike'larında elle yapılan cihaz testi planın sonuna bırakılır; otomatik testler ilgili görevlerde yazılır.

---

## Dosya Yapısı (bu plan)

```
Cargo.toml                         # members += "app/src-tauri"; rust-version 1.90
deny.toml                          # Tauri grafiği için lisans izinleri
.github/workflows/ci.yml           # pnpm/node adımları; ubuntu/windows -p ephemeris
crates/ephemeris/
  src/error.rs                     # + AlreadyOpen, EngineStopped, JobPanicked
  src/swiss.rs                     # + SwissEph::try_open
  src/engine.rs                    # Engine aktörü (yeni)
  src/lib.rs                       # pub use engine::Engine
  tests/engine.rs                  # Engine testleri (yeni)
app/
  package.json, pnpm-lock.yaml, tsconfig.json, vite.config.ts, index.html
  app-icon.svg                     # icon kaynağı (geçici)
  ui/main.tsx, ui/App.tsx, ui/App.test.tsx, ui/setupTests.ts
  ui/bindings.ts                   # tauri-specta çıktısı (commit edilir, elle düzenlenmez)
  ui/PrintSpike.tsx, ui/print.css, ui/PrintSpike.test.tsx
  src-tauri/
    Cargo.toml, build.rs, tauri.conf.json
    capabilities/default.json
    icons/                         # `pnpm tauri icon` çıktısı
    src/main.rs                    # jyotish_desk_lib::run()
    src/lib.rs                     # specta_builder(), run()
    src/commands.rs                # komutlar + DTO'lar + CommandError
    tests/bindings.rs              # bindings.ts güncel mi
docs/spikes/2026-11-24-wkwebview-print.md
```

---

### Task 1: `Engine` aktörü ve `SwissEph::try_open`

**Files:**
- Modify: `crates/ephemeris/src/error.rs`
- Modify: `crates/ephemeris/src/swiss.rs` (`open` gövdesini paylaşılan bir yardımcıya taşı)
- Create: `crates/ephemeris/src/engine.rs`
- Modify: `crates/ephemeris/src/lib.rs`
- Test: `crates/ephemeris/tests/engine.rs`

**Interfaces:**
- Consumes: `SwissEph::open(&Path) -> Result<SwissEph, EphError>`, `Ephemeris` trait, `Body`, `Zodiac`, `julday_gregorian`.
- Produces:
  - `EphError::AlreadyOpen`, `EphError::EngineStopped`, `EphError::JobPanicked` (yeni varyantlar)
  - `SwissEph::try_open(path: &Path) -> Result<SwissEph, EphError>`: kilit tutuluyorsa beklemez, `AlreadyOpen` döner.
  - `pub struct Engine` (`Send + Sync`):
    - `Engine::spawn(path: PathBuf) -> Result<Engine, EphError>`: iş parçacığını başlatır, `try_open` sonucunu bekler ve aynen döndürür.
    - `Engine::call<R: Send + 'static>(&self, f: impl FnOnce(&mut SwissEph) -> R + Send + 'static) -> Result<R, EphError>`
    - `Drop`: kanalı kapatır, iş parçacığını `join` eder ve kilidi bırakır.

- [ ] **Step 1: Hata varyantlarını ekle**

`crates/ephemeris/src/error.rs`, `Calc` varyantından sonra:

```rust
    #[error("a SwissEph instance is already open in this process")]
    AlreadyOpen,
    #[error("ephemeris engine thread has stopped")]
    EngineStopped,
    #[error("an ephemeris engine job panicked")]
    JobPanicked,
```

- [ ] **Step 2: Başarısız testi yaz**

`crates/ephemeris/tests/engine.rs`:

```rust
mod common;

use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use ephemeris::{julday_gregorian, Body, EphError, Engine, Ephemeris, SwissEph, Zodiac};

// SE kilidi süreç çapında; bu dosyadaki testler paralel koşarsa birbirinin
// `try_open`'ını `AlreadyOpen` ile düşürür. Testleri sıraya sokar.
static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|p| p.into_inner())
}

fn j2000() -> f64 {
    julday_gregorian(2000, 1, 1, 12.0)
}

#[test]
fn engine_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Engine>();
}

#[test]
fn call_returns_sun_position() {
    let _s = serial();
    let engine = Engine::spawn(common::ephe_dir()).expect("spawn");
    let pos = engine
        .call(|se| se.body(j2000(), Body::Sun, Zodiac::Tropical))
        .expect("engine alive")
        .expect("calc ok");
    assert!((280.0..281.0).contains(&pos.lon), "sun lon {}", pos.lon);
}

#[test]
fn second_instance_while_engine_alive_is_already_open() {
    let _s = serial();
    let _engine = Engine::spawn(common::ephe_dir()).expect("spawn");
    assert!(matches!(
        Engine::spawn(common::ephe_dir()),
        Err(EphError::AlreadyOpen)
    ));
    assert!(matches!(
        SwissEph::try_open(&common::ephe_dir()),
        Err(EphError::AlreadyOpen)
    ));
}

#[test]
fn drop_releases_the_instance() {
    let _s = serial();
    let engine = Engine::spawn(common::ephe_dir()).expect("first spawn");
    drop(engine);
    let again = Engine::spawn(common::ephe_dir()).expect("respawn after drop");
    drop(again);
}

#[test]
fn missing_data_dir_fails_spawn() {
    let _s = serial();
    let bogus = PathBuf::from("/nonexistent-jyotish-desk-ephe");
    assert!(matches!(
        Engine::spawn(bogus),
        Err(EphError::DataFilesMissing { .. })
    ));
}

#[test]
fn panicking_job_reports_error_and_engine_survives() {
    let _s = serial();
    let engine = Engine::spawn(common::ephe_dir()).expect("spawn");
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let r: Result<(), EphError> = engine.call(|_| panic!("boom"));
    std::panic::set_hook(prev_hook);
    assert!(matches!(r, Err(EphError::JobPanicked)));
    let pos = engine
        .call(|se| se.body(j2000(), Body::Moon, Zodiac::Tropical))
        .expect("engine still alive")
        .expect("calc ok");
    assert!((0.0..360.0).contains(&pos.lon));
}

#[test]
fn concurrent_callers_are_serialized() {
    let _s = serial();
    let engine = Arc::new(Engine::spawn(common::ephe_dir()).expect("spawn"));
    let handles: Vec<_> = (0..4)
        .map(|i| {
            let e = Arc::clone(&engine);
            std::thread::spawn(move || {
                for k in 0..10 {
                    let jd = j2000() + f64::from(i * 10 + k);
                    let pos = e
                        .call(move |se| se.body(jd, Body::Mars, Zodiac::Tropical))
                        .expect("alive")
                        .expect("calc");
                    assert!((0.0..360.0).contains(&pos.lon));
                }
            })
        })
        .collect();
    for h in handles {
        h.join().expect("caller thread");
    }
}
```

- [ ] **Step 3: Testin derlenmediğini doğrula**

Komut: `cargo test -p ephemeris --test engine`
Beklenen: `Engine` ve `SwissEph::try_open` bulunamadığı için derleme hatası (`unresolved import ephemeris::Engine`).

- [ ] **Step 4: `try_open` ekle**

`crates/ephemeris/src/swiss.rs` içinde `open` gövdesini, kilidi parametre alan özel bir fonksiyona taşı. Kilit alma kodu ve kontroller dışında davranış değişmez:

```rust
use std::sync::TryLockError;

impl SwissEph {
    pub fn open(path: &Path) -> Result<Self, EphError> {
        let guard = SWE_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Self::open_locked(guard, path)
    }

    /// `open` gibi, ama süreçte başka bir örnek açıksa beklemez:
    /// `EphError::AlreadyOpen` döner.
    pub fn try_open(path: &Path) -> Result<Self, EphError> {
        let guard = match SWE_LOCK.try_lock() {
            Ok(g) => g,
            Err(TryLockError::Poisoned(p)) => p.into_inner(),
            Err(TryLockError::WouldBlock) => return Err(EphError::AlreadyOpen),
        };
        Self::open_locked(guard, path)
    }

    fn open_locked(guard: MutexGuard<'static, ()>, path: &Path) -> Result<Self, EphError> {
        // Eski `open` gövdesinin `let guard = ...` satırından sonraki kısmı
        // buraya taşınır (SE_EPHE_PATH, yol doğrulama, swe_set_ephe_path,
        // self-check, `SwissEph { _guard: guard, .. }`).
    }
}
```

`open` üzerindeki mevcut doc yorumu korunur. Yoruma şu satır eklenir: "Beklemek istemeyen çağıranlar `try_open` kullanır."

- [ ] **Step 5: `Engine` yaz**

`crates/ephemeris/src/engine.rs`:

```rust
//! `SwissEph` örneğine tek bir iş parçacığında sahip olan aktör.
//!
//! `SwissEph` `!Send` olduğu için örnek, kendi iş parçacığında açılır ve orada
//! yaşar. Çağıranlar kapanışları (closure) kanal üzerinden gönderir; sonuç
//! kendi kanalından geri döner. `Engine` `Send + Sync`'tir.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread::JoinHandle;

use crate::{EphError, SwissEph};

type Job = Box<dyn FnOnce(&mut SwissEph) + Send>;

pub struct Engine {
    tx: Option<mpsc::Sender<Job>>,
    thread: Option<JoinHandle<()>>,
}

impl Engine {
    /// İş parçacığını başlatır ve `SwissEph::try_open(path)` sonucunu döndürür.
    /// Süreçte başka bir örnek açıksa `EphError::AlreadyOpen` döner.
    pub fn spawn(path: PathBuf) -> Result<Engine, EphError> {
        let (tx, rx) = mpsc::channel::<Job>();
        let (ready_tx, ready_rx) = mpsc::sync_channel::<Result<(), EphError>>(1);
        let thread = std::thread::Builder::new()
            .name("swisseph".into())
            .spawn(move || {
                let mut se = match SwissEph::try_open(&path) {
                    Ok(se) => {
                        let _ = ready_tx.send(Ok(()));
                        se
                    }
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                        return;
                    }
                };
                while let Ok(job) = rx.recv() {
                    job(&mut se);
                }
            })
            .map_err(|e| EphError::Calc(format!("cannot start engine thread: {e}")))?;
        match ready_rx.recv() {
            Ok(Ok(())) => Ok(Engine {
                tx: Some(tx),
                thread: Some(thread),
            }),
            Ok(Err(e)) => {
                let _ = thread.join();
                Err(e)
            }
            Err(_) => {
                let _ = thread.join();
                Err(EphError::EngineStopped)
            }
        }
    }

    /// `f`'yi motor iş parçacığında çalıştırır ve sonucunu döndürür.
    /// `f` panik yaparsa `EphError::JobPanicked` döner; motor çalışmaya devam eder.
    pub fn call<R, F>(&self, f: F) -> Result<R, EphError>
    where
        R: Send + 'static,
        F: FnOnce(&mut SwissEph) -> R + Send + 'static,
    {
        let (rtx, rrx) = mpsc::sync_channel::<Result<R, EphError>>(1);
        let job: Job = Box::new(move |se| {
            let out = catch_unwind(AssertUnwindSafe(|| f(se))).map_err(|_| EphError::JobPanicked);
            let _ = rtx.send(out);
        });
        self.tx
            .as_ref()
            .ok_or(EphError::EngineStopped)?
            .send(job)
            .map_err(|_| EphError::EngineStopped)?;
        rrx.recv().map_err(|_| EphError::EngineStopped)?
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        // Göndericiyi düşürmek döngüyü bitirir; join, SE kilidinin bırakılmasını bekler.
        self.tx.take();
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
```

`crates/ephemeris/src/lib.rs` dosyasına `mod engine;` ve `pub use engine::Engine;` ekle.

- [ ] **Step 6: Testleri çalıştır**

Komut: `cargo test -p ephemeris`
Beklenen: Tüm testler geçer. Önceki 19 testin yanına `engine.rs`'ten 7 test eklenir; panik testi stderr'e çıktı basmaz.

Komut: `cargo clippy -p ephemeris --all-targets -- -D warnings && cargo fmt --all --check`
Beklenen: Uyarı yok.

- [ ] **Step 7: Commit**

```bash
git add crates/ephemeris
git commit -m "feat(ephemeris): Engine actor owning SwissEph on a dedicated thread; try_open"
```

---

### Task 2: Arayüz iskeleti (Vite + React + TS + Vitest)

**Files:**
- Create: `app/package.json`, `app/tsconfig.json`, `app/vite.config.ts`, `app/index.html`
- Create: `app/ui/main.tsx`, `app/ui/App.tsx`, `app/ui/App.test.tsx`, `app/ui/setupTests.ts`
- Create: `app/ui/bindings.ts` (geçici elle yazılmış stub, Task 3'te üretilen dosyayla değiştirilir)
- Create: `app/pnpm-lock.yaml` (pnpm üretir)
- Modify: `.gitignore` (`/app/dist`)

**Interfaces:**
- Produces:
  - `app/ui/bindings.ts` modülünden şu imzalarla tüketim:
    - `commands.engineStatus(): Promise<Result<EngineStatus, CommandError>>`
    - `commands.planetLongitudes(jdUt: number): Promise<Result<PlanetRow[], CommandError>>`
  - Tipler:
    - `EngineStatus = { seVersion: string }`
    - `PlanetRow = { body: string; lon: number; retrograde: boolean }`
    - `CommandError = { code: string; params: Partial<{ [key in string]: string }> }`
    - `Result<T, E> = { status: "ok"; data: T } | { status: "error"; error: E }`
  - Bu şekil tauri-specta rc.29 çıktısıyla aynıdır. Task 3 dosyayı üretilmiş haliyle değiştirir; `App.tsx` değişmeden derlenmelidir.
  - `vite build` çıktısı `app/dist` klasörüne yazılır (Tauri `frontendDist: "../dist"`).

- [ ] **Step 1: Proje dosyaları**

`app/package.json`:

```json
{
  "name": "jyotish-desk-app",
  "private": true,
  "version": "0.1.0",
  "license": "AGPL-3.0-or-later",
  "type": "module",
  "packageManager": "pnpm@11.8.0",
  "scripts": {
    "dev": "vite",
    "build": "tsc --noEmit && vite build",
    "test": "vitest run",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.12.5",
    "react": "^19.3.1",
    "react-dom": "^19.3.1"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.12.6",
    "@testing-library/jest-dom": "^8.0.0",
    "@testing-library/react": "^16.3.3",
    "@types/react": "^19.3.0",
    "@types/react-dom": "^19.3.0",
    "@vitejs/plugin-react": "^6.3.0",
    "jsdom": "^31.0.1",
    "typescript": "^7.0.2",
    "vite": "^8.3.8",
    "vitest": "^5.1.1"
  }
}
```

`app/tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "skipLibCheck": true,
    "types": ["vitest/globals", "@testing-library/jest-dom"]
  },
  "include": ["ui", "vite.config.ts"]
}
```

`app/vite.config.ts`:

```ts
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

// Tauri dev sunucusu sabit portta bekler; port doluysa hata verilir.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: { outDir: "dist", target: "safari16" },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["ui/setupTests.ts"],
    include: ["ui/**/*.test.tsx"],
  },
});
```

`app/index.html`:

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Jyotish Desk</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/ui/main.tsx"></script>
  </body>
</html>
```

`app/ui/setupTests.ts`:

```ts
import "@testing-library/jest-dom/vitest";
```

`.gitignore` dosyasına `/app/dist` satırını ekle.

- [ ] **Step 2: Bağlama stub'ı**

`app/ui/bindings.ts` (Task 3'te üretilen dosyayla değiştirilir):

```ts
// Geçici stub: Task 3'te tauri-specta çıktısıyla değiştirilir.
import { invoke } from "@tauri-apps/api/core";

export type EngineStatus = { seVersion: string };
export type PlanetRow = { body: string; lon: number; retrograde: boolean };
export type CommandError = { code: string; params: Partial<{ [key in string]: string }> };
export type Result<T, E> = { status: "ok"; data: T } | { status: "error"; error: E };

async function wrap<T>(p: Promise<T>): Promise<Result<T, CommandError>> {
  try {
    return { status: "ok", data: await p };
  } catch (e) {
    return { status: "error", error: e as CommandError };
  }
}

export const commands = {
  engineStatus: () => wrap(invoke<EngineStatus>("engine_status")),
  planetLongitudes: (jdUt: number) => wrap(invoke<PlanetRow[]>("planet_longitudes", { jdUt })),
};
```

- [ ] **Step 3: Başarısız testi yaz**

`app/ui/App.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import { vi } from "vitest";

vi.mock("./bindings", () => ({
  commands: {
    engineStatus: vi.fn(async () => ({ status: "ok", data: { seVersion: "2.10.03" } })),
    planetLongitudes: vi.fn(async () => ({
      status: "ok",
      data: [
        { body: "Sun", lon: 256.5, retrograde: false },
        { body: "Saturn", lon: 10.25, retrograde: true },
      ],
    })),
  },
}));

import { App, unixMsToJdUt } from "./App";

test("unixMsToJdUt maps the Unix epoch to JD 2440587.5", () => {
  expect(unixMsToJdUt(0)).toBe(2440587.5);
  expect(unixMsToJdUt(86_400_000)).toBe(2440588.5);
});

test("shows engine version and planet rows", async () => {
  render(<App />);
  expect(await screen.findByText(/2\.10\.03/)).toBeInTheDocument();
  expect(await screen.findByText("Sun")).toBeInTheDocument();
  expect(screen.getByText("256.5000")).toBeInTheDocument();
  expect(screen.getByText("R")).toBeInTheDocument();
});

test("shows error code when a command fails", async () => {
  const { commands } = await import("./bindings");
  vi.mocked(commands.planetLongitudes).mockResolvedValueOnce({
    status: "error",
    error: { code: "calc", params: { detail: "x" } },
  });
  render(<App />);
  expect(await screen.findByRole("alert")).toHaveTextContent("calc");
});
```

- [ ] **Step 4: Paketleri kur ve testin başarısız olduğunu doğrula**

Komut: `cd app && pnpm install && pnpm test`
Beklenen: `./App` modülü yok, testler FAIL.

- [ ] **Step 5: Uygulamayı yaz**

`app/ui/App.tsx`:

```tsx
import { useEffect, useState } from "react";
import { commands, type CommandError, type PlanetRow } from "./bindings";

/** Unix milisaniyesini UT Jülyen gününe çevirir (1970-01-01T00:00Z = JD 2440587.5). */
export function unixMsToJdUt(ms: number): number {
  return ms / 86_400_000 + 2440587.5;
}

export function App() {
  const [version, setVersion] = useState<string | null>(null);
  const [rows, setRows] = useState<PlanetRow[]>([]);
  const [error, setError] = useState<CommandError | null>(null);

  useEffect(() => {
    void (async () => {
      const status = await commands.engineStatus();
      if (status.status === "ok") setVersion(status.data.seVersion);
      else setError(status.error);
      const planets = await commands.planetLongitudes(unixMsToJdUt(Date.now()));
      if (planets.status === "ok") setRows(planets.data);
      else setError(planets.error);
    })();
  }, []);

  return (
    <main>
      <h1>Jyotish Desk</h1>
      <p>Swiss Ephemeris {version ?? "…"}</p>
      {error && <p role="alert">{error.code}</p>}
      <table>
        <thead>
          <tr>
            <th>Graha</th>
            <th>Sidereal lon (Lahiri)</th>
            <th />
          </tr>
        </thead>
        <tbody>
          {rows.map((r) => (
            <tr key={r.body}>
              <td>{r.body}</td>
              <td>{r.lon.toFixed(4)}</td>
              <td>{r.retrograde ? "R" : ""}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </main>
  );
}
```

`app/ui/main.tsx`:

```tsx
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
```

- [ ] **Step 6: Test ve build**

Komut: `cd app && pnpm test && pnpm build`
Beklenen: 3 test geçer; `app/dist/index.html` oluşur; `tsc` hata vermez.

TypeScript 7 `tsc --noEmit` komutunu desteklemiyorsa, `typescript` paketini 7.x'in en son sürümünde tut ve `pnpm exec tsc --version` çıktısını rapora yaz. Hatayı kısa haliyle alıntılayıp DONE_WITH_CONCERNS raporla; TS sürümünü kendi başına düşürme.

- [ ] **Step 7: Commit**

```bash
git add .gitignore app/package.json app/pnpm-lock.yaml app/tsconfig.json app/vite.config.ts app/index.html app/ui
git commit -m "feat(app): Vite + React + TS UI skeleton with vitest"
```

---

### Task 3: Tauri crate'i, tauri-specta komutları ve bağlama testi

**Files:**
- Modify: `Cargo.toml` (members, rust-version 1.90)
- Create: `app/src-tauri/Cargo.toml`, `app/src-tauri/build.rs`, `app/src-tauri/tauri.conf.json`, `app/src-tauri/capabilities/default.json`
- Create: `app/src-tauri/src/main.rs`, `app/src-tauri/src/lib.rs`, `app/src-tauri/src/commands.rs`
- Create: `app/src-tauri/tests/bindings.rs`
- Create: `app/app-icon.svg`, `app/src-tauri/icons/*` (`pnpm tauri icon` üretir)
- Modify: `app/ui/bindings.ts` (üretilmiş haliyle değiştirilir)
- Modify: `deny.toml`
- Modify: `.gitignore` (`/app/src-tauri/gen`)

**Interfaces:**
- Consumes:
  - `ephemeris::{Engine, EphError, Ephemeris, Body, Zodiac, SiderealMode, SwissEph}`
  - Task 2 bağlama şekli: `engineStatus`, `planetLongitudes(jdUt)`, `EngineStatus`, `PlanetRow`, `CommandError`.
- Produces:
  - `jyotish_desk_lib::specta_builder() -> tauri_specta::Builder<tauri::Wry>`
  - `jyotish_desk_lib::run()`
  - `commands::planet_rows(eph: &mut dyn Ephemeris, jd_ut: f64) -> Result<Vec<PlanetRow>, EphError>`
  - `CommandError::from(EphError)`; kodlar: `invalid_path`, `env_override`, `data_files_missing`, `calc`, `already_open`, `engine_stopped`, `job_panicked`.
  - Task 4 `specta_builder()`'a `print_window` komutunu ekler.

- [ ] **Step 1: Workspace ve crate manifesti**

Kök `Cargo.toml` dosyasında:

```toml
[workspace]
resolver = "2"
members = ["crates/ephemeris", "app/src-tauri"]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "AGPL-3.0-or-later"
rust-version = "1.90"
repository = "https://github.com/turgaydogusan/jyotish-desk"
```

(`[workspace.dependencies]` değişmez.)

`app/src-tauri/Cargo.toml`:

```toml
[package]
name = "jyotish-desk"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
repository.workspace = true

[lib]
name = "jyotish_desk_lib"

[build-dependencies]
tauri-build = { version = "2.7", features = [] }

[dependencies]
ephemeris = { path = "../../crates/ephemeris" }
tauri = { version = "2.12", features = [] }
serde = { version = "1", features = ["derive"] }
specta = { version = "=2.0.0-rc.28", features = ["derive"] }
tauri-specta = { version = "=2.0.0-rc.29", features = ["derive", "typescript"] }
specta-typescript = "0.0.15"
```

`app/src-tauri/build.rs`:

```rust
fn main() {
    tauri_build::build()
}
```

- [ ] **Step 2: Tauri yapılandırması ve ikonlar**

`app/src-tauri/tauri.conf.json`:

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Jyotish Desk",
  "version": "0.1.0",
  "identifier": "io.github.turgaydogusan.jyotishdesk",
  "build": {
    "beforeDevCommand": "pnpm dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "pnpm build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [{ "title": "Jyotish Desk", "width": 1200, "height": 800 }],
    "security": { "csp": "default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:" }
  },
  "bundle": {
    "active": true,
    "targets": ["app", "dmg"],
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/128x128@2x.png", "icons/icon.icns", "icons/icon.png"],
    "resources": { "../../data/ephe/": "ephe/" },
    "macOS": { "minimumSystemVersion": "13.0" }
  }
}
```

`app/src-tauri/capabilities/default.json`:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "windows": ["main"],
  "permissions": ["core:default"]
}
```

`app/app-icon.svg` (geçici ikon; markalama Plan 10'da):

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024">
  <rect width="1024" height="1024" rx="200" fill="#1f2a44"/>
  <rect x="212" y="212" width="600" height="600" fill="none" stroke="#e8c26a" stroke-width="40"/>
  <path d="M212 212 L812 812 M812 212 L212 812" stroke="#e8c26a" stroke-width="40"/>
</svg>
```

Komut: `cd app && pnpm tauri icon app-icon.svg`
Beklenen: `app/src-tauri/icons/` altında `32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.icns`, `icon.png` ve diğer dosyalar oluşur. iOS ve Android klasörleri oluşursa (`icons/ios`, `icons/android`) sil; v1 yalnızca macOS.

`.gitignore` dosyasına `/app/src-tauri/gen` satırını ekle.

- [ ] **Step 3: Komutlar için başarısız testleri yaz**

`app/src-tauri/src/commands.rs` dosyasının sonuna eklenecek test modülü (dosyanın geri kalanı Step 5'te yazılır):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ephemeris::{Ayanamsa, BodyPosition, Precision};

    /// Her cisme sabit boylam veren sahte motor; Rahu 100°.
    struct Fake;
    impl Ephemeris for Fake {
        fn body(&mut self, _jd: f64, body: Body, _z: Zodiac) -> Result<BodyPosition, EphError> {
            let lon = match body {
                Body::MeanNode => 100.0,
                Body::Saturn => 300.0,
                _ => 10.0,
            };
            Ok(BodyPosition {
                lon,
                lat: 0.0,
                distance_au: 1.0,
                lon_speed: if body == Body::Saturn { -0.05 } else { 1.0 },
                lat_speed: 0.0,
                dist_speed: 0.0,
                precision: Precision::SwissEph,
            })
        }
        fn ayanamsa(&mut self, _jd: f64, _m: SiderealMode) -> Result<Ayanamsa, EphError> {
            Ok(Ayanamsa { deg: 24.0, precision: Precision::SwissEph })
        }
    }

    #[test]
    fn rows_are_nine_grahas_in_order() {
        let rows = planet_rows(&mut Fake, 2451545.0).unwrap();
        let names: Vec<_> = rows.iter().map(|r| r.body.as_str()).collect();
        assert_eq!(
            names,
            ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu"]
        );
    }

    #[test]
    fn ketu_is_opposite_rahu_and_both_retrograde() {
        let rows = planet_rows(&mut Fake, 2451545.0).unwrap();
        let rahu = rows.iter().find(|r| r.body == "Rahu").unwrap();
        let ketu = rows.iter().find(|r| r.body == "Ketu").unwrap();
        assert_eq!(rahu.lon, 100.0);
        assert_eq!(ketu.lon, 280.0);
        assert!(rahu.retrograde && ketu.retrograde);
    }

    #[test]
    fn retrograde_follows_speed_sign() {
        let rows = planet_rows(&mut Fake, 2451545.0).unwrap();
        assert!(rows.iter().find(|r| r.body == "Saturn").unwrap().retrograde);
        assert!(!rows.iter().find(|r| r.body == "Sun").unwrap().retrograde);
    }

    #[test]
    fn error_codes_are_stable() {
        assert_eq!(CommandError::from(EphError::AlreadyOpen).code, "already_open");
        let e = CommandError::from(EphError::DataFilesMissing {
            path: "/p".into(),
            detail: "d".into(),
        });
        assert_eq!(e.code, "data_files_missing");
        assert_eq!(e.params.get("path").map(String::as_str), Some("/p"));
        assert_eq!(CommandError::from(EphError::Calc("x".into())).params["detail"], "x");
    }

    #[test]
    fn real_engine_produces_sidereal_rows() {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/ephe");
        let engine = ephemeris::Engine::spawn(dir).expect("spawn");
        let rows = engine
            .call(|se| planet_rows(se, 2451545.0))
            .expect("alive")
            .expect("calc");
        let sun = rows.iter().find(|r| r.body == "Sun").unwrap();
        // J2000 tropikal Güneş ≈ 280.37°, Lahiri ≈ 23.86° → sidereal ≈ 256.5°.
        assert!((256.0..257.0).contains(&sun.lon), "sun {}", sun.lon);
    }
}
```

Not: `Body` `PartialEq` türetmiyorsa, `body == Body::Saturn` yerine `matches!(body, Body::Saturn)` kullan.

- [ ] **Step 4: Testin başarısız olduğunu doğrula**

Komut: `cargo test -p jyotish-desk --lib`
Beklenen: `planet_rows` / `CommandError` tanımlı olmadığı için derleme hatası. `app/dist` yoksa önce `pnpm --dir app build` çalıştır; Tauri `frontendDist` klasörünün derleme anında var olmasını ister.

- [ ] **Step 5: Komutları yaz**

`app/src-tauri/src/commands.rs` (test modülünün üstü):

```rust
//! İnce komut katmanı: girdiyi motora iletir, sonucu DTO'ya çevirir.

use std::collections::BTreeMap;

use ephemeris::{Body, Engine, EphError, Ephemeris, SiderealMode, SwissEph, Zodiac};
use serde::Serialize;
use specta::Type;
use tauri::State;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EngineStatus {
    pub se_version: String,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PlanetRow {
    pub body: String,
    pub lon: f64,
    pub retrograde: bool,
}

/// Arayüze giden hata: `code` sabit bir anahtar, çeviri i18n katmanında yapılır.
#[derive(Debug, Clone, Serialize, Type)]
pub struct CommandError {
    pub code: String,
    pub params: BTreeMap<String, String>,
}

impl CommandError {
    pub fn new(code: &str, params: &[(&str, String)]) -> Self {
        Self {
            code: code.to_owned(),
            params: params.iter().map(|(k, v)| ((*k).to_owned(), v.clone())).collect(),
        }
    }
}

impl From<EphError> for CommandError {
    fn from(e: EphError) -> Self {
        match e {
            EphError::InvalidPath => Self::new("invalid_path", &[]),
            EphError::EnvOverride => Self::new("env_override", &[]),
            EphError::DataFilesMissing { path, detail } => {
                Self::new("data_files_missing", &[("path", path), ("detail", detail)])
            }
            EphError::Calc(detail) => Self::new("calc", &[("detail", detail)]),
            EphError::AlreadyOpen => Self::new("already_open", &[]),
            EphError::EngineStopped => Self::new("engine_stopped", &[]),
            EphError::JobPanicked => Self::new("job_panicked", &[]),
        }
    }
}

const GRAHAS: [(Body, &str); 8] = [
    (Body::Sun, "Sun"),
    (Body::Moon, "Moon"),
    (Body::Mars, "Mars"),
    (Body::Mercury, "Mercury"),
    (Body::Jupiter, "Jupiter"),
    (Body::Venus, "Venus"),
    (Body::Saturn, "Saturn"),
    (Body::MeanNode, "Rahu"),
];

/// Dokuz grahanın Lahiri sidereal boylamları. Ketu = Rahu + 180°.
/// Spike amaçlıdır; gerçek hesap Plan 04'te `jyotish-core`'a taşınır.
pub fn planet_rows(eph: &mut dyn Ephemeris, jd_ut: f64) -> Result<Vec<PlanetRow>, EphError> {
    let zodiac = Zodiac::Sidereal(SiderealMode::Lahiri);
    let mut rows = Vec::with_capacity(9);
    for (body, name) in GRAHAS {
        let p = eph.body(jd_ut, body, zodiac)?;
        rows.push(PlanetRow { body: name.to_owned(), lon: p.lon, retrograde: p.is_retrograde() });
    }
    let rahu = rows[7].clone();
    rows.push(PlanetRow {
        body: "Ketu".to_owned(),
        lon: (rahu.lon + 180.0).rem_euclid(360.0),
        retrograde: rahu.retrograde,
    });
    Ok(rows)
}

#[tauri::command]
#[specta::specta]
pub async fn engine_status() -> Result<EngineStatus, CommandError> {
    Ok(EngineStatus { se_version: SwissEph::version() })
}

/// Motor çağrısı milisaniyeler sürer. Async komut, ana iş parçacığını (UI) bloklamaz.
#[tauri::command]
#[specta::specta]
pub async fn planet_longitudes(
    engine: State<'_, Engine>,
    jd_ut: f64,
) -> Result<Vec<PlanetRow>, CommandError> {
    Ok(engine.call(move |se: &mut SwissEph| planet_rows(se, jd_ut))??)
}
```

`Zodiac` `Copy` değilse, döngü içinde `zodiac` yerine her çağrıda `Zodiac::Sidereal(SiderealMode::Lahiri)` yaz. `is_retrograde()` düğümler için `lon_speed < 0` döndürür; Mean Node hızı her zaman negatiftir.

`app/src-tauri/src/lib.rs`:

```rust
pub mod commands;

use tauri::path::BaseDirectory;
use tauri::Manager;

/// Komut listesi tek yerde: hem uygulama hem bağlama testi bunu kullanır.
pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
    tauri_specta::Builder::<tauri::Wry>::new().commands(tauri_specta::collect_commands![
        commands::engine_status,
        commands::planet_longitudes,
    ])
}

pub fn run() {
    let builder = specta_builder();
    tauri::Builder::default()
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            let ephe = app.path().resolve("ephe", BaseDirectory::Resource)?;
            // Eksik veri ölümcül: setup hatası uygulamayı açmadan sonlandırır.
            let engine = ephemeris::Engine::spawn(ephe)?;
            app.manage(engine);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Jyotish Desk");
}
```

`app/src-tauri/src/main.rs`:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    jyotish_desk_lib::run()
}
```

tauri-specta rc.29'un API'si yukarıdakinden farklıysa (`Builder::new`, `commands`, `collect_commands!`, `invoke_handler`, `mount_events`, `export`), docs.rs'teki `tauri-specta/2.0.0-rc.29` belgelerine göre en küçük değişikliği yap ve farkı raporuna yaz. Sürümleri değiştirme.

- [ ] **Step 6: Bağlama testi**

`app/src-tauri/tests/bindings.rs`:

```rust
//! `app/ui/bindings.ts` dosyasının komut tanımlarıyla aynı olduğunu doğrular.
//! Güncellemek için: `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings`

use std::path::PathBuf;

#[test]
fn bindings_ts_is_up_to_date() {
    let committed = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../ui/bindings.ts");
    let fresh = std::env::temp_dir().join(format!("jd-bindings-{}.ts", std::process::id()));
    jyotish_desk_lib::specta_builder()
        .export(specta_typescript::Typescript::default(), &fresh)
        .expect("export bindings");
    let fresh_text = std::fs::read_to_string(&fresh).expect("read fresh");
    let _ = std::fs::remove_file(&fresh);
    if std::env::var_os("UPDATE_BINDINGS").is_some() {
        std::fs::write(&committed, &fresh_text).expect("write bindings.ts");
        return;
    }
    let committed_text = std::fs::read_to_string(&committed).unwrap_or_default();
    assert!(
        committed_text == fresh_text,
        "app/ui/bindings.ts is stale; run: UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings"
    );
}
```

Komut: `cargo test -p jyotish-desk --test bindings`
Beklenen: Elle yazılmış stub farklı olduğu için FAIL ("bindings.ts is stale").

Komut: `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings && cargo test -p jyotish-desk --test bindings`
Beklenen: Dosya üretilir, ikinci çalıştırma PASS.

Ardından `cd app && pnpm test && pnpm build` çalıştır. Üretilen dosyadaki `commands.engineStatus` / `commands.planetLongitudes` imzaları ve tip adları Task 2'deki şekille aynı olmalıdır. Fark varsa (ör. `params` tipi), `App.tsx` ve `App.test.tsx` dosyalarını üretilmiş tiplere uydur; `bindings.ts` dosyasını elle düzenleme.

- [ ] **Step 7: Lisans politikası**

Komut: `cargo deny check licenses`
Tauri grafiğinin getirdiği lisanslar için `deny.toml` `allow` listesine yalnızca AGPL-3.0 ile uyumlu, izin verici lisansları ekle: `MPL-2.0`, `CC0-1.0`, `BSL-1.0`, `0BSD`, `Unicode-DFS-2016`, `CDLA-Permissive-2.0`. Bunlardan yalnızca gerçekten raporlananları ekle. Listede olmayan başka bir lisans çıkarsa (ör. GPL-2.0-only, belirsiz ya da lisanssız bir crate), ekleme: BLOCKED raporla ve crate adıyla lisansı yaz.

Komut: `cargo deny check`
Beklenen: `advisories ok, bans ok, licenses ok, sources ok`. Advisory çıkarsa crate adını ve RUSTSEC kimliğini raporla; `ignore` listesine kendi başına ekleme.

- [ ] **Step 8: Tam doğrulama**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo +1.90 check -p ephemeris --all-targets
```

Beklenen: Hepsi geçer. `rustup toolchain install 1.90` gerekebilir.

Uygulamayı bir kez başlatıp kapat: `cd app && timeout 120 pnpm tauri dev` (pencere açılır; sürüm ve 9 satır görünür). Bu adım ortamda pencere açamıyorsa atla ve rapora "dev çalıştırması yapılmadı" yaz; cihaz testi planın sonunda yapılacak.

- [ ] **Step 9: Commit**

```bash
git add Cargo.toml Cargo.lock deny.toml .gitignore app/app-icon.svg app/src-tauri app/ui
git commit -m "feat(app): Tauri 2 shell with tauri-specta commands and engine state"
```

---

### Task 4: WKWebView print → PDF spike

**Files:**
- Modify: `app/src-tauri/src/commands.rs` (`print_window`)
- Modify: `app/src-tauri/src/lib.rs` (komut listesine ekle)
- Modify: `app/ui/bindings.ts` (yeniden üret)
- Create: `app/ui/PrintSpike.tsx`, `app/ui/print.css`, `app/ui/PrintSpike.test.tsx`
- Modify: `app/ui/App.tsx` (spike görünümüne geçiş düğmesi)
- Create: `docs/spikes/2026-11-24-wkwebview-print.md`

**Interfaces:**
- Consumes: `specta_builder()`, `CommandError::new`, `PlanetRow`, `commands.planetLongitudes`, `unixMsToJdUt`.
- Produces: `commands.printWindow(): Promise<Result<null, CommandError>>`; `<PrintSpike rows={PlanetRow[]} />`.

- [ ] **Step 1: Başarısız arayüz testi**

`app/ui/PrintSpike.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import { PrintSpike } from "./PrintSpike";

const rows = [
  { body: "Sun", lon: 256.5, retrograde: false },
  { body: "Moon", lon: 10.0, retrograde: false },
];

test("renders two A4 pages with chart svg and planet table", () => {
  const { container } = render(<PrintSpike rows={rows} />);
  expect(container.querySelectorAll("section.print-page")).toHaveLength(2);
  expect(container.querySelector("svg[data-testid='spike-chart']")).not.toBeNull();
  expect(screen.getByText("Sun")).toBeInTheDocument();
  // Türkçe ve Devanagari karakterleri PDF'te gömülü fontla görünmeli.
  expect(screen.getByText(/Doğum haritası — ğüşıöç İ/)).toBeInTheDocument();
  expect(screen.getByText("राशि")).toBeInTheDocument();
});

test("planet sign index is floor(lon / 30)", () => {
  render(<PrintSpike rows={rows} />);
  // Sun 256.5° → 9. burç (Dhanu, index 8); hücrede "Sun" etiketi.
  const cell = screen.getByTestId("sign-8");
  expect(cell).toHaveTextContent("Sun");
});
```

Komut: `cd app && pnpm test`
Beklenen: `./PrintSpike` bulunamadığı için FAIL.

- [ ] **Step 2: Print düzeni**

`app/ui/print.css`:

```css
@page {
  size: A4;
  margin: 15mm;
}

.print-page {
  break-after: page;
  page-break-after: always;
}
.print-page:last-child {
  break-after: auto;
  page-break-after: auto;
}

@media print {
  html, body {
    background: #fff;
    color: #000;
    -webkit-print-color-adjust: exact;
    print-color-adjust: exact;
  }
  .no-print {
    display: none !important;
  }
}

.spike-table {
  border-collapse: collapse;
  font-size: 10pt;
}
.spike-table td,
.spike-table th {
  border: 0.5pt solid #000;
  padding: 2pt 6pt;
}
```

`app/ui/PrintSpike.tsx`:

```tsx
import type { PlanetRow } from "./bindings";
import "./print.css";

// Güney Hint düzeni: 4×4 ızgaranın dış halkası, Meena sol üstten saat yönünde.
// [satır, sütun] → burç index (0 = Mesha).
const SOUTH_CELLS: Array<[number, number, number]> = [
  [0, 0, 11], [0, 1, 0], [0, 2, 1], [0, 3, 2],
  [1, 3, 3], [2, 3, 4], [3, 3, 5], [3, 2, 6],
  [3, 1, 7], [3, 0, 8], [2, 0, 9], [1, 0, 10],
];

export function PrintSpike({ rows }: { rows: PlanetRow[] }) {
  const bySign = new Map<number, string[]>();
  for (const r of rows) {
    const s = Math.floor(r.lon / 30) % 12;
    bySign.set(s, [...(bySign.get(s) ?? []), r.body]);
  }
  const cell = 45;
  return (
    <div>
      <section className="print-page">
        <h2>Doğum haritası — ğüşıöç İ</h2>
        <p lang="hi">राशि</p>
        <svg
          data-testid="spike-chart"
          viewBox="0 0 180 180"
          width="160mm"
          height="160mm"
          xmlns="http://www.w3.org/2000/svg"
        >
          {SOUTH_CELLS.map(([r, c, sign]) => (
            <g key={sign} data-testid={`sign-${sign}`}>
              <rect x={c * cell} y={r * cell} width={cell} height={cell} fill="none" stroke="#000" strokeWidth={0.5} />
              <text x={c * cell + 3} y={r * cell + 10} fontSize={6}>
                {(bySign.get(sign) ?? []).join(" ")}
              </text>
            </g>
          ))}
        </svg>
      </section>
      <section className="print-page">
        <h2>Graha tablosu</h2>
        <table className="spike-table">
          <thead>
            <tr><th>Graha</th><th>Boylam</th><th>R</th></tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.body}>
                <td>{r.body}</td>
                <td>{r.lon.toFixed(4)}</td>
                <td>{r.retrograde ? "R" : ""}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>
    </div>
  );
}
```

- [ ] **Step 3: Rust print komutu**

`app/src-tauri/src/commands.rs` dosyasına, `planet_longitudes` komutundan sonra:

```rust
/// macOS'ta sistem print penceresini açar; kullanıcı oradan "PDF olarak kaydet"i seçer.
/// Senkron komut: print işlemi ana iş parçacığında çalışmalıdır.
#[tauri::command]
#[specta::specta]
pub fn print_window(window: tauri::WebviewWindow) -> Result<(), CommandError> {
    window
        .print()
        .map_err(|e| CommandError::new("print_failed", &[("detail", e.to_string())]))
}
```

`lib.rs` `collect_commands!` listesine `commands::print_window` ekle.

Komut: `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings && cargo test -p jyotish-desk`
Beklenen: `bindings.ts` dosyasında `printWindow` görünür; testler geçer.

- [ ] **Step 4: App'e bağla**

`app/ui/App.tsx` dosyasında:
- `import { PrintSpike } from "./PrintSpike";` ekle.
- `const [printView, setPrintView] = useState(false);` ekle.
- `<h1>` satırından sonra şu bloğu ekle:

```tsx
      <div className="no-print">
        <button onClick={() => setPrintView((v) => !v)}>
          {printView ? "Back" : "Print spike"}
        </button>
        {printView && (
          <button
            onClick={async () => {
              const r = await commands.printWindow();
              if (r.status === "error") setError(r.error);
            }}
          >
            Print…
          </button>
        )}
      </div>
      {printView && <PrintSpike rows={rows} />}
```

- Mevcut `<table>` öğesini `{!printView && ( ... )}` içine al.

`App.test.tsx` mock'una `printWindow: vi.fn(async () => ({ status: "ok", data: null }))` ekle.

Komut: `cd app && pnpm test && pnpm build`
Beklenen: 5 test geçer, build başarılı.

- [ ] **Step 5: Spike belgesi**

`docs/spikes/2026-11-24-wkwebview-print.md`:

```markdown
# Spike: WKWebView print → PDF

**Durum:** Kod hazır; cihaz testi bekliyor (Plan 02 sonunda elle).
**Karar kuralı:** Aşağıdaki maddelerin hepsi geçerse M7 print düzenleri HTML/CSS ile yapılır.
1, 2, 3 ya da 5 numaralı maddeden biri kalırsa yedek plan devreye girer: Rust tarafında Typst ile PDF.

## Nasıl test edilir

1. `cd app && pnpm tauri dev`
2. "Print spike" → "Print…" → print penceresinde "PDF" → "PDF olarak kaydet".
3. PDF'i Önizleme'de aç.

## Kontrol listesi

| # | Kontrol | Sonuç |
|---|---|---|
| 1 | `@page { size: A4 }` uygulanıyor (Önizleme > Araçlar > Denetçi: 210 × 297 mm) | |
| 2 | 15 mm kenar boşluğu uygulanıyor | |
| 3 | `break-after: page` ile tam 2 sayfa; tablo ikinci sayfada | |
| 4 | Harita SVG vektör kalıyor (yakınlaştırınca bulanıklaşmıyor) | |
| 5 | Metin seçilebilir; Türkçe karakterler (ğüşıöç İ) doğru | |
| 6 | Devanagari (राशि) doğru biçimleniyor | |
| 7 | Düğmeler (`.no-print`) PDF'te yok | |
| 8 | Koyu modda bile arka plan beyaz, metin siyah | |
| 9 | Print penceresinin üst/alt bilgi (URL, tarih) seçeneği kapatılabiliyor | |
| 10 | Letter boyutu seçilince düzen bozulmuyor | |

## Bulgular

(Cihaz testinden sonra doldurulacak.)
```

- [ ] **Step 6: Tam doğrulama ve commit**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
(cd app && pnpm test && pnpm build)
git add app docs/spikes
git commit -m "feat(app): print-to-PDF spike page and print_window command"
```

---

### Task 5: CI'ı uygulamayı kapsayacak şekilde güncelle

**Files:**
- Modify: `.github/workflows/ci.yml`
- Modify: `README.md` (Geliştirme bölümü)

**Interfaces:**
- Consumes: `app/package.json` script'leri (`test`, `build`), `cargo test -p jyotish-desk --test bindings`.
- Produces: CI işleri: `core-macos`, `core (ubuntu-latest)`, `core (windows-latest)`, `macos-x86_64`, `deny`, `msrv`.

- [ ] **Step 1: Workflow**

`.github/workflows/ci.yml` dosyasının tamamı:

```yaml
name: ci

on:
  push:
    branches: [main]
  pull_request:

jobs:
  core-macos:
    runs-on: macos-15
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 24
          cache: pnpm
          cache-dependency-path: app/pnpm-lock.yaml
      - run: pnpm --dir app install --frozen-lockfile
      - run: pnpm --dir app test
      - run: pnpm --dir app build
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all --check
      - run: cargo clippy --workspace --all-targets -- -D warnings
      - run: cargo test --workspace

  core:
    name: core (${{ matrix.os }})
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all --check
      - run: cargo clippy -p ephemeris --all-targets -- -D warnings
      - run: cargo test -p ephemeris

  macos-x86_64:
    runs-on: macos-15
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 24
          cache: pnpm
          cache-dependency-path: app/pnpm-lock.yaml
      - run: pnpm --dir app install --frozen-lockfile
      - run: pnpm --dir app build
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-apple-darwin
      - uses: Swatinem/rust-cache@v2
      - run: cargo build --workspace --target x86_64-apple-darwin

  deny:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: EmbarkStudios/cargo-deny-action@v2

  msrv:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.90
      - uses: Swatinem/rust-cache@v2
      - run: cargo check -p ephemeris --all-targets
```

`pnpm/action-setup@v4` sürümü `app/package.json` dosyasındaki `packageManager` alanından okur. Ancak bu alan kökte değil `app/` altında. Action kökte `package.json` bulamayıp hata verirse `with: { version: 11.8.0 }` ekle.

- [ ] **Step 2: README**

`README.md` dosyasının "Geliştirme" bölümüne şunları ekle:

````markdown
### Uygulama

```bash
cd app
pnpm install
pnpm tauri dev      # geliştirme penceresi
pnpm test           # arayüz testleri
```

Rust komut imzaları değişince TS bağlamalarını yeniden üretin:

```bash
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings
```

`app/ui/bindings.ts` elle düzenlenmez.
````

- [ ] **Step 3: Yerel doğrulama**

Komut: `cargo clippy -p ephemeris --all-targets -- -D warnings && cargo test -p ephemeris && cargo +1.90 check -p ephemeris --all-targets`
Beklenen: Geçer.

Workflow sözdizimi: `actionlint` kuruluysa `actionlint .github/workflows/ci.yml` çalıştır. Kurulu değilse YAML'ı `python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/ci.yml'))"` ile kontrol et; PyYAML da yoksa bu adımı atla ve rapora yaz.

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/ci.yml README.md
git commit -m "ci: build and test the Tauri app on macOS; core-only on Linux/Windows"
```

Push etmek ve CI'ı izlemek kontrolcünün işidir; implementer push etmez.

---

## Bu planın tamamlanma ölçütü

- `cargo test --workspace` (macOS) ve `cargo test -p ephemeris` (Linux/Windows) geçer.
- `pnpm --dir app test` ve `pnpm --dir app build` geçer.
- `bindings.ts` güncellik testi geçer.
- `cargo deny check` temiz.
- GitHub CI'daki 6 işin hepsi yeşil.
- Cihaz testi (spike kontrol listesi + `pnpm tauri dev` penceresi), kullanıcı kararı gereği bütün planların sonunda yapılır. O zamana kadar spike belgesinin durumu "cihaz testi bekliyor" kalır.
