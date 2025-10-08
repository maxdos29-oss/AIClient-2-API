/*!
 * Provider Pool Manager
 *
 * Manages pools of API service providers with health checking and load balancing.
 */

use crate::config::ProviderConfig;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ProviderPoolManager {
    pools: Arc<RwLock<HashMap<String, Vec<ProviderStatus>>>>,
    round_robin_index: Arc<RwLock<HashMap<String, usize>>>,
    max_error_count: u32,
}

struct ProviderStatus {
    config: ProviderConfig,
    is_healthy: bool,
}

impl ProviderPoolManager {
    pub fn new(pools: HashMap<String, Vec<ProviderConfig>>) -> Self {
        let mut status_pools = HashMap::new();
        
        for (provider_type, configs) in pools {
            let statuses: Vec<ProviderStatus> = configs
                .into_iter()
                .map(|config| ProviderStatus {
                    is_healthy: config.is_healthy,
                    config,
                })
                .collect();
            status_pools.insert(provider_type, statuses);
        }

        Self {
            pools: Arc::new(RwLock::new(status_pools)),
            round_robin_index: Arc::new(RwLock::new(HashMap::new())),
            max_error_count: 3,
        }
    }

    pub async fn select_provider(&self, provider_type: &str) -> Option<ProviderConfig> {
        let pools = self.pools.read().await;
        let pool = pools.get(provider_type)?;
        
        let healthy_providers: Vec<&ProviderStatus> = pool
            .iter()
            .filter(|p| p.is_healthy)
            .collect();

        if healthy_providers.is_empty() {
            return None;
        }

        let mut indices = self.round_robin_index.write().await;
        let current_index = indices.entry(provider_type.to_string()).or_insert(0);
        
        let selected = &healthy_providers[*current_index % healthy_providers.len()];
        *current_index = (*current_index + 1) % healthy_providers.len();

        Some(selected.config.clone())
    }

    pub async fn mark_provider_unhealthy(&self, provider_type: &str, uuid: &str) {
        let mut pools = self.pools.write().await;
        if let Some(pool) = pools.get_mut(provider_type) {
            for provider in pool.iter_mut() {
                if provider.config.uuid == uuid {
                    provider.is_healthy = false;
                    tracing::warn!(
                        "Marked provider {} ({}) as unhealthy",
                        provider_type,
                        uuid
                    );
                    break;
                }
            }
        }
    }

    pub async fn mark_provider_healthy(&self, provider_type: &str, uuid: &str) {
        let mut pools = self.pools.write().await;
        if let Some(pool) = pools.get_mut(provider_type) {
            for provider in pool.iter_mut() {
                if provider.config.uuid == uuid {
                    provider.is_healthy = true;
                    tracing::info!(
                        "Marked provider {} ({}) as healthy",
                        provider_type,
                        uuid
                    );
                    break;
                }
            }
        }
    }

    pub async fn perform_health_checks(&self) {
        // TODO: Implement periodic health checks
        tracing::info!("Performing health checks on all providers...");
    }
}

