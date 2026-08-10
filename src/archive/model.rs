use std::path::PathBuf;

use sqlx::SqlitePool;

#[derive(Debug)]
pub struct Archive {
    pool: SqlitePool,
    folder: PathBuf,
}

impl Archive {
    pub fn new(pool: SqlitePool, folder: PathBuf) -> Self {
        Self { pool, folder }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub fn folder(&self) -> &std::path::Path {
        &self.folder
    }

    pub async fn close(self) {
        self.pool.close().await;
    }
}
