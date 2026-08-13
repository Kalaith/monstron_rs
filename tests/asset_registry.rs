use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

fn read_json(relative: &str) -> Value {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let json = fs::read_to_string(root.join(relative))
        .unwrap_or_else(|error| panic!("{relative} must be readable: {error}"));
    serde_json::from_str(&json).unwrap_or_else(|error| panic!("{relative} must be JSON: {error}"))
}

#[test]
fn asset_registry_matches_the_external_runtime_asset_set() {
    let registry = read_json("asset_registry.json");
    assert_eq!(registry["version"], 1);

    let registered: BTreeSet<&str> = registry["assets"]
        .as_array()
        .expect("asset registry needs an assets array")
        .iter()
        .map(|entry| entry.as_str().expect("asset paths must be strings"))
        .collect();

    // Hatchspire embeds every runtime texture and data file into the binary.
    // Keep this explicit so a future external loader must update the registry
    // and this integrity check together.
    let external_runtime_assets: BTreeSet<&str> = BTreeSet::new();

    assert_eq!(registered, external_runtime_assets);
}
