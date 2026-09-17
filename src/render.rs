use serde_json::Value;

use crate::catalog::{Control, Generator, Story};

pub struct RenderedStory {
    pub html: String,
    pub code: String,
    pub react: Option<ReactRender>,
}

#[derive(Debug, Clone)]
pub struct ReactRender {
    pub source: String,
    pub export_name: String,
    pub props: serde_json::Map<String, Value>,
}

/// Code Usage snippet at the story's default control values.
pub fn usage_code(story: &Story) -> String {
    let values = with_attr_tokens(defaults_from_controls(&story.controls));
    interpolate(&story.code, &values, false)
}

pub fn render_story(story: &Story, values: &Value) -> Result<RenderedStory, String> {
    let mut merged = defaults_from_controls(&story.controls);
    if let Some(object) = values.as_object() {
        for (key, value) in object {
            merged.insert(key.clone(), value.clone());
        }
    }
    let props = merged.clone();
    let values = with_attr_tokens(merged);
    let html = match story.generator {
        Generator::AvatarGroup => render_avatar_group(&values),
        Generator::Html => {
            let template = story
                .template
                .as_deref()
                .ok_or_else(|| format!("story '{}' is missing a template", story.id))?;
            interpolate(template, &values, true)
        }
        Generator::React => String::new(),
    };
    let react = match story.generator {
        Generator::React => {
            let source = story.component_source.clone().ok_or_else(|| {
                format!("story '{}' is missing component source", story.id)
            })?;
            Some(ReactRender {
                source,
                export_name: story
                    .component_export
                    .clone()
                    .or_else(|| story.component_name.clone())
                    .unwrap_or_else(|| "default".into()),
                props,
            })
        }
        Generator::Html | Generator::AvatarGroup => None,
    };

    Ok(RenderedStory {
        html,
        code: interpolate(&story.code, &values, false),
        react,
    })
}

fn defaults_from_controls(controls: &[Control]) -> serde_json::Map<String, Value> {
    let mut values = serde_json::Map::new();
    for control in controls {
        let value = match control {
            Control::Select { default, .. } | Control::Text { default, .. } => {
                Value::String(default.clone())
            }
            Control::Number { default, .. } => Value::Number((*default).into()),
            Control::Boolean { default, .. } => Value::Bool(*default),
        };
        values.insert(control.id().to_string(), value);
    }
    values
}

fn with_attr_tokens(
    mut values: serde_json::Map<String, Value>,
) -> serde_json::Map<String, Value> {
    let extras: Vec<(String, String)> = values
        .iter()
        .filter_map(|(key, value)| {
            let flag = value.as_bool()?;
            let token = match key.as_str() {
                "open" => {
                    if flag {
                        " open"
                    } else {
                        ""
                    }
                }
                "disabled" => {
                    if flag {
                        " disabled"
                    } else {
                        ""
                    }
                }
                "dismissible" => {
                    if flag {
                        ""
                    } else {
                        " hidden"
                    }
                }
                _ => return None,
            };
            Some((format!("{key}Attr"), token.to_string()))
        })
        .collect();

    for (key, token) in extras {
        values.insert(key, Value::String(token));
    }
    values
}

pub fn interpolate(
    template: &str,
    values: &serde_json::Map<String, Value>,
    escape_html: bool,
) -> String {
    let mut output = template.to_string();
    for (key, value) in values {
        let raw = json_to_string(value);
        let replacement = if escape_html {
            escape(&raw)
        } else {
            raw
        };
        output = output.replace(&format!("{{{{{key}}}}}"), &replacement);
    }
    output
}

pub fn overflow_faces(total: usize, visible: usize) -> (usize, Option<usize>) {
    if total == 0 || visible == 0 {
        return (0, None);
    }
    if total <= visible {
        (total, None)
    } else {
        let faces = visible.saturating_sub(1);
        (faces, Some(total.saturating_sub(faces)))
    }
}

