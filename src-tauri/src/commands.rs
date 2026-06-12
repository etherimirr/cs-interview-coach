use chrono::Utc;
use std::collections::HashSet;
use tauri::State;

use crate::jobs::Job;
use crate::models::{Grade, KnowledgeCard, ReviewCard, Slots};
use crate::srs;
use crate::state::AppState;
use crate::suggest::Suggestion;
use crate::taxonomy::Taxonomy;

type CmdResult<T> = std::result::Result<T, String>;

fn err<E: std::fmt::Display>(e: E) -> String { e.to_string() }

// ---------- Taxonomy ----------

#[tauri::command]
pub fn get_taxonomy(state: State<'_, AppState>) -> CmdResult<Taxonomy> {
    Ok((*state.taxonomy).clone())
}

// ---------- KnowledgeCard ----------

#[tauri::command]
pub fn create_card(
    title: String,
    topic_ids: Vec<String>,
    state: State<'_, AppState>,
) -> CmdResult<KnowledgeCard> {
    let card = KnowledgeCard::new(title, topic_ids);
    state.store.put_card(&card).map_err(err)?;
    state.refresh_suggester().map_err(err)?;
    Ok(card)
}

#[tauri::command]
pub fn get_card(id: String, state: State<'_, AppState>) -> CmdResult<Option<KnowledgeCard>> {
    state.store.get_card(&id).map_err(err)
}

#[tauri::command]
pub fn list_cards(state: State<'_, AppState>) -> CmdResult<Vec<KnowledgeCard>> {
    state.store.list_cards().map_err(err)
}

#[tauri::command]
pub fn list_cards_by_topic(
    topic_id: String,
    state: State<'_, AppState>,
) -> CmdResult<Vec<KnowledgeCard>> {
    state.store.list_cards_by_topic(&topic_id).map_err(err)
}

#[tauri::command]
pub fn update_card_slots(
    id: String,
    slots: Slots,
    aliases: Vec<String>,
    state: State<'_, AppState>,
) -> CmdResult<KnowledgeCard> {
    let mut card = state.store.get_card(&id).map_err(err)?
        .ok_or_else(|| "card not found".to_string())?;
    card.slots = slots;
    card.aliases = aliases;
    card.updated_at = Utc::now();
    state.store.put_card(&card).map_err(err)?;
    state.refresh_suggester().map_err(err)?;
    Ok(card)
}

#[tauri::command]
pub fn delete_card(id: String, state: State<'_, AppState>) -> CmdResult<()> {
    state.store.delete_card(&id).map_err(err)?;
    state.refresh_suggester().map_err(err)?;
    Ok(())
}

// ---------- Suggest / Search ----------

#[tauri::command]
pub fn suggest_titles(
    prefix: String,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> CmdResult<Vec<Suggestion>> {
    let lim = limit.unwrap_or(10);
    Ok(state.suggester.suggest(&prefix, lim))
}

// ---------- ReviewCard ----------

#[tauri::command]
pub fn create_review(
    knowledge_card_id: String,
    question: String,
    answer: String,
    state: State<'_, AppState>,
) -> CmdResult<ReviewCard> {
    let rc = ReviewCard::new(knowledge_card_id, question, answer);
    state.store.put_review(&rc).map_err(err)?;
    Ok(rc)
}

#[tauri::command]
pub fn list_reviews_for_card(
    knowledge_card_id: String,
    state: State<'_, AppState>,
) -> CmdResult<Vec<ReviewCard>> {
    state.store.list_reviews_for_card(&knowledge_card_id).map_err(err)
}

#[tauri::command]
pub fn due_reviews(state: State<'_, AppState>) -> CmdResult<Vec<ReviewCard>> {
    state.store.due_reviews(Utc::now()).map_err(err)
}

// ---------- Jobs ----------

#[tauri::command]
pub fn list_jobs(state: State<'_, AppState>) -> CmdResult<Vec<Job>> {
    Ok(state.jobs.jobs.clone())
}

#[tauri::command]
pub fn get_job(id: String, state: State<'_, AppState>) -> CmdResult<Option<Job>> {
    Ok(state.jobs.jobs.iter().find(|j| j.id == id).cloned())
}

/// Return all KnowledgeCards relevant to a Job, which is the union of:
///   1) every card whose topic_ids intersects job.relevant_topic_ids
///   2) every card whose title or aliases matches a job.cherry_picked_cards entry
///   3) every card whose title or aliases matches a job.my_anchors entry
/// Returned cards are deduplicated by id; cards may carry a "match_reason"
/// classification — but for M1 we return the raw KnowledgeCard list and let the
/// frontend group by (cherry / anchor / topic) on its own using the job spec.
#[tauri::command]
pub fn list_cards_for_job(
    id: String,
    state: State<'_, AppState>,
) -> CmdResult<Vec<KnowledgeCard>> {
    let job = state.jobs.jobs.iter().find(|j| j.id == id).cloned()
        .ok_or_else(|| format!("job not found: {id}"))?;
    let cards = state.store.list_cards().map_err(err)?;

    let wanted_topics: HashSet<String> = job.relevant_topic_ids.iter().cloned().collect();
    let cherry: HashSet<String> = job.cherry_picked_cards.iter()
        .map(|s| s.trim().to_lowercase()).collect();
    let anchors: HashSet<String> = job.my_anchors.iter()
        .map(|s| s.trim().to_lowercase()).collect();

    let mut matched: Vec<KnowledgeCard> = cards.into_iter().filter(|c| {
        // 1) topic intersection
        if c.topic_ids.iter().any(|t| wanted_topics.contains(t)) { return true; }
        // 2) title in cherry / anchors
        let title_norm = c.title.trim().to_lowercase();
        if cherry.contains(&title_norm) || anchors.contains(&title_norm) {
            return true;
        }
        // 3) any alias in cherry / anchors
        for a in &c.aliases {
            let an = a.trim().to_lowercase();
            if cherry.contains(&an) || anchors.contains(&an) { return true; }
        }
        false
    }).collect();

    // Sort: cherry-picked > anchors > by title
    matched.sort_by(|a, b| {
        let a_cherry = cherry.contains(&a.title.trim().to_lowercase());
        let b_cherry = cherry.contains(&b.title.trim().to_lowercase());
        match (a_cherry, b_cherry) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.title.cmp(&b.title),
        }
    });

    Ok(matched)
}

#[tauri::command]
pub fn rate_review(
    id: String,
    grade: Grade,
    state: State<'_, AppState>,
) -> CmdResult<ReviewCard> {
    let mut rc = state.store.get_review(&id).map_err(err)?
        .ok_or_else(|| "review card not found".to_string())?;
    srs::apply_grade(&mut rc.fsrs, grade, Utc::now());
    state.store.put_review(&rc).map_err(err)?;
    Ok(rc)
}
