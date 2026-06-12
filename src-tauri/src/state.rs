use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

use crate::jobs::JobsFile;
use crate::store::Store;
use crate::suggest::Suggester;
use crate::taxonomy::Taxonomy;

pub struct AppState {
    pub store: Arc<Store>,
    pub suggester: Arc<Suggester>,
    pub taxonomy: Arc<Taxonomy>,
    pub jobs: Arc<JobsFile>,
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn new(data_dir: PathBuf, taxonomy: Taxonomy, jobs: JobsFile) -> Result<Self> {
        std::fs::create_dir_all(&data_dir)?;
        let db_path = data_dir.join("coach.redb");
        let store = Arc::new(Store::open(&db_path)?);
        let suggester = Arc::new(Suggester::default());

        // Initial FST build from whatever's in store.
        let cards = store.list_cards()?;
        suggester.rebuild(&cards)?;

        Ok(Self {
            store,
            suggester,
            taxonomy: Arc::new(taxonomy),
            jobs: Arc::new(jobs),
            data_dir,
        })
    }

    pub fn refresh_suggester(&self) -> Result<()> {
        let cards = self.store.list_cards()?;
        self.suggester.rebuild(&cards)?;
        Ok(())
    }
}
