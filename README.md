# HackMD Rust API Client

[![crates.io](https://img.shields.io/crates/v/hackmd-api-client-rs.svg?style=for-the-badge)](https://crates.io/crates/hackmd-api-client-rs)
[![docs](https://img.shields.io/docsrs/hackmd-api-client-rs?style=for-the-badge)](https://docs.rs/hackmd-api-client-rs)

> 🦀📝 A Rust client library for the [HackMD API](https://hackmd.io/@docs/HackMD_API_Book).

You can sign up for an account at [hackmd.io](https://hackmd.io/), and then create access tokens by following the [developer portal](https://hackmd.io/@hackmd-api/developer-portal).

## Features

- ✅ Complete API coverage (User, Notes, Teams, Image Upload)
- ✅ Async/await support with `tokio`
- ✅ Retry mechanism with exponential backoff
- ✅ Comprehensive error handling & Type-safe request/response

## Installation

```bash
cargo add hackmd-api-client-rs
```

Set `HACKMD_ACCESS_TOKEN` before running the examples or your own binaries:

```bash
export HACKMD_ACCESS_TOKEN=<YOUR_ACCESS_TOKEN>
```

## Quick Start

```rust
use hackmd_api_client_rs::{ApiClient, CreateNoteOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let access_token = std::env::var("HACKMD_ACCESS_TOKEN")?;
    let client = ApiClient::new(&access_token)?;

    let user = client.get_me().await?;
    println!("User: {} ({})", user.name, user.email.unwrap_or_default());

    let note = client
        .create_note(&CreateNoteOptions {
            title: Some("My First Note".to_string()),
            content: Some("# Hello World\n\nThis is my first note!".to_string()),
            tags: Some(vec!["rust".to_string()]),
            ..Default::default()
        })
        .await?;

    println!("Created note: {} (ID: {})", note.note.title, note.note.id);

    Ok(())
}
```

## Configuration

You can customize the client behavior with `ApiClientOptions`:

```rust
use hackmd_api_client_rs::{ApiClient, ApiClientOptions, RetryOptions};
use std::time::Duration;

let options = ApiClientOptions {
    wrap_response_errors: true, // Convert HTTP errors to custom error types
    timeout: Some(Duration::from_secs(30)), // Request timeout
    retry_options: Some(RetryOptions {
        max_retries: 3,
        base_delay: Duration::from_millis(100),
    }),
};

let access_token = std::env::var("HACKMD_ACCESS_TOKEN")?;
let client = ApiClient::with_options(&access_token, None, Some(options))?;
```

Use `with_base_url()` when targeting a self-hosted HackMD deployment. A trailing slash is optional:

```rust
let access_token = std::env::var("HACKMD_ACCESS_TOKEN")?;
let client = ApiClient::with_base_url(&access_token, "https://your-hackmd.example/api/v1")?;
```

## API Methods

### User API

- `get_me()` - Get current user information
- `get_history(limit)` - Get user's note history (`limit` is `Option<u32>`)
- `get_note_list()` - Get user's notes
- `get_note(note_id)` - Get a specific note
- `create_note(options)` - Create a new note
- `update_note(note_id, options)` - Update a note
- `update_note_content(note_id, content)` - Update note content only
- `delete_note(note_id)` - Delete a note
- `upload_note_image(note_id, image_bytes, file_name, mime_type)` - Upload an image for a note

### Team API

- `get_teams()` - Get user's teams
- `get_team_notes(team_path)` - Get team's notes
- `create_team_note(team_path, options)` - Create a team note
- `update_team_note(team_path, note_id, options)` - Update a team note
- `update_team_note_content(team_path, note_id, content)` - Update team note content
- `delete_team_note(team_path, note_id)` - Delete a team note

## Error Handling

The client provides comprehensive error handling with custom error types:

```rust
use hackmd_api_client_rs::error::ApiError;

match client.get_me().await {
    Ok(user) => println!("User: {}", user.name),
    Err(ApiError::TooManyRequests(err)) => {
        println!(
            "Rate limited: {}/{} requests remaining",
            err.user_remaining, err.user_limit
        );
    }
    Err(ApiError::InternalServer(err)) => {
        println!("Server error: {}", err.message);
    }
    Err(err) => println!("Other error: {}", err),
}
```

## Examples

The examples read `HACKMD_ACCESS_TOKEN` from the environment. A `.env.example` template is included if you prefer to keep a local placeholder file.

Run the basic usage example:

```bash
cargo run --example basic_usage
```

Advanced usage example:

```bash
cargo run --example advanced_usage
```

## Types

All API types are available in the `types` module:

- `User` - User information
- `Team` - Team information (`owner_id`, `visibility`, etc.)
- `Note` - Note metadata (includes `description`, `tags`, `folder_paths`, `title_updated_at`, `tags_updated_at`)
- `SingleNote` - Note with full content
- `FolderPath` - Folder path entry for note folder organisation
- `SimpleUserProfile` - Minimal user profile (used in `Note.last_change_user`)
- `CreateNoteOptions` - Options for creating notes (title, content, description, tags, permissions, `parent_folder_id`, `origin`, etc.)
- `UpdateNoteOptions` - Options for updating notes (title, content, description, tags, permissions, `parent_folder_id`)
- `NoteImageUploadResponse` - Response from the image upload endpoint
- `NotePermissionRole` - `owner` | `signed_in` | `guest`
- `NotePublishType` - `edit` | `view` | `slide` | `book`
- `CommentPermissionType` - `disabled` | `forbidden` | `owners` | `signed_in_users` | `everyone`
- `SuggestEditPermissionType` - `disabled` | `forbidden` | `owners` | `signed_in_users`
- `TeamVisibilityType` - `public` | `private`

## Release

```bash
just preview X.Y.Z
just prep X.Y.Z
just commit X.Y.Z
just push
```
