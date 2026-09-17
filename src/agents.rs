use crate::catalog::{Catalog, Control, Story};
use crate::render;

/// Relative path of the catalog index: `AGENTS.md`.
pub const INDEX_PATH: &str = "AGENTS.md";

/// `/{id}/AGENTS.md` when `id` is a single safe path segment.
pub fn story_path(id: &str) -> Option<String> {
    if !is_safe_story_id(id) {
        return None;
    }
    Some(format!("{id}/AGENTS.md"))
}

pub fn is_safe_story_id(id: &str) -> bool {
    !id.is_empty()
        && !id.contains('/')
        && !id.contains('\\')
        && !id.contains("..")
        && id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}

/// Index + one file per story. Same bytes for live GET and `schublade build`.
pub fn files(catalog: &Catalog) -> Vec<(String, String)> {
    let mut out = vec![(INDEX_PATH.to_string(), index(catalog))];
    for story in &catalog.stories {
        if let Some(path) = story_path(&story.id) {
            out.push((path, story_doc(story)));
        }
    }
    out
}

pub fn index(catalog: &Catalog) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", catalog.name));
    out.push_str(
        "Generated from the story catalog. Usage and controls match the workshop.\n\n",
    );

    if catalog.stories.is_empty() {
        out.push_str("No stories yet.\n");
        return out;
    }

    let mut last_section: Option<&str> = None;
    for story in &catalog.stories {
        let Some(path) = story_path(&story.id) else {
            continue;
        };
        if last_section != Some(story.section.as_str()) {
            out.push_str(&format!("## {}\n\n", story.section));
            last_section = Some(story.section.as_str());
        }
        let href = format!("/{path}");
        if story.description.is_empty() {
            out.push_str(&format!("- [{}]({href})\n", escape_link_text(&story.title)));
        } else {
            out.push_str(&format!(
                "- [{}]({href}) — {}\n",
                escape_link_text(&story.title),
                story.description
            ));
        }
    }
    out.push('\n');
    out
}

pub fn story_doc(story: &Story) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", story.title));
    if !story.description.is_empty() {
        out.push_str(&story.description);
        out.push_str("\n\n");
    }
    if !story.section.is_empty() {
        out.push_str(&format!("Section: {}\n\n", story.section));
    }

    out.push_str("## Usage\n\n");
    out.push_str("```jsx\n");
    out.push_str(&render::usage_code(story));
    out.push_str("\n```\n\n");

    out.push_str("## Controls\n\n");
    if story.controls.is_empty() {
        out.push_str("This story has no controls.\n\n");
    } else {
        out.push_str("| Id | Label | Kind | Default | Options |\n");
        out.push_str("| --- | --- | --- | --- | --- |\n");
        for control in &story.controls {
            out.push_str(&control_row(control));
        }
        out.push('\n');
    }

    out.push_str("[Catalog index](/AGENTS.md)\n");
    out
}

fn control_row(control: &Control) -> String {
    format!(
        "| {} | {} | {} | {} | {} |\n",
        escape_cell(control.id()),
        escape_cell(control.label()),
        control.kind(),
        escape_cell(&control.default_display()),
        escape_cell(&control.extra_display()),
    )
}

fn escape_link_text(text: &str) -> String {
    text.replace('[', "\\[").replace(']', "\\]")
}

