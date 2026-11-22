use std::path::PathBuf;
use std::sync::Arc;

use obsidian_common::{CommonError, EventBus, Loader, Paths, TaskRegistry};
use obsidian_db::Db;
use obsidian_instances::{CreateInstance, InstanceManager, UpdateInstance};
use obsidian_java::JavaManager;
use obsidian_net::UpnpManager;

fn request(name: &str, port: Option<u16>) -> CreateInstance {
	CreateInstance {
		name: name.into(),
		loader: Loader::Vanilla,
		mc_version: "1.21.4".into(),
		loader_version: None,
		java_runtime_id: None,
		port,
		upnp_enabled: None,
		accept_eula: true,
	}
}

async fn open(dir_name: &str) -> (InstanceManager, Paths, Db) {
	let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(dir_name);
	let _ = std::fs::remove_dir_all(&dir);
	let paths = Paths::new(&dir).expect("paths");
	let db = Db::open(&paths).await.expect("db");
	let bus = EventBus::new();
	let manager = InstanceManager::new(
		db.clone(),
		paths.clone(),
		bus.clone(),
		TaskRegistry::new(bus),
		Arc::new(JavaManager::new(db.clone(), paths.clone())),
		Arc::new(UpnpManager::new()),
	);
	(manager, paths, db)
}

async fn add_user(db: &Db, username: &str, instance_limit: i64) -> i64 {
	sqlx::query("INSERT INTO users (username, password_hash, is_admin, instance_limit) VALUES (?, 'x', 0, ?)")
		.bind(username)
		.bind(instance_limit)
		.execute(db.pool())
		.await
		.expect("insert user");
	sqlx::query_scalar("SELECT id FROM users WHERE username = ?")
		.bind(username)
		.fetch_one(db.pool())
		.await
		.expect("user id")
}

#[tokio::test(flavor = "multi_thread")]
async fn instance_crud_round_trip() {
	let (manager, paths, db) = open("instances-crud").await;
	let alice = add_user(&db, "alice", 2).await;

	let first = manager.create(request("First", None), alice).await.expect("create first");
	assert_eq!(first.port, 25565);
	assert_eq!(first.directory, paths.instance_dir(first.id));
	let second = manager.create(request("Second", None), alice).await.expect("create second");
	assert_eq!(second.port, 25566);
	assert!(matches!(
		manager.create(request("Third", None), alice).await,
		Err(CommonError::Forbidden(_))
	));

	let moved = manager
		.update(first.id, UpdateInstance { port: Some(25567), memory_max_mb: Some(4096), ..Default::default() })
		.await
		.expect("update");
	assert_eq!(moved.port, 25567);
	assert_eq!(moved.memory_max_mb, 4096);
	assert!(matches!(
		manager.update(second.id, UpdateInstance { port: Some(25567), ..Default::default() }).await,
		Err(CommonError::Conflict(_))
	));

	let properties = std::collections::BTreeMap::from([("motd".to_owned(), "hello".to_owned())]);
	manager.set_properties(first.id, properties).await.expect("set properties");
	let stored = manager.properties(first.id).await.expect("read properties");
	assert_eq!(stored.get("server-port").map(String::as_str), Some("25567"));
	assert_eq!(stored.get("motd").map(String::as_str), Some("hello"));

	assert_eq!(manager.list_for(alice).await.expect("list").len(), 2);
	while manager.status(first.id).await.expect("status").installing {
		tokio::time::sleep(std::time::Duration::from_millis(20)).await;
	}
	manager.delete(first.id).await.expect("delete");
	assert!(matches!(manager.get(first.id).await, Err(CommonError::NotFound(_))));
	assert!(!first.directory.exists());
	assert_eq!(manager.list_for(alice).await.expect("list").len(), 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn custom_create_does_not_start_install() {
	let (manager, _paths, db) = open("instances-custom").await;
	let owner = add_user(&db, "bob", 1).await;

	let custom = CreateInstance { loader: Loader::Custom, ..request("Custom", None) };
	let created = manager.create(custom, owner).await.expect("create custom");
	assert_eq!(created.loader, Loader::Custom);
	assert!(created.directory.exists());
	assert!(!manager.status(created.id).await.expect("status").installing);
}
