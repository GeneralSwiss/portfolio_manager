use crate::Portfolio;
use dashmap::DashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct Repo {
    cache: Arc<DashMap<String, Portfolio>>, // key = "current"
}

impl Default for Repo {
    fn default() -> Self {
        Self::new()
    }
}

impl Repo {
    pub fn new() -> Self {
        let cache = DashMap::new();
        cache.insert("current".into(), Portfolio::default());
        Self {
            cache: Arc::new(cache),
        }
    }

    /* -------- API ---------- */

    pub fn get(&self) -> Portfolio {
        // clone is cheap thanks to Arc<String>/Vec internals
        self.cache.get("current").unwrap().clone()
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
            positions: vec![Position {
                id: "test".into(),
                underlying: "SPY".into(),
                book_layer: BookLayer::Income,
                pos_type: PositionType::CreditSpread,
                legs: vec![],
                margin_used: 10_000.0,
            }],
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
        assert_eq!(fetched.positions[0].id, "test");
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