fn json_to_string(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(number) => number.to_string(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

struct Person {
    name: &'static str,
    initials: &'static str,
    color: &'static str,
    portrait: Option<&'static str>,
}

const PEOPLE: &[Person] = &[
    Person {
        name: "Lena Meier",
        initials: "LM",
        color: "#C4B5A0",
        portrait: Some(LENA),
    },
    Person {
        name: "Jonas Keller",
        initials: "JK",
        color: "#8FA08C",
        portrait: Some(JONAS),
    },
    Person {
        name: "Theo Hart",
        initials: "T",
        color: "#F97316",
        portrait: None,
    },
    Person {
        name: "Sofia Nguyen",
        initials: "SN",
        color: "#B7A99A",
        portrait: Some(SOFIA),
    },
    Person {
        name: "Amir Rossi",
        initials: "AR",
        color: "#9AA7B2",
        portrait: Some(AMIR),
    },
    Person {
        name: "Pia Hofmann",
        initials: "PH",
        color: "#C9B8C4",
        portrait: None,
    },
    Person {
        name: "Noah Graf",
        initials: "NG",
        color: "#A3B18A",
        portrait: None,
    },
    Person {
        name: "Elena Berg",
        initials: "EB",
        color: "#D4A373",
        portrait: None,
    },
    Person {
        name: "Milo Farid",
        initials: "MF",
        color: "#7C93A8",
        portrait: None,
    },
    Person {
        name: "Ava Kunz",
        initials: "AK",
        color: "#C08497",
        portrait: None,
    },
    Person {
        name: "Rico Steiner",
        initials: "RS",
        color: "#8E9A7C",
        portrait: None,
    },
    Person {
        name: "Noor Salim",
        initials: "NS",
        color: "#B08968",
        portrait: None,
    },
];

const LENA: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 64'%3E%3Crect fill='%23C4B5A0' width='64' height='64'/%3E%3Ccircle cx='32' cy='26' r='12' fill='%23F0D2B6'/%3E%3Cpath d='M14 64c2-16 10-24 18-24s16 8 18 24' fill='%232C3A4A'/%3E%3Cpath d='M20 22c4-10 20-10 24 2-6-2-18-2-24-2z' fill='%233E2A22'/%3E%3C/svg%3E";
const JONAS: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 64'%3E%3Crect fill='%238FA08C' width='64' height='64'/%3E%3Ccircle cx='32' cy='25' r='12' fill='%23E6C2A0'/%3E%3Cpath d='M12 64c3-15 11-23 20-23s17 8 20 23' fill='%231E2A28'/%3E%3Cpath d='M18 20c6-9 22-8 26 3-8-3-20-3-26-3z' fill='%23241A14'/%3E%3C/svg%3E";
const SOFIA: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 64'%3E%3Crect fill='%23B7A99A' width='64' height='64'/%3E%3Ccircle cx='32' cy='26' r='12' fill='%23F2C9B0'/%3E%3Cpath d='M13 64c2-16 10-24 19-24s17 8 19 24' fill='%234A342C'/%3E%3Cpath d='M16 24c8-14 24-14 32 0-10-4-22-4-32 0z' fill='%231A1210'/%3E%3C/svg%3E";
const AMIR: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 64'%3E%3Crect fill='%239AA7B2' width='64' height='64'/%3E%3Ccircle cx='32' cy='25' r='12' fill='%23D7B08C'/%3E%3Cpath d='M12 64c3-15 11-23 20-23s17 8 20 23' fill='%23222630'/%3E%3Cpath d='M20 18c6-8 18-8 24 2-7-1-17-1-24-2z' fill='%23171412'/%3E%3C/svg%3E";

fn render_avatar_group(values: &serde_json::Map<String, Value>) -> String {
    let size = values
        .get("size")
        .and_then(Value::as_str)
        .unwrap_or("md");
    let total = values
        .get("total")
        .and_then(Value::as_i64)
        .unwrap_or(7)
        .clamp(1, PEOPLE.len() as i64) as usize;
    let visible = values
        .get("visible")
        .and_then(Value::as_i64)
        .unwrap_or(4)
        .clamp(1, 8) as usize;
    let show_count = values
        .get("showCount")
        .and_then(Value::as_bool)
        .unwrap_or(true);

    let (faces, overflow) = overflow_faces(total, visible);
    let mut items = String::new();

    for person in PEOPLE.iter().take(faces) {
        items.push_str(&avatar_item(person));
    }

    if let Some(count) = overflow {
        items.push_str(&format!(
            r#"<span class="ag-item"><span class="ag-overflow" aria-hidden="true">+{count}</span></span>"#
        ));
    }

    let caption = if show_count {
        format!(r#"<p class="ag-caption">{total} members</p>"#)
    } else {
        String::new()
    };

    format!(
        r#"<div class="ag-wrap">
  <div class="ag" data-size="{size}" role="img" aria-label="{total} members">{items}</div>
  {caption}
</div>"#
    )
}

fn avatar_item(person: &Person) -> String {
    match person.portrait {
        Some(src) => format!(
            r#"<span class="ag-item"><img src="{src}" alt="{}" /></span>"#,
            escape(person.name)
        ),
        None => format!(
            r#"<span class="ag-item"><span class="ag-initials" style="background:{}" aria-label="{}">{}</span></span>"#,
            person.color,
            escape(person.name),
            escape(person.initials)
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overflow_matches_max_slots() {
        assert_eq!(overflow_faces(7, 4), (3, Some(4)));
        assert_eq!(overflow_faces(4, 4), (4, None));
        assert_eq!(overflow_faces(1, 4), (1, None));
    }

    #[test]
    fn interpolate_replaces_tokens() {
        let mut values = serde_json::Map::new();
        values.insert("label".into(), Value::String("Save".into()));
        assert_eq!(
            interpolate(r#"<button>{{label}}</button>"#, &values, true),
            "<button>Save</button>"
        );
    }

    #[test]
    fn usage_code_matches_default_code_usage() {
        let story = Story {
            id: "button".into(),
            title: "Button".into(),
            group: None,
            item: "Button".into(),
            section: "Components".into(),
            description: String::new(),
            generator: Generator::Html,
            template: Some("<button>{{label}}</button>".into()),
            code: "<Button>{{label}}</Button>".into(),
            props: Vec::new(),
            controls: vec![Control::Text {
                id: "label".into(),
                label: "Label".into(),
                default: "Save".into(),
            }],
            component_source: None,
            component_export: None,
            component_name: None,
        };
        assert_eq!(usage_code(&story), "<Button>Save</Button>");
        let rendered = render_story(&story, &Value::Object(Default::default())).unwrap();
        assert_eq!(usage_code(&story), rendered.code);
    }

    #[test]
    fn interpolate_escapes_html() {
        let mut values = serde_json::Map::new();
        values.insert("label".into(), Value::String("<em>x</em>".into()));
        assert_eq!(
            interpolate("{{label}}", &values, true),
            "&lt;em&gt;x&lt;/em&gt;"
        );
    }
}
