//! Authentication cache integration tests.

use chromenet::socket::authcache::{AuthCache, BasicAuthEntry};

#[test]
fn test_store_and_retrieve() {
    let cache = AuthCache::new();

    let entry = BasicAuthEntry::new("MyRealm", "user", "pass");
    cache.store_basic("proxy.example.com", 8080, "MyRealm", entry);

    let retrieved = cache.lookup_basic("proxy.example.com", 8080, "MyRealm");
    assert!(retrieved.is_some());

    let entry = retrieved.unwrap();
    assert_eq!(entry.username, "user");
    assert_eq!(entry.password, "pass");
}

#[test]
fn test_different_ports() {
    let cache = AuthCache::new();

    cache.store_basic(
        "proxy.com",
        8080,
        "Realm",
        BasicAuthEntry::new("Realm", "user1", "pass1"),
    );
    cache.store_basic(
        "proxy.com",
        8888,
        "Realm",
        BasicAuthEntry::new("Realm", "user2", "pass2"),
    );

    // Different ports should have different credentials
    let entry1 = cache.lookup_basic("proxy.com", 8080, "Realm").unwrap();
    let entry2 = cache.lookup_basic("proxy.com", 8888, "Realm").unwrap();

    assert_eq!(entry1.username, "user1");
    assert_eq!(entry2.username, "user2");
}

#[test]
fn test_different_realms() {
    let cache = AuthCache::new();

    cache.store_basic(
        "proxy.com",
        80,
        "Admin",
        BasicAuthEntry::new("Admin", "admin", "secret"),
    );
    cache.store_basic(
        "proxy.com",
        80,
        "User",
        BasicAuthEntry::new("User", "guest", "public"),
    );

    let admin = cache.lookup_basic("proxy.com", 80, "Admin").unwrap();
    let user = cache.lookup_basic("proxy.com", 80, "User").unwrap();

    assert_eq!(admin.username, "admin");
    assert_eq!(user.username, "guest");
}

#[test]
fn test_authorization_header() {
    let entry = BasicAuthEntry::new("Realm", "user", "pass");
    let header = entry.to_header_value();

    // user:pass in base64 = dXNlcjpwYXNz
    assert_eq!(header, "Basic dXNlcjpwYXNz");
}

#[test]
fn test_remove_host() {
    let cache = AuthCache::new();

    cache.store_basic(
        "proxy.com",
        80,
        "Realm1",
        BasicAuthEntry::new("Realm1", "u", "p"),
    );
    cache.store_basic(
        "proxy.com",
        80,
        "Realm2",
        BasicAuthEntry::new("Realm2", "u", "p"),
    );
    cache.store_basic(
        "other.com",
        80,
        "Realm",
        BasicAuthEntry::new("Realm", "u", "p"),
    );

    cache.remove_host("proxy.com", 80);

    // proxy.com entries should be removed
    assert!(cache.lookup_basic("proxy.com", 80, "Realm1").is_none());
    assert!(cache.lookup_basic("proxy.com", 80, "Realm2").is_none());

    // other.com should still exist
    assert!(cache.lookup_basic("other.com", 80, "Realm").is_some());
}

#[test]
fn test_clear_all() {
    let cache = AuthCache::new();

    cache.store_basic("a.com", 80, "R", BasicAuthEntry::new("R", "u", "p"));
    cache.store_basic("b.com", 80, "R", BasicAuthEntry::new("R", "u", "p"));
    cache.store_basic("c.com", 80, "R", BasicAuthEntry::new("R", "u", "p"));

    assert_eq!(cache.len(), 3);

    cache.clear();

    assert!(cache.is_empty());
}
