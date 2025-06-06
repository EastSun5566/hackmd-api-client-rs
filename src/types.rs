use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TeamVisibilityType {
    Public,
    Private,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NotePublishType {
    Edit,
    View,
    Slide,
    Book,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommentPermissionType {
    Disabled,
    Forbidden,
    Owners,
    SignedInUsers,
    Everyone,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NotePermissionRole {
    Owner,
    SignedIn,
    Guest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNoteOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_permission: Option<NotePermissionRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_permission: Option<NotePermissionRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_permission: Option<CommentPermissionType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permalink: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub id: String,
    pub owner_id: Option<String>,
    pub name: String,
    pub logo: String,
    pub path: String,
    pub description: Option<String>,
    pub hard_breaks: Option<bool>,
    pub visibility: TeamVisibilityType,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub created_at: DateTime<Utc>,
    pub upgraded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub email: Option<String>,
    pub name: String,
    pub user_path: String,
    pub photo: String,
    pub teams: Vec<Team>,
    pub upgraded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimpleUserProfile {
    pub name: String,
    pub user_path: String,
    pub photo: String,
    pub biography: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub title: String,
    pub tags: Vec<String>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub last_changed_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub created_at: DateTime<Utc>,
    pub last_change_user: Option<SimpleUserProfile>,
    pub publish_type: NotePublishType,
    #[serde(with = "chrono::serde::ts_milliseconds_option")]
    pub published_at: Option<DateTime<Utc>>,
    pub user_path: Option<String>,
    pub team_path: Option<String>,
    pub permalink: Option<String>,
    pub short_id: String,
    pub publish_link: String,
    pub read_permission: NotePermissionRole,
    pub write_permission: NotePermissionRole,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SingleNote {
    pub content: String,
    #[serde(flatten)]
    pub note: Note,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNoteOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_permission: Option<NotePermissionRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_permission: Option<NotePermissionRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permalink: Option<String>,
}
