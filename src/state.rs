// App State
use std::sync::{Arc, Mutex};

use crate::models::apiconfig::ApiConfig;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Mutex<ApiConfig>>,
    pub dbpool: sqlx::Pool<sqlx::Sqlite>,
}
