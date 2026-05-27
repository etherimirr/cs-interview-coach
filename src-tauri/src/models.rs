use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cite {
    pub source_id: String,
    pub locator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Fact {
    pub text: String,
    #[serde(default)]
    pub cites: Vec<Cite>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Slots {
    #[serde(default)] pub definition: Vec<Fact>,
    #[serde(default)] pub mechanism: Vec<Fact>,
    #[serde(default)] pub complexity: Vec<Fact>,
    #[serde(default)] pub comparison: Vec<Fact>,
    #[serde(default)] pub use_cases: Vec<Fact>,
    #[serde(default)] pub interview_points: Vec<Fact>,
    #[serde(default)] pub pitfalls: Vec<Fact>,
    #[serde(default)] pub code: Vec<Fact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeCard {
    pub id: String,
    pub title: String,
    #[serde(default)] pub aliases: Vec<String>,
    #[serde(default)] pub topic_ids: Vec<String>,
    #[serde(default)] pub related_card_ids: Vec<String>,
    #[serde(default)] pub slots: Slots,
    #[serde(default)] pub question_tree: Vec<QuestionNode>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl KnowledgeCard {
    pub fn new(title: impl Into<String>, topic_ids: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.into(),
            aliases: vec![],
            topic_ids,
            related_card_ids: vec![],
            slots: Slots::default(),
            question_tree: vec![],
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionNode {
    pub dimension: QuestionDimension,
    pub question: String,
    #[serde(default)] pub answer_outline: String,
    #[serde(default)] pub children: Vec<QuestionNode>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum QuestionDimension {
    What,
    Why,
    How,
    When,
    Pitfall,
    Extension,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewCard {
    pub id: String,
    pub knowledge_card_id: String,
    pub question: String,
    pub answer: String,
    pub fsrs: FsrsState,
    pub created_at: DateTime<Utc>,
}

impl ReviewCard {
    pub fn new(knowledge_card_id: String, question: String, answer: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            knowledge_card_id,
            question,
            answer,
            fsrs: FsrsState::new(now),
            created_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsrsState {
    pub stability: f32,
    pub difficulty: f32,
    pub last_review: DateTime<Utc>,
    pub next_review: DateTime<Utc>,
    pub reps: u32,
    pub lapses: u32,
}

impl FsrsState {
    pub fn new(now: DateTime<Utc>) -> Self {
        Self {
            stability: 1.0,
            difficulty: 5.0,
            last_review: now,
            next_review: now,
            reps: 0,
            lapses: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Grade {
    Again,
    Hard,
    Good,
    Easy,
}
