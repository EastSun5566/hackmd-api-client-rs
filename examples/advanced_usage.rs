use hackmd_api_client_rs::{
    ApiClient, ApiClientOptions, CommentPermissionType, CreateNoteOptions, NotePermissionRole,
    RetryOptions, UpdateNoteOptions,
};
use std::{error, time};

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    // Example with custom configuration
    let options = ApiClientOptions {
        wrap_response_errors: true,
        timeout: Some(time::Duration::from_secs(30)),
        retry_options: Some(RetryOptions {
            max_retries: 3,
            base_delay: time::Duration::from_millis(200),
        }),
    };

    let client = ApiClient::with_options(
        "your_access_token_here",
        None, // Use default API endpoint
        Some(options),
    )?;

    println!("=== HackMD API Client Advanced Example ===\n");

    // 1. Get user information
    println!("📋 Getting user information...");
    match client.get_me().await {
        Ok(user) => {
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
        }
        Err(e) => {
            eprintln!("❌ Error getting user info: {}", e);
            return Ok(());
        }
    }

    println!();

    // 2. Create a new note with various permissions
    println!("📝 Creating a new note...");
    let note_options = CreateNoteOptions {
        title: Some("Advanced Rust Example".to_string()),
        content: Some("# Advanced HackMD Rust Client Example\n\n## Features\n\n- ✅ Async/await support\n- ✅ Retry mechanism with exponential backoff\n- ✅ Comprehensive error handling\n- ✅ Type-safe API\n\n## Code\n\n```rust\nlet client = ApiClient::new(\"token\")?;\nlet user = client.get_me().await?;\nprintln!(\"Hello, {}!\", user.name);\n```\n\nThis note was created using the Rust HackMD API client! 🦀".to_string()),
        read_permission: Some(NotePermissionRole::SignedIn),
        write_permission: Some(NotePermissionRole::Owner),
        comment_permission: Some(CommentPermissionType::Owners),
        permalink: Some(format!("rust-example-{}", chrono::Utc::now().timestamp())),
    };

    let created_note = match client.create_note(&note_options).await {
        Ok(note) => {
            println!(
                "✅ Created note: {} (ID: {})",
                note.note.title, note.note.id
            );
            println!("   Short ID: {}", note.note.short_id);
            println!("   Publish Link: {}", note.note.publish_link);
            note
        }
        Err(e) => {
            eprintln!("❌ Error creating note: {}", e);
            return Ok(());
        }
    };

    println!();

    // 3. Update the note content
    println!("✏️  Updating note content...");
    let updated_content = format!(
        "{}\n\n## Update\n\nThis content was updated at {} using the Rust API client.\n\n### Statistics\n\n- Original content length: {} characters\n- Update timestamp: {}\n- API client: hackmd-api-client-rs v0.1.0",
        created_note.content,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        created_note.content.len(),
        chrono::Utc::now().timestamp()
    );

    match client
        .update_note_content(&created_note.note.id, &updated_content)
        .await
    {
        Ok(_) => println!("✅ Note content updated successfully"),
        Err(e) => eprintln!("❌ Error updating note: {}", e),
    }

    println!();

    // 4. Update note permissions
    println!("🔒 Updating note permissions...");
    let permission_update = UpdateNoteOptions {
        content: None,
        read_permission: Some(NotePermissionRole::Guest), // Make it publicly readable
        write_permission: Some(NotePermissionRole::SignedIn),
        permalink: None,
    };

    match client
        .update_note(&created_note.note.id, &permission_update)
        .await
    {
        Ok(_) => println!("✅ Note permissions updated successfully"),
        Err(e) => eprintln!("❌ Error updating permissions: {}", e),
    }

    println!();

    // 5. Get updated note
    println!("🔍 Fetching updated note...");
    match client.get_note(&created_note.note.id).await {
        Ok(note) => {
            println!("✅ Fetched note: {}", note.note.title);
            println!("   Content length: {} characters", note.content.len());
            println!("   Read permission: {:?}", note.note.read_permission);
            println!("   Write permission: {:?}", note.note.write_permission);
            println!("   Last changed: {}", note.note.last_changed_at);
        }
        Err(e) => eprintln!("❌ Error fetching note: {}", e),
    }

    println!();

    // 6. List recent notes
    println!("📚 Getting note list...");
    match client.get_note_list().await {
        Ok(notes) => {
            println!("✅ Found {} notes", notes.len());
            println!("   Recent notes:");
            for note in notes.iter().take(5) {
                println!("     - {} ({})", note.title, note.short_id);
                println!("       Tags: {}", note.tags.join(", "));
                println!("       Last changed: {}", note.last_changed_at);
            }
        }
        Err(e) => eprintln!("❌ Error getting notes: {}", e),
    }

    println!();

    // 7. Get teams and team notes (if any)
    println!("👥 Getting teams...");
    match client.get_teams().await {
        Ok(teams) => {
            if teams.is_empty() {
                println!("ℹ️  No teams found");
            } else {
                println!("✅ Found {} teams", teams.len());
                for team in &teams {
                    println!("   - Team: {} ({})", team.name, team.path);
                    println!("     Description: {}", team.description);

                    // Get team notes
                    match client.get_team_notes(&team.path).await {
                        Ok(team_notes) => {
                            println!("     Notes: {} notes", team_notes.len());
                            for note in team_notes.iter().take(3) {
                                println!("       - {}", note.title);
                            }
                        }
                        Err(e) => eprintln!("     ❌ Error getting team notes: {}", e),
                    }
                }
            }
        }
        Err(e) => eprintln!("❌ Error getting teams: {}", e),
    }

    println!("\n🎉 Advanced example completed!");
    println!("   Created note ID: {}", created_note.note.id);
    println!("   View at: {}", created_note.note.publish_link);

    Ok(())
}
