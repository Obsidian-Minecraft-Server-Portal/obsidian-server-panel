use modrinth::{ModrinthClient, SearchBuilder};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn search_result_json() -> serde_json::Value {
    serde_json::from_str(include_str!("../test_fixtures/search_result.json")).unwrap()
}

fn project_json() -> serde_json::Value {
    serde_json::from_str(include_str!("../test_fixtures/project.json")).unwrap()
}

fn versions_json() -> serde_json::Value {
    serde_json::from_str(include_str!("../test_fixtures/versions.json")).unwrap()
}

fn categories_json() -> serde_json::Value {
    serde_json::from_str(include_str!("../test_fixtures/categories.json")).unwrap()
}

fn game_versions_json() -> serde_json::Value {
    serde_json::from_str(include_str!("../test_fixtures/game_versions.json")).unwrap()
}

fn loaders_json() -> serde_json::Value {
    serde_json::from_str(include_str!("../test_fixtures/loaders.json")).unwrap()
}

#[tokio::test]
async fn test_search() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v2/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(search_result_json()))
        .mount(&mock_server)
        .await;

    let client = ModrinthClient::with_base_url(format!("{}/v2", mock_server.uri()));
    let params = SearchBuilder::new().query("sodium").limit(10).build();
    let result = client.search(&params).await.unwrap();

    assert_eq!(result.total_hits, 2);
    assert_eq!(result.hits.len(), 2);
    assert_eq!(result.hits[0].title, "Sodium");
    assert_eq!(result.hits[0].project_id, "AANobbMI");
}

#[tokio::test]
async fn test_search_with_facets() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v2/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(search_result_json()))
        .mount(&mock_server)
        .await;

    let client = ModrinthClient::with_base_url(format!("{}/v2", mock_server.uri()));
    let params = SearchBuilder::new()
        .query("sodium")
        .project_type("mod")
        .versions(&["1.20.1"])
        .loaders(&["fabric"])
        .server_side()
        .limit(10)
        .build();

    let result = client.search(&params).await.unwrap();
    assert_eq!(result.total_hits, 2);
}

#[tokio::test]
async fn test_get_project() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v2/project/sodium"))
        .respond_with(ResponseTemplate::new(200).set_body_json(project_json()))
        .mount(&mock_server)
        .await;

    let client = ModrinthClient::with_base_url(format!("{}/v2", mock_server.uri()));
    let project = client.get_project("sodium").await.unwrap();

    assert_eq!(project.id, "AANobbMI");
    assert_eq!(project.slug, "sodium");
    assert_eq!(project.title, "Sodium");
    assert_eq!(project.downloads, 50000000);
    assert!(project.license.is_some());
    assert_eq!(project.license.unwrap().id, "LGPL-3.0-only");
    assert_eq!(project.gallery.len(), 1);
    assert_eq!(project.donation_urls.as_ref().unwrap().len(), 1);
}

#[tokio::test]
async fn test_get_project_versions() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v2/project/AANobbMI/version"))
        .respond_with(ResponseTemplate::new(200).set_body_json(versions_json()))
        .mount(&mock_server)
        .await;

    let client = ModrinthClient::with_base_url(format!("{}/v2", mock_server.uri()));
    let versions = client.get_project_versions("AANobbMI").await.unwrap();

    assert_eq!(versions.len(), 2);
    assert_eq!(versions[0].name, "Sodium 0.5.8");
    assert_eq!(versions[0].version_type, "release");
    assert_eq!(versions[0].files.len(), 1);
    assert!(versions[0].files[0].primary);
    assert_eq!(versions[0].dependencies.len(), 1);
    assert_eq!(versions[0].dependencies[0].dependency_type, "required");
}

#[tokio::test]
async fn test_get_categories() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v2/tag/category"))
        .respond_with(ResponseTemplate::new(200).set_body_json(categories_json()))
        .mount(&mock_server)
        .await;

    let client = ModrinthClient::with_base_url(format!("{}/v2", mock_server.uri()));
    let categories = client.get_categories().await.unwrap();

    assert_eq!(categories.len(), 4);
    assert_eq!(categories[0].name, "adventure");
    assert_eq!(categories[1].name, "optimization");
}

#[tokio::test]
async fn test_get_game_versions() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v2/tag/game_version"))
        .respond_with(ResponseTemplate::new(200).set_body_json(game_versions_json()))
        .mount(&mock_server)
        .await;

    let client = ModrinthClient::with_base_url(format!("{}/v2", mock_server.uri()));
    let versions = client.get_game_versions().await.unwrap();

    assert_eq!(versions.len(), 4);
    assert_eq!(versions[0].version, "1.21.4");
    assert!(versions[0].major);
    assert_eq!(versions[3].version_type, "snapshot");
}

