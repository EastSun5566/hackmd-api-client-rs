use hackmd_api_client_rs::{ApiClient, ApiError, CreateNoteOptions, UpdateNoteOptions};
use serde_json::json;
use std::collections::BTreeMap;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn sample_single_note_response(
    team_path: Option<&str>,
    title: &str,
    content: &str,
) -> serde_json::Value {
    json!({
        "id": "note-123",
        "title": title,
        "description": "",
        "tags": [],
        "lastChangedAt": 1_710_000_000_000i64,
        "createdAt": 1_710_000_000_000i64,
        "titleUpdatedAt": null,
        "tagsUpdatedAt": null,
        "lastChangeUser": null,
        "publishType": "edit",
        "publishedAt": null,
        "userPath": if team_path.is_some() { None::<&str> } else { Some("demo-user") },
        "teamPath": team_path,
        "permalink": null,
        "shortId": "short-123",
        "publishLink": "https://hackmd.io/note-123",
        "folderPaths": [],
        "readPermission": "owner",
        "writePermission": "owner",
        "content": content,
    })
}

#[tokio::test]
async fn get_team_note_uses_team_note_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/teams/platform-team/notes/note-123"))
        .and(header("authorization", "Bearer test-token"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(sample_single_note_response(
                Some("platform-team"),
                "Team Note",
                "# Team note",
            )),
        )
        .mount(&server)
        .await;

    let client = ApiClient::with_base_url("test-token", &server.uri()).unwrap();
    let note = client
        .get_team_note("platform-team", "note-123")
        .await
        .unwrap();

    assert_eq!(note.note.id, "note-123");
    assert_eq!(note.note.team_path.as_deref(), Some("platform-team"));
    assert_eq!(note.content, "# Team note");
}

#[tokio::test]
async fn create_note_content_sends_json_string_body() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/notes"))
        .and(header("authorization", "Bearer test-token"))
        .and(body_json(json!("# Markdown content")))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(sample_single_note_response(
                None,
                "Markdown Content Note",
                "# Markdown content",
            )),
        )
        .mount(&server)
        .await;

    let client = ApiClient::with_base_url("test-token", &server.uri()).unwrap();
    let note = client
        .create_note_content("# Markdown content")
        .await
        .unwrap();

    assert_eq!(note.note.title, "Markdown Content Note");
    assert_eq!(note.content, "# Markdown content");
}

#[tokio::test]
async fn create_note_serializes_note_features() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/notes"))
        .and(header("authorization", "Bearer test-token"))
        .and(body_json(json!({
            "title": "Feature Note",
            "content": "# Feature note",
            "noteFeatures": {
                "experimentalFeature": {
                    "enabled": true,
                    "scope": "team"
                }
            }
        })))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(sample_single_note_response(
                None,
                "Feature Note",
                "# Feature note",
            )),
        )
        .mount(&server)
        .await;

    let client = ApiClient::with_base_url("test-token", &server.uri()).unwrap();
    let payload = CreateNoteOptions {
        title: Some("Feature Note".to_string()),
        content: Some("# Feature note".to_string()),
        note_features: Some(BTreeMap::from([(
            "experimentalFeature".to_string(),
            json!({
                "enabled": true,
                "scope": "team"
            }),
        )])),
        ..Default::default()
    };

    let note = client.create_note(&payload).await.unwrap();
    assert_eq!(note.note.title, "Feature Note");
}

#[tokio::test]
async fn create_team_note_content_sends_json_string_body() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/teams/platform-team/notes"))
        .and(header("authorization", "Bearer test-token"))
        .and(body_json(json!("# Team Markdown content")))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(sample_single_note_response(
                Some("platform-team"),
                "Team Markdown Content Note",
                "# Team Markdown content",
            )),
        )
        .mount(&server)
        .await;

    let client = ApiClient::with_base_url("test-token", &server.uri()).unwrap();
    let note = client
        .create_team_note_content("platform-team", "# Team Markdown content")
        .await
        .unwrap();

    assert_eq!(note.note.team_path.as_deref(), Some("platform-team"));
    assert_eq!(note.content, "# Team Markdown content");
}

#[tokio::test]
async fn update_note_accepts_latest_accepted_status() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/notes/note-123"))
        .and(header("authorization", "Bearer test-token"))
        .and(body_json(json!({
            "content": "# Updated content"
        })))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({})))
        .mount(&server)
        .await;

    let client = ApiClient::with_base_url("test-token", &server.uri()).unwrap();
    client
        .update_note(
            "note-123",
            &UpdateNoteOptions {
                content: Some("# Updated content".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn error_response_keeps_body_detail() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/me"))
        .and(header("authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(401).set_body_string("invalid token"))
        .mount(&server)
        .await;

    let client = ApiClient::with_base_url("test-token", &server.uri()).unwrap();
    let error = client.get_me().await.unwrap_err();

    match error {
        ApiError::HttpResponse(error) => assert!(error.message.contains("invalid token")),
        error => panic!("expected HTTP response error, got {error:?}"),
    }
}
