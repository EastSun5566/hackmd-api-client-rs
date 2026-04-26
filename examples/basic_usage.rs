use hackmd_api_client_rs::{ApiClient, CreateNoteOptions};
use std::{env, error, io};

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
    let client = ApiClient::new(&access_token)?;

    let user = client.get_me().await?;
    println!("User: {} ({})", user.name, user.email.unwrap_or_default());
    println!("User path: {}", user.user_path);
    println!("Teams: {}", user.teams.len());
    for team in &user.teams {
        println!("  - {} ({})", team.name, team.path);
    }

    let note = client
        .create_note(&CreateNoteOptions {
            title: Some("Test Note from Rust API Client".to_string()),
            content: Some(
                "# Hello from Rust API Client\n\nThis note was created using the Rust HackMD API client."
                    .to_string(),
            ),
            ..Default::default()
        })
        .await?;
    println!("Created note: {} (ID: {})", note.note.title, note.note.id);

    client
        .update_note_content(
            &note.note.id,
            "# Updated from Rust API client\n\nThis content has been updated!",
        )
        .await?;
    println!("Note content updated successfully");

    client.delete_note(&note.note.id).await?;
    println!("Note deleted successfully");

    let notes = client.get_note_list().await?;
    println!("Found {} notes", notes.len());
    for note in notes.iter().take(5) {
        println!("  - {} ({})", note.title, note.id);
    }

    Ok(())
}
