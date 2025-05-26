use hackmd_api_client_rs::{ApiClient, CreateNoteOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create API client
    let client: ApiClient = ApiClient::new("<YOUR_ACCESS_TOKEN>")?;

    // Get user information
    match client.get_me().await {
        Ok(user) => {
            println!("User: {} ({})", user.name, user.email.unwrap_or_default());
            println!("User path: {}", user.user_path);
            println!("Teams: {}", user.teams.len());
            for team in &user.teams {
                println!("     - {} ({})", team.name, team.path);
            }
        }
        Err(e) => {
            eprintln!("Error getting user info: {:?}", e);
        }
    }

    // Create a new note
    let note_options = CreateNoteOptions {
        title: Some("Test Note from Rust API Client".to_string()),
        content: Some(
            "# Hello from Rust API Client\n\nThis note was created using the Rust HackMD API client."
                .to_string(),
        ),
        read_permission: None,
        write_permission: None,
        comment_permission: None,
        permalink: None,
    };
    match client.create_note(&note_options).await {
        Ok(note) => {
            println!("Created note: {} (ID: {})", note.note.title, note.note.id);

            // Update note content
            let updated_content =
                "# Updated from Rust API client\n\nThis content has been updated!";
            match client
                .update_note_content(&note.note.id, updated_content)
                .await
            {
                Ok(_) => println!("Note content updated successfully"),
                Err(e) => eprintln!("Error updating note: {:?}", e),
            }

            // Delete the note
            match client.delete_note(&note.note.id).await {
                Ok(_) => println!("Note deleted successfully"),
                Err(e) => eprintln!("Error deleting note: {:?}", e),
            }
        }
        Err(e) => {
            eprintln!("Error creating note: {:?}", e);
        }
    }

    // Get notes list
    match client.get_note_list().await {
        Ok(notes) => {
            println!("Found {} notes", notes.len());
            for note in notes.iter().take(5) {
                println!("  - {} ({})", note.title, note.id);
            }
        }
        Err(e) => {
            eprintln!("Error getting notes: {:?}", e);
        }
    }

    Ok(())
}
