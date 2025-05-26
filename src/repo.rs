use crate::Portfolio;
use crate::Result;
use dashmap::DashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

#[derive(Clone, Default)]
pub struct Repo {
    cache: DashMap<String, Portfolio>, // key = portfolio name/id
}

impl Repo {
    pub fn new() -> Self {
        let cache = DashMap::new();
        cache.insert("current".into(), Portfolio::default());
        Self { cache }
    }

    /* -------- API ---------- */

    pub fn get(&self) -> Portfolio {
        // clone is cheap thanks to Arc<String>/Vec internals
        self.cache.get("current").unwrap().clone()
    }

    pub fn upsert(&self, key: impl Into<String>, p: Portfolio) {
        self.cache.insert(key.into(), p);
    }

    pub fn remove(&self, key: impl Into<String>) -> Option<Portfolio> {
        self.cache.remove(&key.into()).map(|p| p.1)
    }

    pub fn with_portfolio<F, R>(&self, key: &str, f: F) -> Option<R>
    where
        F: FnOnce(&Portfolio) -> R,
    {
        self.cache.get(key).map(|guard| f(&guard))
    }

    pub fn list_keys(&self) -> Vec<String> {
        self.cache.iter().map(|r| r.key().clone()).collect()
    }

    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        Ok(serde_json::to_writer_pretty(writer, &self.list_as_vec())?)
    }

    pub fn load_from_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let vec: Vec<(String, Portfolio)> = serde_json::from_reader(reader)?;
        self.cache.clear();
        vec.into_iter().for_each(|(k, v)| {
            self.cache.insert(k, v);
        });
        Ok(())
    }

    /// Returns a Vec<(key, portfolio)> — handy for JSON dump.
    fn list_as_vec(&self) -> Vec<(String, Portfolio)> {
        self.cache
            .iter()
            .map(|r| (r.key().clone(), r.value().clone()))
            .collect()
    }

    pub fn set(&self, p: Portfolio) {
        self.cache.insert("current".into(), p);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use std::thread;

    fn empty_portfolio() -> Portfolio {
        Portfolio::default()
    }

    #[test]
    fn roundtrip() {
        let repo = Repo::new();
        repo.set(empty_portfolio());
        assert!(repo.get().positions.is_empty());
    }

    #[test]
    fn thread_safe() {
        let repo = Repo::new();
        let writer = repo.clone();
        thread::spawn(move || {
            for _ in 0..100 {
                writer.set(empty_portfolio());
            }
        })
        .join()
        .unwrap();
    }

    /// Helper: build a portfolio with one dummy domain
    fn demo_portfolio() -> Portfolio {
        Portfolio {
            cash: Decimal::try_from(42_000.0).unwrap(),
            positions: vec![BookLayer::Income(Position {
                id: "test".into(),
                underlying: "SPY".into(),
                pos_type: PositionType::CreditSpread,
                legs: vec![],
                margin_used: Decimal::try_from(10_000.0).unwrap(),
            })],
        }
    }

    #[test]
    fn repo_roundtrip() {
        let repo = Repo::new();
        let p = demo_portfolio();
        repo.set(p.clone());

        let fetched = repo.get();
        assert_eq!(fetched.cash, Decimal::try_from(42_000.0).unwrap());
        assert_eq!(fetched.positions.len(), 1);
        assert_eq!(fetched.positions[0].as_ref().id, "test");
    }

    #[test]
    fn repo_thread_safety() {
        let repo = Repo::new();
        let repo2 = repo.clone(); // share across threads

        // Spawn a writer thread
        let handle = thread::spawn(move || {
            for i in 0..100 {
                let mut p = demo_portfolio();
                p.cash += Decimal::from_i32(i).unwrap();
                repo2.set(p);
            }
        });

        // Meanwhile read repeatedly
        for _ in 0..100 {
            let _ = repo.get(); // should never panic
        }

        handle.join().unwrap();
    }
}
