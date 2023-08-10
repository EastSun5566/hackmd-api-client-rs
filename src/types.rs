use chrono::NaiveDateTime;
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Deserialize)]
enum TeamVisibilityType {
    PUBLIC,
    PRIVATE,
}

// enum NotePublishType {
//     EDIT,
//     VIEW,
//     SLIDE,
//     BOOK,
// }

// enum CommentPermissionType {
//     DISABLED,
//     FORBIDDEN,
//     OWNERS,
//     SIGNED_IN_USERS,
//     EVERYONE,
// }

// enum NotePermissionRole {
//     OWNER,
//     SIGNED_IN,
//     GUEST,
// }

// struct CreateNoteOptions {
//     title: Option<String>,
//     content: Option<String>,
//     read_permission: Option<NotePermissionRole>,
//     write_permission: Option<NotePermissionRole>,
//     comment_permission: Option<CommentPermissionType>,
//     permalink: Option<String>,
// }

#[derive(Debug, PartialEq, Eq, Deserialize)]
struct Team {
    id: String,
    owner_id: String,
    name: String,
    logo: String,
    path: String,
    description: String,
    hard_breaks: bool,
    visibility: TeamVisibilityType,
    created_at: NaiveDateTime,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct User {
    id: String,
    email: Option<String>,
    name: String,
    user_path: String,
    photo: String,
    teams: Vec<Team>,
}

// struct SimpleUserProfile {
//     name: String,
//     user_path: String,
//     photo: String,
//     biography: Option<String>,
//     created_at: NaiveDateTime,
// }

// struct Note {
//     id: String,
//     title: String,
//     tags: Vec<String>,
//     last_changed_at: String,
//     created_at: String,
//     last_change_user: Option<SimpleUserProfile>,
//     publish_type: NotePublishType,
//     published_at: Option<String>,
//     user_path: Option<String>,
//     team_path: Option<String>,
//     permalink: Option<String>,
//     short_id: String,
//     publish_link: String,

//     read_permission: NotePermissionRole,
//     write_permission: NotePermissionRole,
// }

// struct SingleNote {
//     content: String,
//     // Include fields from Note struct here
// }
