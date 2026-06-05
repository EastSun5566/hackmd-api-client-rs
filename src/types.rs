use chrono::{DateTime, TimeZone, Utc};
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

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
pub enum SuggestEditPermissionType {
    Disabled,
    Forbidden,
    Owners,
    SignedInUsers,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NotePermissionRole {
    Owner,
    SignedIn,
    Guest,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderPath {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub client_id: String,
}

/// Personal folder ordering keyed by either `root` or a parent folder ID.
pub type FolderOrder = BTreeMap<String, Vec<String>>;

/// Arbitrary note feature configuration accepted by HackMD's create-note APIs.
///
/// The upstream OpenAPI schema currently leaves the object shape underspecified,
/// so the client preserves it as raw JSON keyed by feature name for forward
/// compatibility.
pub type NoteFeatures = BTreeMap<String, Value>;

fn datetime_from_milliseconds<E>(value: f64) -> Result<DateTime<Utc>, E>
where
    E: de::Error,
{
    if !value.is_finite() {
        return Err(E::custom("timestamp must be finite"));
    }

    Utc.timestamp_millis_opt(value.round() as i64)
        .single()
        .ok_or_else(|| E::custom("timestamp is out of range"))
}

fn deserialize_ts_milliseconds<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = f64::deserialize(deserializer)?;
    datetime_from_milliseconds(value)
}

fn deserialize_ts_milliseconds_option<'de, D>(
    deserializer: D,
) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<f64>::deserialize(deserializer)?
        .map(datetime_from_milliseconds)
        .transpose()
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub parent_folder_id: Option<String>,
    #[serde(deserialize_with = "deserialize_ts_milliseconds")]
    pub created_at: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_ts_milliseconds")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateFolderOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_folder_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// Options for partially updating a folder.
///
/// Nullable fields use a nested option so callers can distinguish among:
/// - `None`: do not send the field
/// - `Some(Some(value))`: set the field to a concrete value
/// - `Some(None)`: explicitly clear the field by sending `null`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFolderOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_folder_id: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFolderOrderOptions {
    pub order: FolderOrder,
}

/// Options for creating a note in a personal or team workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateNoteOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_features: Option<NoteFeatures>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_permission: Option<NotePermissionRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_permission: Option<NotePermissionRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_permission: Option<CommentPermissionType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggest_edit_permission: Option<SuggestEditPermissionType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permalink: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_folder_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct NoteImageUploadData {
    pub link: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct NoteImageUploadResponse {
    pub data: NoteImageUploadData,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub id: String,
    pub owner_id: String,
    pub name: String,
    pub logo: String,
    pub path: String,
    pub description: Option<String>,
    pub visibility: TeamVisibilityType,
    #[serde(deserialize_with = "deserialize_ts_milliseconds")]
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
    #[serde(default)]
    pub description: String,
    pub tags: Vec<String>,
    #[serde(deserialize_with = "deserialize_ts_milliseconds")]
    pub last_changed_at: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_ts_milliseconds")]
    pub created_at: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_ts_milliseconds_option")]
    pub title_updated_at: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "deserialize_ts_milliseconds_option")]
    pub tags_updated_at: Option<DateTime<Utc>>,
    pub last_change_user: Option<SimpleUserProfile>,
    pub publish_type: NotePublishType,
    #[serde(deserialize_with = "deserialize_ts_milliseconds_option")]
    pub published_at: Option<DateTime<Utc>>,
    pub user_path: Option<String>,
    pub team_path: Option<String>,
    pub permalink: Option<String>,
    pub short_id: String,
    pub publish_link: String,
    #[serde(default)]
    pub folder_paths: Vec<FolderPath>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNoteOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_permission: Option<NotePermissionRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_permission: Option<NotePermissionRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permalink: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_folder_id: Option<String>,
}
