use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::models::{Fact, KnowledgeCard, ReviewCard, Slots};
use crate::store::Store;

#[derive(Debug, Deserialize)]
pub struct ImportConcept {
    pub title: String,
    #[serde(default)] pub aliases: Vec<String>,
    #[serde(default)] pub topic_ids: Vec<String>,
    #[serde(default)] pub slots: ImportSlots,
}

#[derive(Debug, Deserialize, Default)]
pub struct ImportSlots {
    #[serde(default)] pub definition: Vec<Fact>,
    #[serde(default)] pub mechanism: Vec<Fact>,
    #[serde(default)] pub complexity: Vec<Fact>,
    #[serde(default)] pub comparison: Vec<Fact>,
    #[serde(default)] pub use_cases: Vec<Fact>,
    #[serde(default)] pub interview_points: Vec<Fact>,
    #[serde(default)] pub pitfalls: Vec<Fact>,
    #[serde(default)] pub code: Vec<Fact>,
}

#[derive(Debug, Default)]
pub struct ImportReport {
    pub total_in: usize,
    pub created: usize,
    pub merged: usize,
    pub merged_into: Vec<(String, String)>, // (incoming_title, existing_id)
}

pub fn import_from_file(store: &Store, path: &Path) -> Result<ImportReport> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?;
    import_from_json(store, &text)
}

pub fn import_from_json(store: &Store, json: &str) -> Result<ImportReport> {
    let concepts: Vec<ImportConcept> = serde_json::from_str(json)
        .context("parse concept JSON array")?;

    // Build lookup of existing cards by normalized title and alias.
    let existing = store.list_cards()?;
    let mut by_key: HashMap<String, KnowledgeCard> = HashMap::new();
    for c in &existing {
        by_key.insert(norm(&c.title), c.clone());
        for a in &c.aliases { by_key.insert(norm(a), c.clone()); }
    }

    let mut report = ImportReport { total_in: concepts.len(), ..Default::default() };

    for inc in concepts {
        let key = norm(&inc.title);
        if let Some(mut existing_card) = by_key.get(&key).cloned() {
            // Merge into existing card.
            merge_slots(&mut existing_card.slots, inc.slots);
            for a in inc.aliases {
                if !existing_card.aliases.iter().any(|x| norm(x) == norm(&a)) {
                    existing_card.aliases.push(a);
                }
            }
            for t in inc.topic_ids {
                if !existing_card.topic_ids.contains(&t) {
                    existing_card.topic_ids.push(t);
                }
            }
            existing_card.updated_at = chrono::Utc::now();
            store.put_card(&existing_card)?;
            report.merged += 1;
            report.merged_into.push((inc.title, existing_card.id.clone()));
            // refresh lookup so subsequent concepts with the same title also merge
            by_key.insert(key, existing_card);
        } else {
            // Create new.
            let mut new_card = KnowledgeCard::new(inc.title, inc.topic_ids);
            new_card.aliases = inc.aliases;
            new_card.slots = into_slots(inc.slots);
            store.put_card(&new_card)?;
            by_key.insert(key, new_card.clone());
            report.created += 1;
        }
    }

    Ok(report)
}

fn merge_slots(dst: &mut Slots, src: ImportSlots) {
    extend(&mut dst.definition,        src.definition);
    extend(&mut dst.mechanism,         src.mechanism);
    extend(&mut dst.complexity,        src.complexity);
    extend(&mut dst.comparison,        src.comparison);
    extend(&mut dst.use_cases,         src.use_cases);
    extend(&mut dst.interview_points,  src.interview_points);
    extend(&mut dst.pitfalls,          src.pitfalls);
    extend(&mut dst.code,              src.code);
}

fn into_slots(src: ImportSlots) -> Slots {
    Slots {
        definition: src.definition,
        mechanism: src.mechanism,
        complexity: src.complexity,
        comparison: src.comparison,
        use_cases: src.use_cases,
        interview_points: src.interview_points,
        pitfalls: src.pitfalls,
        code: src.code,
    }
}

// dedup by exact-text to avoid double-ingest when re-running the same JSON.
fn extend(dst: &mut Vec<Fact>, src: Vec<Fact>) {
    for f in src {
        let dup = dst.iter().any(|x| x.text.trim() == f.text.trim());
        if !dup { dst.push(f); }
    }
}

fn norm(s: &str) -> String {
    s.trim().to_lowercase()
}

// ─────────────── Review (Q&A) bulk import ───────────────

#[derive(Debug, Deserialize)]
pub struct ImportReview {
    pub knowledge_card_title: String,
    pub question: String,
    pub answer: String,
}

#[derive(Debug, Default)]
pub struct ReviewImportReport {
    pub total_in: usize,
    pub created: usize,
    pub skipped_no_card: Vec<String>,
    pub skipped_dup_q: Vec<String>,
}

pub fn import_reviews_from_file(store: &Store, path: &Path) -> Result<ReviewImportReport> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?;
    import_reviews_from_json(store, &text)
}

pub fn import_reviews_from_json(store: &Store, json: &str) -> Result<ReviewImportReport> {
    let items: Vec<ImportReview> = serde_json::from_str(json)
        .context("parse review JSON array")?;

    let cards = store.list_cards()?;
    let mut by_key: HashMap<String, String> = HashMap::new();
    for c in &cards {
        by_key.insert(norm(&c.title), c.id.clone());
        for a in &c.aliases { by_key.insert(norm(a), c.id.clone()); }
    }

    // Existing (card_id, normalized_question) set for dedup
    let existing_reviews = store.list_reviews()?;
    let mut existing_keys: HashSet<(String, String)> = HashSet::new();
    for r in &existing_reviews {
        existing_keys.insert((r.knowledge_card_id.clone(), norm(&r.question)));
    }

    let mut report = ReviewImportReport { total_in: items.len(), ..Default::default() };

    for it in items {
        let card_id = match by_key.get(&norm(&it.knowledge_card_title)) {
            Some(id) => id.clone(),
            None => {
                report.skipped_no_card.push(it.knowledge_card_title);
                continue;
            }
        };
        let key = (card_id.clone(), norm(&it.question));
        if existing_keys.contains(&key) {
            report.skipped_dup_q.push(it.question);
            continue;
        }
        let rc = ReviewCard::new(card_id, it.question, it.answer);
        store.put_review(&rc)?;
        existing_keys.insert(key);
        report.created += 1;
    }

    Ok(report)
}
