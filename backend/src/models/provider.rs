use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub api_type: String,
    pub base_url: String,
    pub encrypted_api_key: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct ProviderResponse {
    pub id: String,
    pub name: String,
    pub api_type: String,
    pub base_url: String,
    pub api_key_masked: String,
    pub created_at: String,
    pub updated_at: String,
}

impl Provider {
    pub fn to_response(&self, masked_key: String) -> ProviderResponse {
        ProviderResponse {
            id: self.id.clone(),
            name: self.name.clone(),
            api_type: self.api_type.clone(),
            base_url: self.base_url.clone(),
            api_key_masked: masked_key,
            created_at: self.created_at.clone(),
            updated_at: self.updated_at.clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateProviderRequest {
    pub name: String,
    pub api_type: String,
    pub base_url: String,
    pub api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProviderRequest {
    pub name: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
}
