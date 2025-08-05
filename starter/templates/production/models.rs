use crate::error::Error;
use crate::types::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct __MODULE_STRUCT__ {
    pub id: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub user_id: Uuid,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl __MODULE_STRUCT__ {
    pub fn to_response(&self) -> __MODULE_STRUCT__Response {
        __MODULE_STRUCT__Response {
            id: self.id,
            title: self.title.clone(),
            content: self.content.clone(),
            user_id: self.user_id,
            is_active: self.is_active,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct __MODULE_STRUCT__Response {
    pub id: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub user_id: Uuid,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Create__MODULE_STRUCT__Request {
    pub title: String,
    pub content: Option<String>,
}

impl Create__MODULE_STRUCT__Request {
    pub fn validate(&self) -> Result<()> {
        validate_title(&self.title)?;
        if let Some(ref content) = self.content {
            validate_content(content)?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Update__MODULE_STRUCT__Request {
    pub title: Option<String>,
    pub content: Option<String>,
}

impl Update__MODULE_STRUCT__Request {
    pub fn validate(&self) -> Result<()> {
        if let Some(ref title) = self.title {
            validate_title(title)?;
        }
        if let Some(ref content) = self.content {
            validate_content(content)?;
        }
        Ok(())
    }
}

pub fn validate_title(title: &str) -> Result<()> {
    if title.trim().is_empty() {
        return Err(Error::validation("title", "Title cannot be empty"));
    }
    if title.len() > 255 {
        return Err(Error::validation(
            "title",
            "Title must be less than 255 characters",
        ));
    }
    Ok(())
}

pub fn validate_content(content: &str) -> Result<()> {
    if content.len() > 10000 {
        return Err(Error::validation(
            "content",
            "Content must be less than 10000 characters",
        ));
    }
    Ok(())
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct __MODULE_STRUCT__Stats {
    pub total___MODULE_NAME_PLURAL__: i64,
    pub active___MODULE_NAME_PLURAL__: i64,
    pub inactive___MODULE_NAME_PLURAL__: i64,
    pub __MODULE_NAME_PLURAL___by_user: Vec<UserStats>,
    pub recent___MODULE_NAME_PLURAL__: Recent__MODULE_STRUCT__s,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserStats {
    pub user_id: Uuid,
    pub count: i64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct Recent__MODULE_STRUCT__s {
    pub last_24h: i64,
    pub last_7d: i64,
    pub last_30d: i64,
}