#[tokio::test]
async fn test_get_loaders() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v2/tag/loader"))
        .respond_with(ResponseTemplate::new(200).set_body_json(loaders_json()))
        .mount(&mock_server)
        .await;

    let client = ModrinthClient::with_base_url(format!("{}/v2", mock_server.uri()));
    let loaders = client.get_loaders().await.unwrap();

    assert_eq!(loaders.len(), 4);
    assert_eq!(loaders[0].name, "fabric");
    assert!(loaders[0].supported_project_types.contains(&"mod".to_string()));
}

#[tokio::test]
async fn test_caching_prevents_duplicate_requests() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v2/project/sodium"))
        .respond_with(ResponseTemplate::new(200).set_body_json(project_json()))
        .expect(1) // Should only be called once due to caching
        .mount(&mock_server)
        .await;

    let client = ModrinthClient::with_base_url(format!("{}/v2", mock_server.uri()));

    // First call - hits the API
    let project1 = client.get_project("sodium").await.unwrap();
    // Second call - should come from cache
    let project2 = client.get_project("sodium").await.unwrap();

    assert_eq!(project1.id, project2.id);
    assert_eq!(project1.title, project2.title);
}

#[tokio::test]
async fn test_clear_cache_allows_fresh_request() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v2/project/sodium"))
        .respond_with(ResponseTemplate::new(200).set_body_json(project_json()))
        .expect(2) // Should be called twice since we clear the cache
        .mount(&mock_server)
        .await;

    let client = ModrinthClient::with_base_url(format!("{}/v2", mock_server.uri()));

    let _ = client.get_project("sodium").await.unwrap();
    client.clear_cache().await;
    let _ = client.get_project("sodium").await.unwrap();
}

#[tokio::test]
async fn test_api_error_handling() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v2/project/nonexistent"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not found"))
        .mount(&mock_server)
        .await;

    let client = ModrinthClient::with_base_url(format!("{}/v2", mock_server.uri()));
    let result = client.get_project("nonexistent").await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        modrinth::ModrinthError::Api { status, message } => {
            assert_eq!(status, 404);
            assert_eq!(message, "Not found");
        }
        _ => panic!("Expected Api error, got: {err:?}"),
    }
}

#[tokio::test]
async fn test_rate_limit_handling() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v2/project/test"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("x-ratelimit-reset", "30"),
        )
        .mount(&mock_server)
        .await;

    let client = ModrinthClient::with_base_url(format!("{}/v2", mock_server.uri()));
    let result = client.get_project("test").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        modrinth::ModrinthError::RateLimited { retry_after_ms } => {
            assert_eq!(retry_after_ms, 30000);
        }
        err => panic!("Expected RateLimited error, got: {err:?}"),
    }
}

#[tokio::test]
async fn test_model_deserialization_from_fixtures() {
    // Test that all fixture files deserialize correctly
    let _: modrinth::models::SearchResult =
        serde_json::from_str(include_str!("../test_fixtures/search_result.json")).unwrap();
    let _: modrinth::models::Project =
        serde_json::from_str(include_str!("../test_fixtures/project.json")).unwrap();
    let _: Vec<modrinth::models::Version> =
        serde_json::from_str(include_str!("../test_fixtures/versions.json")).unwrap();
    let _: Vec<modrinth::models::Category> =
        serde_json::from_str(include_str!("../test_fixtures/categories.json")).unwrap();
    let _: Vec<modrinth::models::GameVersion> =
        serde_json::from_str(include_str!("../test_fixtures/game_versions.json")).unwrap();
    let _: Vec<modrinth::models::Loader> =
        serde_json::from_str(include_str!("../test_fixtures/loaders.json")).unwrap();
}

#[tokio::test]
async fn test_get_projects_batch() {
    let mock_server = MockServer::start().await;

    let projects = serde_json::json!([
        {
            "id": "AANobbMI",
            "slug": "sodium",
            "title": "Sodium",
            "description": "Fast rendering",
            "body": "",
            "categories": [],
            "client_side": "required",
            "server_side": "unsupported",
            "project_type": "mod",
            "downloads": 50000000,
            "followers": 250000,
            "versions": [],
            "game_versions": [],
            "loaders": [],
            "published": "2021-01-01T00:00:00Z",
            "updated": "2024-06-15T12:00:00Z",
            "gallery": [],
            "team": "team1",
            "status": "approved"
        }
    ]);

    Mock::given(method("GET"))
        .and(path("/v2/projects"))
        .respond_with(ResponseTemplate::new(200).set_body_json(projects))
        .mount(&mock_server)
        .await;

    let client = ModrinthClient::with_base_url(format!("{}/v2", mock_server.uri()));
    let result = client.get_projects(&["AANobbMI"]).await.unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].title, "Sodium");
}