fn escape_cell(text: &str) -> String {
    text.replace('|', "\\|")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{Catalog, Control, Generator, SelectOption, Story};

    fn sample_story() -> Story {
        Story {
            id: "button-ghost".into(),
            title: "Button / Ghost".into(),
            group: Some("Button".into()),
            item: "Ghost".into(),
            section: "Components".into(),
            description: "A quiet action that sits next to a primary button.".into(),
            generator: Generator::Html,
            template: Some("<button class=\"btn\">{{label}}</button>".into()),
            code: r#"<Button variant="{{variant}}" size="{{size}}" disabled={{{disabled}}}>{{label}}</Button>"#.into(),
            controls: vec![
                Control::Text {
                    id: "label".into(),
                    label: "Label".into(),
                    default: "Cancel".into(),
                },
                Control::Select {
                    id: "variant".into(),
                    label: "Variant".into(),
                    options: vec![
                        SelectOption {
                            value: "primary".into(),
                            label: "Primary".into(),
                        },
                        SelectOption {
                            value: "ghost".into(),
                            label: "Ghost".into(),
                        },
                    ],
                    default: "ghost".into(),
                },
                Control::Select {
                    id: "size".into(),
                    label: "Size".into(),
                    options: vec![
                        SelectOption {
                            value: "sm".into(),
                            label: "Small".into(),
                        },
                        SelectOption {
                            value: "md".into(),
                            label: "Medium".into(),
                        },
                    ],
                    default: "md".into(),
                },
                Control::Boolean {
                    id: "disabled".into(),
                    label: "Disabled".into(),
                    default: false,
                },
            ],
            component_source: Some("export function Button() { return null }".into()),
            component_export: None,
            component_name: Some("Button".into()),
        }
    }

    fn sample_catalog() -> Catalog {
        Catalog {
            name: "Demo catalog".into(),
            stories: vec![sample_story()],
            pages: Vec::new(),
        }
    }

    #[test]
    fn index_lists_path_urls_not_hashes() {
        let markdown = index(&sample_catalog());
        assert!(markdown.starts_with("# Demo catalog\n"));
        assert!(markdown.contains("## Components\n"));
        assert!(markdown.contains("[Button / Ghost](/button-ghost/AGENTS.md)"));
        assert!(markdown.contains("A quiet action that sits next to a primary button."));
        assert!(!markdown.contains("#/"));
        assert!(!markdown.contains("export function Button"));
        assert!(!markdown.contains("class=\"btn\""));
    }

    #[test]
    fn story_doc_uses_usage_and_controls() {
        let catalog = sample_catalog();
        let markdown = story_doc(&catalog.stories[0]);
        assert!(markdown.starts_with("# Button / Ghost\n"));
        assert!(markdown.contains("A quiet action that sits next to a primary button."));
        assert!(markdown.contains("Section: Components"));
        assert!(
            markdown.contains("<Button variant=\"ghost\" size=\"md\" disabled={false}>Cancel</Button>"),
            "{markdown}"
        );
        assert!(markdown.contains("| label | Label | text | Cancel |  |"));
        assert!(markdown.contains("| variant | Variant | select | ghost | primary, ghost |"));
        assert!(markdown.contains("| disabled | Disabled | boolean | false |  |"));
        assert!(markdown.contains("[Catalog index](/AGENTS.md)"));
        assert!(!markdown.contains("#/button-ghost"));
        assert!(!markdown.contains("export function Button"));
        assert!(!markdown.contains("class=\"btn\""));
        assert!(!markdown.contains("component_source"));
    }

    #[test]
    fn files_cover_index_and_each_story() {
        let catalog = sample_catalog();
        let written = files(&catalog);
        assert_eq!(written[0].0, "AGENTS.md");
        assert_eq!(written[1].0, "button-ghost/AGENTS.md");
        assert_eq!(written[0].1, index(&catalog));
        assert_eq!(written[1].1, story_doc(&catalog.stories[0]));
    }

    #[test]
    fn empty_catalog_index() {
        let catalog = Catalog::empty("Empty catalog");
        let markdown = index(&catalog);
        assert!(markdown.starts_with("# Empty catalog\n"));
        assert!(markdown.contains("No stories yet."));
        assert_eq!(files(&catalog).len(), 1);
    }

    #[test]
    fn rejects_unsafe_story_ids() {
        assert!(story_path("button-ghost").is_some());
        assert!(story_path("../etc").is_none());
        assert!(story_path("a/b").is_none());
        assert!(story_path("").is_none());
    }

    #[test]
    fn bundled_catalog_renders_button_ghost() {
        let (catalog, _) = Catalog::load(None).unwrap();
        let story = catalog.story("button-ghost").expect("button-ghost");
        let markdown = story_doc(story);
        assert!(markdown.contains("# Button / Ghost"));
        assert!(markdown.contains("Cancel"));
        assert!(markdown.contains("ghost"));
        let index = index(&catalog);
        assert!(index.contains("/button-ghost/AGENTS.md"));
        assert!(!index.contains("#/"));
    }
}
