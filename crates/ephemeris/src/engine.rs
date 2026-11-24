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
