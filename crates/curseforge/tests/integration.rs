use curseforge::{CurseForgeClient, SearchBuilder};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn search_result_json() -> serde_json::Value {
    serde_json::from_str(include_str!("../test_fixtures/search_result.json")).unwrap()
}

fn mod_json() -> serde_json::Value {
    serde_json::from_str(include_str!("../test_fixtures/mod.json")).unwrap()
}

fn files_json() -> serde_json::Value {
    serde_json::from_str(include_str!("../test_fixtures/files.json")).unwrap()
}

fn categories_json() -> serde_json::Value {
    serde_json::from_str(include_str!("../test_fixtures/categories.json")).unwrap()
}

#[tokio::test]
async fn test_search_mods() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/mods/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(search_result_json()))
        .mount(&mock_server)
        .await;

    let client = CurseForgeClient::with_base_url("test-key", format!("{}/v1", mock_server.uri()));
    let params = SearchBuilder::new().query("sodium").page_size(20).build();
    let result = client.search(&params).await.unwrap();

    assert_eq!(result.pagination.total_count, 2);
    assert_eq!(result.data.len(), 2);
    assert_eq!(result.data[0].name, "Sodium");
    assert_eq!(result.data[0].id, 394468);
}

#[tokio::test]
async fn test_search_modpacks() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/mods/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(search_result_json()))
        .mount(&mock_server)
        .await;

    let client = CurseForgeClient::with_base_url("test-key", format!("{}/v1", mock_server.uri()));
    let params = SearchBuilder::new().query("all the mods").page_size(10).build();
    let result = client.search_modpacks(&params).await.unwrap();

    assert_eq!(result.pagination.total_count, 2);
    assert_eq!(result.data.len(), 2);
}

#[tokio::test]
async fn test_search_with_filters() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/mods/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(search_result_json()))
        .mount(&mock_server)
        .await;

    let client = CurseForgeClient::with_base_url("test-key", format!("{}/v1", mock_server.uri()));
    let params = SearchBuilder::new()
        .query("sodium")
        .game_version("1.20.1")
        .mod_loader_type(4) // Fabric
        .category_id(423)
        .sort_field(2)
        .sort_order("desc")
        .page_size(20)
        .index(0)
        .build();

    let result = client.search(&params).await.unwrap();
    assert_eq!(result.data[0].name, "Sodium");
}

#[tokio::test]
async fn test_get_mod() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/mods/394468"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mod_json()))
        .mount(&mock_server)
        .await;

    let client = CurseForgeClient::with_base_url("test-key", format!("{}/v1", mock_server.uri()));
    let m = client.get_mod(394468).await.unwrap();

    assert_eq!(m.id, 394468);
    assert_eq!(m.name, "Sodium");
    assert_eq!(m.slug, "sodium");
    assert_eq!(m.download_count, 52000000);
    assert!(m.is_featured);
    assert_eq!(m.categories.len(), 1);
    assert_eq!(m.categories[0].name, "Performance");
    assert_eq!(m.authors.len(), 1);
    assert_eq!(m.authors[0].name, "jellysquid3_");
    assert!(m.logo.is_some());
    assert!(m.links.is_some());
    assert_eq!(
        m.links.unwrap().source_url.unwrap(),
        "https://github.com/CaffeineMC/sodium"
    );
}

#[tokio::test]
async fn test_get_mods_batch() {
    let mock_server = MockServer::start().await;

    let response = serde_json::json!({
        "data": [
            {
                "id": 394468,
                "gameId": 432,
                "name": "Sodium",
                "slug": "sodium",
                "summary": "Performance mod",
                "status": 4,
                "downloadCount": 52000000,
                "isFeatured": true,
                "categories": [],
                "authors": [],
                "screenshots": [],
                "latestFiles": [],
                "latestFilesIndexes": [],
                "isAvailable": true,
                "thumbsUpCount": 25000
            },
            {
                "id": 394469,
                "gameId": 432,
                "name": "Lithium",
                "slug": "lithium",
                "summary": "General optimization",
                "status": 4,
                "downloadCount": 30000000,
                "isFeatured": false,
                "categories": [],
                "authors": [],
                "screenshots": [],
                "latestFiles": [],
                "latestFilesIndexes": [],
                "isAvailable": true,
                "thumbsUpCount": 15000
            }
        ]
    });

    Mock::given(method("POST"))
        .and(path("/v1/mods"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&mock_server)
        .await;

    let client = CurseForgeClient::with_base_url("test-key", format!("{}/v1", mock_server.uri()));
    let mods = client.get_mods(&[394468, 394469]).await.unwrap();

    assert_eq!(mods.len(), 2);
    assert_eq!(mods[0].name, "Sodium");
    assert_eq!(mods[1].name, "Lithium");
}

#[tokio::test]
async fn test_get_mod_files() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/mods/394468/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(files_json()))
        .mount(&mock_server)
        .await;

    let client = CurseForgeClient::with_base_url("test-key", format!("{}/v1", mock_server.uri()));
    let files = client.get_mod_files(394468).await.unwrap();

    assert_eq!(files.len(), 2);
    assert_eq!(files[0].id, 5000001);
    assert_eq!(files[0].display_name, "sodium-fabric-0.5.3+mc1.20.1");
    assert_eq!(files[0].release_type, 1);
    assert_eq!(files[0].file_length, 678912);
    assert_eq!(files[0].hashes.len(), 2);
    assert_eq!(files[0].dependencies.len(), 1);
    assert_eq!(files[0].dependencies[0].relation_type, 3); // required
    assert_eq!(files[0].modules.len(), 2);
    assert!(files[0].download_url.is_some());
}

