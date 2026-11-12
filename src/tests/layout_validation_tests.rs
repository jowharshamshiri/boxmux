//! Every configuration shipped with the project must load: schema, semantic checks and setup.

use crate::model::app::load_app_from_yaml;
use std::fs;
use std::path::{Path, PathBuf};

/// The `.yaml` files in `dir` and its subdirectories, such as an example's own folder.
fn yaml_files(dir: impl AsRef<Path>) -> Vec<PathBuf> {
    let dir = dir.as_ref();
    let mut files = Vec::new();
    for entry in
        fs::read_dir(dir).unwrap_or_else(|e| panic!("cannot read {}: {}", dir.display(), e))
    {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            files.extend(yaml_files(&path));
        } else if path.extension().is_some_and(|ext| ext == "yaml") {
            files.push(path);
        }
    }
    files.sort();
    files
}

#[test]
fn every_shipped_configuration_loads() {
    let files: Vec<PathBuf> = yaml_files("layouts")
        .into_iter()
        .chain(yaml_files("examples"))
        .collect();
    assert!(files.len() >= 10, "found only {:?}", files);
    let failures: Vec<String> = files
        .iter()
        .filter_map(|path| {
            load_app_from_yaml(path)
                .err()
                .map(|e| format!("{}: {}", path.display(), e))
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn every_loaded_configuration_shows_its_root_layout() {
    for path in yaml_files("layouts") {
        let app = load_app_from_yaml(&path).unwrap();
        let active = app
            .get_active_layout()
            .unwrap_or_else(|| panic!("{}: no active layout", path.display()));
        // The root layout, or the first one when none is marked root.
        let expected = app.get_root_layout().unwrap();
        assert_eq!(active.id, expected.id, "{}", path.display());
    }
}

#[test]
fn feature_demos_contain_the_feature_they_demonstrate() {
    let has = |file: &str, check: &dyn Fn(&crate::MuxBox) -> bool| {
        let app = load_app_from_yaml(&Path::new("layouts").join(file)).unwrap();
        app.layouts
            .iter()
            .flat_map(|layout| layout.get_all_muxboxes())
            .any(check)
    };
    assert!(has("chart_demo.yaml", &|b| b.chart_type.is_some()));
    assert!(has("table_demo.yaml", &|b| b.table_data.is_some()));
    assert!(has("plugin_demo.yaml", &|b| b.plugin_component.is_some()));
}
