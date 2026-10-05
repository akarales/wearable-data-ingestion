//! Shared state: provider + normalized store (per user).

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::RwLock;

use crate::normalize::Observation;
use crate::provider::Provider;

#[derive(Clone)]
pub struct AppState {
    pub provider: Provider,
    pub store: std::sync::Arc<Store>,
}

pub struct Store {
    // user -> LOINC code -> chronological observations
    user_data: RwLock<BTreeMap<String, BTreeMap<String, Vec<Observation>>>>,
}

impl Store {
    pub fn insert(&self, user_id: &str, observations: &[Observation]) {
        let mut data = self.user_data.write().expect("store lock");
        for observation in observations {
            let code = observation.code.coding[0].code.clone();
            data.entry(user_id.to_string())
                .or_default()
                .entry(code)
                .or_default()
                .push(observation.clone());
        }
        for series in data.values_mut() {
            for observations in series.values_mut() {
                observations.sort_by_key(|o| o.effective_date_time);
            }
        }
    }

    pub fn observations(&self, user_id: &str, code: Option<&str>) -> Vec<Observation> {
        let data = self.user_data.read().expect("store lock");
        match code {
            Some(code) => data
                .get(user_id)
                .and_then(|user| user.get(code))
                .cloned()
                .unwrap_or_default(),
            None => {
                let mut all: Vec<Observation> = data
                    .get(user_id)
                    .map(|user| {
                        user.values()
                            .flat_map(Vec::clone)
                            .collect::<Vec<Observation>>()
                    })
                    .unwrap_or_default();
                all.sort_by_key(|o| o.effective_date_time);
                all
            }
        }
    }

    pub fn codes(&self, user_id: &str) -> Vec<String> {
        let data = self.user_data.read().expect("store lock");
        data.get(user_id)
            .map(|user| user.keys().cloned().collect())
            .unwrap_or_default()
    }
}

impl AppState {
    /// Offline default: the committed fixture provider.
    pub fn with_fixture() -> Self {
        Self::with_provider(Provider::Fixture {
            path: PathBuf::from("tests/fixtures/wearable_series.json"),
        })
    }

    pub fn with_provider(provider: Provider) -> Self {
        Self {
            provider,
            store: std::sync::Arc::new(Store {
                user_data: RwLock::new(BTreeMap::new()),
            }),
        }
    }
}
