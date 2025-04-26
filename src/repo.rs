use crate::Portfolio;
use dashmap::DashMap;
use serde_json;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::time::{Duration, SystemTime};
use tokio::{sync::mpsc, task};

const CHANNEL_CAP: usize = 32;           // queued writes
const RETRY_DELAY: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub struct Repo {
    /// fast read cache
    cache: DashMap<String, Portfolio>,
    /// async write sender
    tx:    mpsc::Sender<Portfolio>,
}

impl Repo {
    pub async fn new(db_url: &str) -> anyhow::Result<Self> {
        // Build connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(4)
            .connect(db_url)
            .await?;

        // Migration
        sqlx::query(include_str!("schema.sql")).execute(&pool).await?;

        // Spawn write-behind task
        let (tx, mut rx) = mpsc::channel::<Portfolio>(CHANNEL_CAP);
        task::spawn(async move {
            while let Some(p) = rx.recv().await {
                // retry loop
                loop {
                    let blob = serde_json::to_string(&p).expect("serialize");
                    let now  = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;
                    let res = sqlx::query!(
                        "REPLACE INTO portfolio_snapshots (id, json_blob, updated)
                         VALUES ('current', ?, ?)",
                        blob,
                        now
                    )
                    .execute(&pool)
                    .await;

                    match res {
                        Ok(_) => break,                       // success
                        Err(e) => {
                            eprintln!("db write failed: {e:?} — retrying in 5s");
                            tokio::time::sleep(RETRY_DELAY).await;
                        }
                    }
                }
            }
        });

        Ok(Self { cache: DashMap::new(), tx })
    }

    /// Read-only access is always from cache (very fast).
    pub fn get_current(&self) -> Option<Portfolio> {
        self.cache.get("current").map(|r| r.clone())
    }

    /// Update portfolio in-mem and enqueue async write.
    pub fn upsert(&self, p: Portfolio) {
        self.cache.insert("current".into(), p.clone());
        // best-effort: ignore if channel full (rare, but keeps app non-blocking)
        let _ = self.tx.try_send(p);
    }
}
