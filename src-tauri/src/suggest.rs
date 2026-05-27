use anyhow::Result;
use fst::{IntoStreamer, Streamer};
use std::collections::HashMap;
use std::sync::RwLock;

use crate::models::KnowledgeCard;

// M1: in-memory FST title autocomplete. Rebuilt from KnowledgeCard list on
// every mutation (cards << 5000 so this is < 10ms).
// M2 TODO: incremental update; merge with Tantivy for full-text BM25.

pub struct Suggester {
    inner: RwLock<Inner>,
}

struct Inner {
    fst: fst::Set<Vec<u8>>,
    // normalized_key -> KnowledgeCard.id
    key_to_card: HashMap<String, String>,
}

impl Default for Suggester {
    fn default() -> Self {
        let empty: Vec<&[u8]> = Vec::new();
        Self {
            inner: RwLock::new(Inner {
                fst: fst::Set::from_iter(empty).expect("empty fst"),
                key_to_card: HashMap::new(),
            }),
        }
    }
}

impl Suggester {
    pub fn rebuild(&self, cards: &[KnowledgeCard]) -> Result<()> {
        let mut entries: Vec<(String, String)> = Vec::new();
        for c in cards {
            entries.push((normalize(&c.title), c.id.clone()));
            for a in &c.aliases {
                entries.push((normalize(a), c.id.clone()));
            }
        }
        // dedup + sort for FST builder
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        entries.dedup_by(|a, b| a.0 == b.0);

        let keys: Vec<&[u8]> = entries.iter().map(|(k, _)| k.as_bytes()).collect();
        let fst = fst::Set::from_iter(keys)?;

        let key_to_card = entries.into_iter().collect();
        let mut w = self.inner.write().unwrap();
        w.fst = fst;
        w.key_to_card = key_to_card;
        Ok(())
    }

    pub fn suggest(&self, prefix: &str, limit: usize) -> Vec<Suggestion> {
        let prefix = normalize(prefix);
        let r = self.inner.read().unwrap();
        let mut out = Vec::new();
        let mut stream = r.fst.range().ge(&prefix).into_stream();
        while let Some(k) = stream.next() {
            let key = match std::str::from_utf8(k) {
                Ok(s) => s,
                Err(_) => continue,
            };
            if !key.starts_with(&prefix) { break; }
            if let Some(id) = r.key_to_card.get(key) {
                out.push(Suggestion { key: key.to_string(), card_id: id.clone() });
            }
            if out.len() >= limit { break; }
        }
        out
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Suggestion {
    pub key: String,
    pub card_id: String,
}

fn normalize(s: &str) -> String {
    s.trim().to_lowercase()
}
