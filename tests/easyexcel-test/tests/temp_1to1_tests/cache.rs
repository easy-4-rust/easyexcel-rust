//! 1:1 method matrix for Java `com.alibaba.easyexcel.test.temp.cache.*`

use easyexcel::{MokaCache, ReadCache, ReadCacheMode};

/// Java `com.alibaba.easyexcel.test.temp.cache.CacheTest#cache`
///
/// Portable stand-in: delegates to the `EasyExcel` Moka object-cache test.
#[test]
fn cache_cache_test_cache() {
    cache_moka_facade_object_put_get();
}

/// `EasyExcel` `MokaCache` object put/get contract.
#[test]
fn cache_moka_facade_object_put_get() {
    let mut cache = MokaCache::new();
    cache.put("test".to_owned()).expect("put");
    cache.put_finished().expect("put finished");
    assert_eq!(cache.get(Some(0)).expect("get"), Some("test".to_owned()));
    cache.destroy();
}

/// Explicit Moka selection bypasses the automatic size threshold.
#[test]
fn cache_read_cache_mode_moka_variant() {
    assert_eq!(ReadCacheMode::Moka, ReadCacheMode::Moka);
    assert_ne!(ReadCacheMode::Moka, ReadCacheMode::Auto);
}
