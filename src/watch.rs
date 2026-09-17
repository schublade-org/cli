use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WatchSpec {
    pub path: PathBuf,
    pub recursive: bool,
}

/// Directories the serve watcher should subscribe to.
///
/// Config and catalog files are watched via their parent directory so atomic
/// editor saves (write-temp + rename) still notify. The stories root is
/// recursive so CSF files and the components they import are both covered.
pub fn watch_specs(
    config_path: Option<&Path>,
    catalog_path: Option<&Path>,
    stories_root: Option<&Path>,
) -> Vec<WatchSpec> {
    let mut specs = BTreeSet::new();

    if let Some(path) = config_path {
        specs.insert(WatchSpec {
            path: watch_dir_for_file(path),
            recursive: false,
        });
    }
    if let Some(path) = catalog_path {
        specs.insert(WatchSpec {
            path: watch_dir_for_file(path),
            recursive: false,
        });
    }
    if let Some(path) = stories_root {
        specs.insert(WatchSpec {
            path: path.to_path_buf(),
            recursive: true,
        });
    }

    if specs.is_empty() {
        specs.insert(WatchSpec {
            path: PathBuf::from("."),
            recursive: false,
        });
    }

    specs.into_iter().collect()
}

fn watch_dir_for_file(path: &Path) -> PathBuf {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

pub fn is_noise_path(path: &Path) -> bool {
    path.components().any(|component| {
        let name = component.as_os_str().to_string_lossy();
        matches!(
            name.as_ref(),
            "target" | "node_modules" | "dist" | "build" | ".git" | "npm" | "vendor"
        )
    }) || {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        name.starts_with('.')
            || name.ends_with('~')
            || name.ends_with(".swp")
            || name.ends_with(".swo")
            || name.ends_with(".tmp")
    }
}

/// Whether a filesystem event should trigger catalog + story reload.
pub fn should_reload(path: &Path, stories_root: Option<&Path>) -> bool {
    if is_noise_path(path) {
        return false;
    }

    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    if name == "schublade.toml" || name == "catalog.toml" {
        return true;
    }

    if let Some(root) = stories_root {
        if path.starts_with(root) {
            return true;
        }
    }

    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("js" | "jsx" | "html" | "css" | "toml" | "svg" | "png" | "ico" | "webp")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watches_config_and_catalog_parents_and_stories_tree() {
        let specs = watch_specs(
            Some(Path::new("examples/story-files/schublade.toml")),
            Some(Path::new("examples/story-files/catalog.toml")),
            Some(Path::new("examples/story-files/components")),
        );
        assert!(specs.contains(&WatchSpec {
            path: PathBuf::from("examples/story-files"),
            recursive: false,
        }));
        assert!(specs.contains(&WatchSpec {
            path: PathBuf::from("examples/story-files/components"),
            recursive: true,
        }));
        assert_eq!(specs.len(), 2);
    }

    #[test]
    fn catalog_only_example_watches_its_folder() {
        let specs = watch_specs(
            Some(Path::new("examples/empty-catalog/schublade.toml")),
            Some(Path::new("examples/empty-catalog/catalog.toml")),
            None,
        );
        assert_eq!(
            specs,
            vec![WatchSpec {
                path: PathBuf::from("examples/empty-catalog"),
                recursive: false,
            }]
        );
    }

    #[test]
    fn bundled_defaults_watch_cwd() {
        let specs = watch_specs(None, None, None);
        assert_eq!(
            specs,
            vec![WatchSpec {
                path: PathBuf::from("."),
                recursive: false,
            }]
        );
    }

    #[test]
    fn reloads_toml_and_story_sources() {
        assert!(should_reload(Path::new("examples/a11y/schublade.toml"), None));
        assert!(should_reload(Path::new("examples/a11y/catalog.toml"), None));
        assert!(should_reload(
            Path::new("components/button.stories.jsx"),
            Some(Path::new("components"))
        ));
        assert!(should_reload(
            Path::new("components/button.jsx"),
            Some(Path::new("components"))
        ));
        assert!(!should_reload(Path::new("README.md"), None));
        assert!(!should_reload(Path::new("components/.DS_Store"), Some(Path::new("components"))));
        assert!(!should_reload(Path::new("target/debug/schublade"), None));
    }
}
