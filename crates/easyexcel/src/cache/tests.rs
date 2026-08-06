//! Cache facade tests.

use super::{
    EternalReadCacheSelector, FileCache, MapCache, MokaCache, ReadCache, ReadCacheSelector,
    SimpleReadCacheSelector, XlsCache,
};
use crate::ReadCacheMode;

#[test]
fn map_cache_stores_and_retrieves_values() {
    let mut cache = MapCache::new();
    cache.put("alpha".to_owned()).expect("put");
    cache.put("beta".to_owned()).expect("put");
    cache.put_finished().expect("put finished");
    assert_eq!(cache.get(Some(1)).expect("get"), Some("beta".to_owned()));
}

#[test]
fn xls_cache_reads_preloaded_sst_values() {
    let cache = XlsCache::new(vec!["one".to_owned(), "two".to_owned()]);
    assert_eq!(cache.len(), 2);
    assert_eq!(cache.get(Some(0)).expect("get"), Some("one".to_owned()));
    assert!(cache.get(Some(99)).expect("get").is_none());
}

#[test]
fn simple_selector_matches_java_five_megabyte_threshold() {
    let selector = SimpleReadCacheSelector::new();
    assert_eq!(selector.select_mode(4_999_999), ReadCacheMode::Memory);
    assert_eq!(selector.select_mode(5_000_000), ReadCacheMode::File);
}

#[test]
fn simple_selector_custom_mb_threshold() {
    let selector = SimpleReadCacheSelector::with_max_use_map_cache_size_mb(1);
    assert_eq!(selector.select_mode(999_999), ReadCacheMode::Memory);
    assert_eq!(selector.select_mode(1_000_000), ReadCacheMode::File);
}

#[test]
fn eternal_selector_pins_backend_mode() {
    let selector = EternalReadCacheSelector::map_cache();
    assert_eq!(selector.select_mode(9_999_999), ReadCacheMode::Memory);
    let moka = EternalReadCacheSelector::moka();
    assert_eq!(moka.select_mode(0), ReadCacheMode::Moka);
    let file = EternalReadCacheSelector::file_cache();
    assert_eq!(file.select_mode(0), ReadCacheMode::File);
}

#[test]
fn moka_cache_round_trips_objects() {
    let mut cache = MokaCache::new();
    cache.put("object-value".to_owned()).expect("put");
    cache.put_finished().expect("put finished");
    assert_eq!(
        cache.get(Some(0)).expect("get"),
        Some("object-value".to_owned())
    );
    cache.destroy();
    assert!(cache.get(Some(0)).is_err());
}

#[test]
fn moka_cache_retains_all_objects_until_destroy() {
    let mut cache = MokaCache::new();
    for index in 0..10_000 {
        cache.put(format!("object-{index}")).expect("put");
    }
    cache.put_finished().expect("put finished");
    assert_eq!(
        cache.get(Some(0)).expect("get").as_deref(),
        Some("object-0")
    );
    assert_eq!(
        cache.get(Some(9_999)).expect("get").as_deref(),
        Some("object-9999")
    );
}

#[test]
fn file_cache_round_trips_values_and_releases_file_on_destroy() {
    let mut cache = FileCache::new().expect("file cache");
    cache.put("file-value".to_owned()).expect("put");
    cache.put_finished().expect("put finished");
    assert_eq!(
        cache.get(Some(0)).expect("get"),
        Some("file-value".to_owned())
    );
    cache.destroy();
    assert!(cache.get(Some(0)).is_err());
}
