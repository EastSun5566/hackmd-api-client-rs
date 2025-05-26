# HackMD Rust API Client

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)

A Rust client library for the [HackMD API](https://hackmd.io/@docs/HackMD_API_Book).

You can sign up for an account at [hackmd.io](https://hackmd.io/), and then create access tokens for your projects by following the [HackMD API documentation](https://hackmd.io/@hackmd-api/developer-portal).

## Features

- ✅ Complete API coverage (User, Notes, Teams)
- ✅ Async/await support with `tokio`
- ✅ Automatic retry with exponential backoff
- ✅ Rate limiting handling
- ✅ Comprehensive error handling
- ✅ Type-safe request/response models
- ✅ Configurable timeouts and retry policies

## Installation

```bash
cargo add hackmd-api-client-rs
```

## Quick Start

```rust
use hackmd_api_client_rs::{ApiClient, ApiClientOptions};
use hackmd_api_client_rs::types::CreateNoteOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create API client
    let client = ApiClient::new(
        "your_access_token_here",
        None, // Use default API endpoint
        Some(ApiClientOptions::default()),
    )?;

    // Get user information
    let user = client.get_me().await?;
    println!("User: {} ({})", user.name, user.email.unwrap_or_default());

    // Create a new note
    let note_options = CreateNoteOptions {
        title: Some("My First Note".to_string()),
        content: Some("# Hello World\\n\\nThis is my first note!".to_string()),
        read_permission: None,
        write_permission: None,
        comment_permission: None,
        permalink: None,
    };

    let note = client.create_note(&note_options).await?;
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

let client = ApiClient::new("your_token", None, Some(options))?;
```

## API Methods

### User API

- `get_me()` - Get current user information
- `get_history()` - Get user's note history
- `get_note_list()` - Get user's notes
- `get_note(note_id)` - Get a specific note
- `create_note(options)` - Create a new note
- `update_note(note_id, options)` - Update a note
- `update_note_content(note_id, content)` - Update note content only
- `delete_note(note_id)` - Delete a note

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
        println!("Rate limited: {}/{} requests remaining",
                err.user_remaining, err.user_limit);
    },
    Err(ApiError::InternalServer(err)) => {
        println!("Server error: {}", err.message);
    },
    Err(err) => println!("Other error: {}", err),
}
```

## Examples

Run the basic usage example:

```bash
cargo run --example basic_usage
```

Make sure to set your HackMD access token in the example code before running.

## Types

All API types are available in the `types` module:

- `User` - User information
- `Team` - Team information
- `Note` - Note metadata
- `SingleNote` - Note with content
- `CreateNoteOptions` - Options for creating notes
- `UpdateNoteOptions` - Options for updating notes
- Various enums for permissions and visibility

## License

MIT License
