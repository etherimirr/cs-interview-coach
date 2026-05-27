use anyhow::{Context, Result};
use redb::{Database, ReadableTable, TableDefinition};
use std::path::Path;
use std::sync::Arc;

use crate::models::{KnowledgeCard, ReviewCard};

const CARDS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("knowledge_cards");
const REVIEWS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("review_cards");

pub struct Store {
    db: Arc<Database>,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        let db = Database::create(path).context("open redb")?;
        let store = Self { db: Arc::new(db) };
        store.init_tables()?;
        Ok(store)
    }

    fn init_tables(&self) -> Result<()> {
        let txn = self.db.begin_write()?;
        { let _ = txn.open_table(CARDS_TABLE)?; }
        { let _ = txn.open_table(REVIEWS_TABLE)?; }
        txn.commit()?;
        Ok(())
    }

    // ---------- KnowledgeCard ----------

    pub fn put_card(&self, card: &KnowledgeCard) -> Result<()> {
        let json = serde_json::to_string(card)?;
        let txn = self.db.begin_write()?;
        {
            let mut t = txn.open_table(CARDS_TABLE)?;
            t.insert(card.id.as_str(), json.as_str())?;
        }
        txn.commit()?;
        Ok(())
    }

    pub fn get_card(&self, id: &str) -> Result<Option<KnowledgeCard>> {
        let txn = self.db.begin_read()?;
        let t = txn.open_table(CARDS_TABLE)?;
        Ok(match t.get(id)? {
            Some(v) => Some(serde_json::from_str(v.value())?),
            None => None,
        })
    }

    pub fn list_cards(&self) -> Result<Vec<KnowledgeCard>> {
        let txn = self.db.begin_read()?;
        let t = txn.open_table(CARDS_TABLE)?;
        let mut out = Vec::new();
        for entry in t.iter()? {
            let (_k, v) = entry?;
            out.push(serde_json::from_str(v.value())?);
        }
        Ok(out)
    }

    pub fn list_cards_by_topic(&self, topic_id: &str) -> Result<Vec<KnowledgeCard>> {
        Ok(self.list_cards()?
            .into_iter()
            .filter(|c| c.topic_ids.iter().any(|t| t == topic_id))
            .collect())
    }

    pub fn delete_card(&self, id: &str) -> Result<()> {
        let txn = self.db.begin_write()?;
        {
            let mut t = txn.open_table(CARDS_TABLE)?;
            t.remove(id)?;
        }
        txn.commit()?;
        Ok(())
    }

    // ---------- ReviewCard ----------

    pub fn put_review(&self, card: &ReviewCard) -> Result<()> {
        let json = serde_json::to_string(card)?;
        let txn = self.db.begin_write()?;
        {
            let mut t = txn.open_table(REVIEWS_TABLE)?;
            t.insert(card.id.as_str(), json.as_str())?;
        }
        txn.commit()?;
        Ok(())
    }

    pub fn get_review(&self, id: &str) -> Result<Option<ReviewCard>> {
        let txn = self.db.begin_read()?;
        let t = txn.open_table(REVIEWS_TABLE)?;
        Ok(match t.get(id)? {
            Some(v) => Some(serde_json::from_str(v.value())?),
            None => None,
        })
    }

    pub fn list_reviews(&self) -> Result<Vec<ReviewCard>> {
        let txn = self.db.begin_read()?;
        let t = txn.open_table(REVIEWS_TABLE)?;
        let mut out = Vec::new();
        for entry in t.iter()? {
            let (_k, v) = entry?;
            out.push(serde_json::from_str(v.value())?);
        }
        Ok(out)
    }

    pub fn list_reviews_for_card(&self, knowledge_card_id: &str) -> Result<Vec<ReviewCard>> {
        Ok(self.list_reviews()?
            .into_iter()
            .filter(|r| r.knowledge_card_id == knowledge_card_id)
            .collect())
    }

    pub fn due_reviews(&self, now: chrono::DateTime<chrono::Utc>) -> Result<Vec<ReviewCard>> {
        let mut due: Vec<ReviewCard> = self.list_reviews()?
            .into_iter()
            .filter(|r| r.fsrs.next_review <= now)
            .collect();
        due.sort_by_key(|r| r.fsrs.next_review);
        Ok(due)
    }
}
