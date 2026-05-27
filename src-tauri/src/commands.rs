use chrono::Utc;
use tauri::State;

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
