mod commands;
mod import;
mod models;
mod srs;
mod state;
mod store;
mod suggest;
mod taxonomy;

use state::AppState;
use tauri::Manager;

/// Standalone CLI: bulk-import knowledge cards from a JSON file into the
/// production redb (under macOS Application Support). The Tauri app MUST be
/// stopped first — redb is single-writer.
pub fn import_cli(json_path: &str) -> anyhow::Result<()> {
    let path = std::path::Path::new(json_path);
    if !path.exists() { anyhow::bail!("file not found: {}", json_path); }
    let store = open_store_for_cli()?;
    let report = import::import_from_file(&store, path)?;
    println!("✓ imported {} concepts: {} created, {} merged",
        report.total_in, report.created, report.merged);
    for (title, id) in &report.merged_into {
        println!("  merged → {}  (existing card {})", title, &id[..8]);
    }
    Ok(())
}

/// Standalone CLI: bulk-import review (Q&A) cards. Each item must reference an
/// existing KnowledgeCard by title (or alias). Same single-writer caveat.
pub fn import_reviews_cli(json_path: &str) -> anyhow::Result<()> {
    let path = std::path::Path::new(json_path);
    if !path.exists() { anyhow::bail!("file not found: {}", json_path); }
    let store = open_store_for_cli()?;
    let report = import::import_reviews_from_file(&store, path)?;
    println!("✓ imported {} reviews: {} created, {} dup-skipped, {} missing-card-skipped",
        report.total_in, report.created, report.skipped_dup_q.len(), report.skipped_no_card.len());
    if !report.skipped_no_card.is_empty() {
        println!("⚠ skipped (no matching card by title):");
        for t in &report.skipped_no_card { println!("    - {}", t); }
    }
    Ok(())
}

fn open_store_for_cli() -> anyhow::Result<store::Store> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot resolve user data dir"))?
        .join("com.jyj.cs-interview-coach");
    std::fs::create_dir_all(&data_dir)?;
    let db_path = data_dir.join("coach.redb");
    store::Store::open(&db_path).map_err(Into::into)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle();
            let data_dir = handle.path().app_data_dir()
                .expect("resolve app_data_dir");

            let taxonomy_path = resolve_taxonomy_path();
            let tax = taxonomy::load_from_file(&taxonomy_path)
                .expect("load seed taxonomy");

            let app_state = AppState::new(data_dir, tax)
                .expect("init AppState");
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_taxonomy,
            commands::create_card,
            commands::get_card,
            commands::list_cards,
            commands::list_cards_by_topic,
            commands::update_card_slots,
            commands::delete_card,
            commands::suggest_titles,
            commands::create_review,
            commands::list_reviews_for_card,
            commands::due_reviews,
            commands::rate_review,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// In dev: walk up from CWD to find seed/taxonomy.yaml.
// M2 TODO: bundle as Tauri resource for packaged builds.
fn resolve_taxonomy_path() -> std::path::PathBuf {
    let mut dir = std::env::current_dir().expect("cwd");
    for _ in 0..5 {
        let candidate = dir.join("seed").join("taxonomy.yaml");
        if candidate.exists() { return candidate; }
        if !dir.pop() { break; }
    }
    std::path::PathBuf::from("../seed/taxonomy.yaml")
}