#[tokio::test]
async fn test_get_mod_file() {
    let mock_server = MockServer::start().await;

    let single_file = serde_json::json!({
        "data": {
            "id": 5000001,
            "gameId": 432,
            "modId": 394468,
            "isAvailable": true,
            "displayName": "sodium-fabric-0.5.3+mc1.20.1",
            "fileName": "sodium-fabric-0.5.3+mc1.20.1.jar",
            "releaseType": 1,
            "fileStatus": 4,
            "hashes": [],
            "fileDate": "2024-01-10T10:00:00.000Z",
            "fileLength": 678912,
            "downloadCount": 5000000,
            "downloadUrl": "https://edge.forgecdn.net/files/5000/001/sodium-fabric-0.5.3+mc1.20.1.jar",
            "gameVersions": ["1.20.1", "Fabric"],
            "sortableGameVersions": [],
            "dependencies": [],
            "isServerPack": false,
            "modules": []
        }
    });

    Mock::given(method("GET"))
        .and(path("/v1/mods/394468/files/5000001"))
        .respond_with(ResponseTemplate::new(200).set_body_json(single_file))
        .mount(&mock_server)
        .await;

    let client = CurseForgeClient::with_base_url("test-key", format!("{}/v1", mock_server.uri()));
    let file = client.get_mod_file(394468, 5000001).await.unwrap();

    assert_eq!(file.id, 5000001);
    assert_eq!(file.mod_id, 394468);
    assert_eq!(file.file_name, "sodium-fabric-0.5.3+mc1.20.1.jar");
}

#[tokio::test]
async fn test_get_categories() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/categories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(categories_json()))
        .mount(&mock_server)
        .await;

    let client = CurseForgeClient::with_base_url("test-key", format!("{}/v1", mock_server.uri()));
    let categories = client.get_categories().await.unwrap();

    assert_eq!(categories.len(), 3);
    assert_eq!(categories[0].name, "Mods");
    assert_eq!(categories[0].is_class, Some(true));
    assert_eq!(categories[1].name, "Performance");
    assert_eq!(categories[1].class_id, Some(6));
    assert_eq!(categories[2].name, "Map and Information");
}

#[tokio::test]
async fn test_caching_prevents_duplicate_requests() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/mods/394468"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mod_json()))
        .expect(1) // Should only be called once due to caching
        .mount(&mock_server)
        .await;

    let client = CurseForgeClient::with_base_url("test-key", format!("{}/v1", mock_server.uri()));

    let mod1 = client.get_mod(394468).await.unwrap();
    let mod2 = client.get_mod(394468).await.unwrap();

    assert_eq!(mod1.id, mod2.id);
    assert_eq!(mod1.name, mod2.name);
}

#[tokio::test]
async fn test_clear_cache_allows_fresh_request() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/mods/394468"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mod_json()))
        .expect(2) // Called twice since we clear cache
        .mount(&mock_server)
        .await;

    let client = CurseForgeClient::with_base_url("test-key", format!("{}/v1", mock_server.uri()));

    let _ = client.get_mod(394468).await.unwrap();
    client.clear_cache().await;
    let _ = client.get_mod(394468).await.unwrap();
}

#[tokio::test]
async fn test_api_error_handling() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/mods/999999"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not found"))
        .mount(&mock_server)
        .await;

    let client = CurseForgeClient::with_base_url("test-key", format!("{}/v1", mock_server.uri()));
    let result = client.get_mod(999999).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        curseforge::CurseForgeError::Api { status, message } => {
            assert_eq!(status, 404);
            assert_eq!(message, "Not found");
        }
        err => panic!("Expected Api error, got: {err:?}"),
    }
}

#[tokio::test]
async fn test_rate_limit_handling() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/mods/394468"))
        .respond_with(
            ResponseTemplate::new(429).insert_header("retry-after", "30"),
        )
        .mount(&mock_server)
        .await;

    let client = CurseForgeClient::with_base_url("test-key", format!("{}/v1", mock_server.uri()));
    let result = client.get_mod(394468).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        curseforge::CurseForgeError::RateLimited { retry_after_ms } => {
            assert_eq!(retry_after_ms, 30000);
        }
        err => panic!("Expected RateLimited error, got: {err:?}"),
    }
}

#[tokio::test]
async fn test_model_deserialization_from_fixtures() {
    let _: curseforge::SearchResult =
        serde_json::from_str(include_str!("../test_fixtures/search_result.json")).unwrap();

    let wrapper: serde_json::Value =
        serde_json::from_str(include_str!("../test_fixtures/mod.json")).unwrap();
    let _: curseforge::Mod = serde_json::from_value(wrapper["data"].clone()).unwrap();

    let wrapper: serde_json::Value =
        serde_json::from_str(include_str!("../test_fixtures/files.json")).unwrap();
    let _: Vec<curseforge::File> = serde_json::from_value(wrapper["data"].clone()).unwrap();

    let wrapper: serde_json::Value =
        serde_json::from_str(include_str!("../test_fixtures/categories.json")).unwrap();
    let _: Vec<curseforge::Category> = serde_json::from_value(wrapper["data"].clone()).unwrap();
}
