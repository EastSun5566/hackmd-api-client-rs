use hackmd_api_client_rs::{
    ApiClient, ApiClientOptions, CommentPermissionType, CreateNoteOptions, NotePermissionRole,
    RetryOptions, UpdateNoteOptions,
};
use std::{env, error, io, time};

fn read_access_token() -> Result<String, io::Error> {
    env::var("HACKMD_ACCESS_TOKEN").map_err(|_| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Set HACKMD_ACCESS_TOKEN before running this example",
        )
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let access_token = read_access_token()?;
    let options = ApiClientOptions {
        wrap_response_errors: true,
        timeout: Some(time::Duration::from_secs(30)),
        retry_options: Some(RetryOptions {
            max_retries: 3,
            base_delay: time::Duration::from_millis(200),
        }),
    };

    let client = ApiClient::with_options(&access_token, None, Some(options))?;

    println!("=== HackMD API Client Advanced Example ===\n");

    println!("📋 Getting user information...");
    let user = client.get_me().await?;
    println!(
        "✅ User: {} ({})",
        user.name,
        user.email.unwrap_or("no email".to_string())
    );
    println!("   Path: {}", user.user_path);
    println!("   Teams: {}", user.teams.len());
    for team in &user.teams {
        println!("     - {} ({})", team.name, team.path);
    }

    println!();

    println!("📝 Creating a new note...");
    let note_options = CreateNoteOptions {
        title: Some("Advanced Rust Example".to_string()),
        content: Some("# Advanced HackMD Rust Client Example\n\n## Features\n\n- ✅ Async/await support\n- ✅ Retry mechanism with exponential backoff\n- ✅ Comprehensive error handling\n- ✅ Type-safe API\n\n## Code\n\n```rust\nlet client = ApiClient::new(\"token\")?;\nlet user = client.get_me().await?;\nprintln!(\"Hello, {}!\", user.name);\n```\n\nThis note was created using the Rust HackMD API client! 🦀".to_string()),
        read_permission: Some(NotePermissionRole::SignedIn),
        write_permission: Some(NotePermissionRole::Owner),
        comment_permission: Some(CommentPermissionType::Owners),
        permalink: Some(format!("rust-example-{}", chrono::Utc::now().timestamp())),
        ..Default::default()
    };

    let created_note = client.create_note(&note_options).await?;
    println!(
        "✅ Created note: {} (ID: {})",
        created_note.note.title, created_note.note.id
    );
    println!("   Short ID: {}", created_note.note.short_id);
    println!("   Publish Link: {}", created_note.note.publish_link);

    println!();

    println!("✏️  Updating note content...");
    let updated_content = format!(
        "{}\n\n## Update\n\nThis content was updated at {} using the Rust API client.\n\n### Statistics\n\n- Original content length: {} characters\n- Update timestamp: {}\n- API client: hackmd-api-client-rs v{}",
        created_note.content,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        created_note.content.len(),
        chrono::Utc::now().timestamp(),
        env!("CARGO_PKG_VERSION")
    );

    client
        .update_note_content(&created_note.note.id, &updated_content)
        .await?;
    println!("✅ Note content updated successfully");

    println!();

    println!("🔒 Updating note permissions...");
    let permission_update = UpdateNoteOptions {
        read_permission: Some(NotePermissionRole::Guest),
        write_permission: Some(NotePermissionRole::SignedIn),
        ..Default::default()
    };

    client
        .update_note(&created_note.note.id, &permission_update)
        .await?;
    println!("✅ Note permissions updated successfully");

    println!();

    println!("🔍 Fetching updated note...");
    let note = client.get_note(&created_note.note.id).await?;
    println!("✅ Fetched note: {}", note.note.title);
    println!("   Content length: {} characters", note.content.len());
    println!("   Read permission: {:?}", note.note.read_permission);
    println!("   Write permission: {:?}", note.note.write_permission);
    println!("   Last changed: {}", note.note.last_changed_at);

    println!();

    println!("📚 Getting note list...");
    let notes = client.get_note_list().await?;
    println!("✅ Found {} notes", notes.len());
    println!("   Recent notes:");
    for note in notes.iter().take(5) {
        println!("     - {} ({})", note.title, note.short_id);
        println!("       Tags: {}", note.tags.join(", "));
        println!("       Last changed: {}", note.last_changed_at);
    }

    println!();

    println!("👥 Getting teams...");
    let teams = client.get_teams().await?;
    if teams.is_empty() {
        println!("ℹ️  No teams found");
    } else {
        println!("✅ Found {} teams", teams.len());
        for team in &teams {
            println!("   - Team: {} ({})", team.name, team.path);
            println!(
                "     Description: {}",
                team.description.as_deref().unwrap_or("")
            );

            match client.get_team_notes(&team.path).await {
                Ok(team_notes) => {
                    println!("     Notes: {} notes", team_notes.len());
                    for note in team_notes.iter().take(3) {
                        println!("       - {}", note.title);
                    }
                }
                Err(error) => eprintln!("     ❌ Error getting team notes: {:?}", error),
            }
        }
    }

    println!("\n🎉 Advanced example completed!");
    println!("   Created note ID: {}", created_note.note.id);
    println!("   View at: {}", created_note.note.publish_link);

    Ok(())
}
