//! Authentication cache integration tests.

use chromenet::socket::authcache::{AuthCache, AuthEntry, AuthScheme};

#[test]
fn test_store_and_retrieve() {
    let cache = AuthCache::new();

    let entry = AuthEntry::basic("MyRealm", "user", "pass");
    cache.store("proxy.example.com", 8080, "MyRealm", entry);

    let retrieved = cache.lookup("proxy.example.com", 8080, "MyRealm");
    assert!(retrieved.is_some());

    let entry = retrieved.unwrap();
    assert_eq!(entry.username, "user");
    assert_eq!(entry.password, "pass");
    assert_eq!(entry.scheme, AuthScheme::Basic);
}

#[test]
fn test_different_ports() {
    let cache = AuthCache::new();

    cache.store(
        "proxy.com",
        8080,
        "Realm",
        AuthEntry::basic("Realm", "user1", "pass1"),
    );
    cache.store(
        "proxy.com",
        8888,
        "Realm",
        AuthEntry::basic("Realm", "user2", "pass2"),
    );

    // Different ports should have different credentials
    let entry1 = cache.lookup("proxy.com", 8080, "Realm").unwrap();
    let entry2 = cache.lookup("proxy.com", 8888, "Realm").unwrap();

    assert_eq!(entry1.username, "user1");
    assert_eq!(entry2.username, "user2");
}

#[test]
fn test_different_realms() {
    let cache = AuthCache::new();

    cache.store(
        "proxy.com",
        80,
        "Admin",
        AuthEntry::basic("Admin", "admin", "secret"),
    );
    cache.store(
        "proxy.com",
        80,
        "User",
        AuthEntry::basic("User", "guest", "public"),
    );

    let admin = cache.lookup("proxy.com", 80, "Admin").unwrap();
    let user = cache.lookup("proxy.com", 80, "User").unwrap();

    assert_eq!(admin.username, "admin");
    assert_eq!(user.username, "guest");
}

#[test]
fn test_authorization_header() {
    let entry = AuthEntry::basic("Realm", "user", "pass");
    let header = entry.to_header_value();

    // user:pass in base64 = dXNlcjpwYXNz
    assert_eq!(header, "Basic dXNlcjpwYXNz");
}

#[test]
fn test_remove_host() {
    let cache = AuthCache::new();

    cache.store(
        "proxy.com",
        80,
        "Realm1",
        AuthEntry::basic("Realm1", "u", "p"),
    );
    cache.store(
        "proxy.com",
        80,
        "Realm2",
        AuthEntry::basic("Realm2", "u", "p"),
    );
    cache.store(
        "other.com",
        80,
        "Realm",
        AuthEntry::basic("Realm", "u", "p"),
    );

    cache.remove_host("proxy.com", 80);

    // proxy.com entries should be removed
    assert!(cache.lookup("proxy.com", 80, "Realm1").is_none());
    assert!(cache.lookup("proxy.com", 80, "Realm2").is_none());

    // other.com should still exist
    assert!(cache.lookup("other.com", 80, "Realm").is_some());
}

#[test]
fn test_clear_all() {
    let cache = AuthCache::new();

    cache.store("a.com", 80, "R", AuthEntry::basic("R", "u", "p"));
    cache.store("b.com", 80, "R", AuthEntry::basic("R", "u", "p"));
    cache.store("c.com", 80, "R", AuthEntry::basic("R", "u", "p"));

    assert_eq!(cache.len(), 3);

    cache.clear();

    assert!(cache.is_empty());
}
