//! Text assets: web app manifest, browserconfig.xml and head snippets.

use serde_json::{Map, Value, json};

use crate::config::Settings;
use crate::spec::{
    ICON_SIZE, LEGACY_APPLE_TOUCH_LINK_SIZES, LEGACY_ICON_LINK_SIZES, MANIFEST_SIZES,
    TILE_IMAGE_SIZE, TILE_SIZES, maskable_name, png_name,
};

/// What the text assets describe: the settings, and whether the source was an SVG.
pub struct Config<'a> {
    pub settings: &'a Settings,
    pub has_svg: bool,
}

impl Config<'_> {
    fn url(&self, file: &str) -> String {
        format!("{}{file}", self.settings.path_prefix)
    }
}

/// `manifest.json` with the configured fields; `None` if the manifest is off.
pub fn manifest(cfg: &Config) -> Option<String> {
    let manifest = cfg.settings.manifest.as_ref()?;
    let mut icons: Vec<Value> = MANIFEST_SIZES
        .iter()
        .map(|&size| {
            json!({
                "src": cfg.url(&png_name(size)),
                "type": "image/png",
                "sizes": format!("{size}x{size}"),
            })
        })
        .collect();
    if manifest.maskable {
        icons.extend(MANIFEST_SIZES.iter().map(|&size| {
            json!({
                "src": cfg.url(&maskable_name(size)),
                "type": "image/png",
                "sizes": format!("{size}x{size}"),
                "purpose": "maskable",
            })
        }));
    }

    let mut object = Map::new();
    let mut set = |key: &str, value: Option<&str>| {
        if let Some(value) = value {
            object.insert(key.to_owned(), Value::String(value.to_owned()));
        }
    };
    set("name", manifest.name.as_deref());
    set("short_name", manifest.short_name.as_deref());
    set("description", manifest.description.as_deref());
    set("start_url", manifest.start_url.as_deref());
    set("scope", manifest.scope.as_deref());
    set("display", manifest.display.map(|d| d.as_str()));
    set("theme_color", cfg.settings.theme_color.as_deref());
    set("background_color", manifest.background_color.as_deref());
    object.insert("icons".into(), Value::Array(icons));

    Some(serde_json::to_string_pretty(&Value::Object(object)).expect("manifest serializes") + "\n")
}

/// `browserconfig.xml`; `None` if Windows tiles are off.
pub fn browserconfig(cfg: &Config) -> Option<String> {
    let windows = cfg.settings.windows.as_ref()?;
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<browserconfig>\n\t<msapplication>\n\t\t<tile>\n",
    );
    for &size in TILE_SIZES {
        xml += &format!(
            "\t\t\t<square{size}x{size}logo src=\"{}\"/>\n",
            escape(&cfg.url(&png_name(size)))
        );
    }
    if let Some(color) = &windows.tile_color {
        xml += &format!("\t\t\t<TileColor>{}</TileColor>\n", escape(color));
    }
    xml += "\t\t</tile>\n\t</msapplication>\n</browserconfig>\n";
    Some(xml)
}

enum TagKind {
    Link,
    Meta,
}

struct Tag {
    kind: TagKind,
    attrs: Vec<(&'static str, String)>,
}

fn link(attrs: Vec<(&'static str, String)>) -> Tag {
    Tag {
        kind: TagKind::Link,
        attrs,
    }
}

fn meta(name: &str, content: String) -> Tag {
    Tag {
        kind: TagKind::Meta,
        attrs: vec![("name", name.to_owned()), ("content", content)],
    }
}

/// All `<head>` tags, shared by the HTML, JSON and Nuxt snippets.
fn head_tags(cfg: &Config) -> Vec<Tag> {
    let settings = cfg.settings;
    // `sizes="32x32"` keeps browsers that support SVG favicons from preferring the .ico.
    let mut tags = vec![link(vec![
        ("rel", "icon".into()),
        ("href", cfg.url("favicon.ico")),
        ("sizes", "32x32".into()),
    ])];
    if cfg.has_svg {
        tags.push(link(vec![
            ("rel", "icon".into()),
            ("type", "image/svg+xml".into()),
            ("href", cfg.url("favicon.svg")),
        ]));
    }
    let icon_sizes: &[u32] = if settings.legacy {
        LEGACY_ICON_LINK_SIZES
    } else {
        &[ICON_SIZE]
    };
    for &size in icon_sizes {
        tags.push(link(vec![
            ("rel", "icon".into()),
            ("type", "image/png".into()),
            ("sizes", format!("{size}x{size}")),
            ("href", cfg.url(&png_name(size))),
        ]));
    }
    if settings.legacy {
        tags.push(link(vec![
            ("rel", "shortcut icon".into()),
            ("href", cfg.url("favicon.ico")),
        ]));
    }

    if settings.apple_touch_icon.is_some() {
        tags.push(link(vec![
            ("rel", "apple-touch-icon".into()),
            ("href", cfg.url("apple-touch-icon.png")),
        ]));
        if settings.legacy {
            for &size in LEGACY_APPLE_TOUCH_LINK_SIZES {
                tags.push(link(vec![
                    ("rel", "apple-touch-icon".into()),
                    ("sizes", format!("{size}x{size}")),
                    ("href", cfg.url(&png_name(size))),
                ]));
            }
        }
    }

    if let Some(manifest) = &settings.manifest {
        let mut attrs = vec![("rel", "manifest".into())];
        if let Some(crossorigin) = &manifest.crossorigin {
            attrs.push(("crossorigin", crossorigin.clone()));
        }
        attrs.push(("href", cfg.url("manifest.json")));
        tags.push(link(attrs));
    }

    if let Some(color) = &settings.theme_color {
        tags.push(meta("theme-color", color.clone()));
    }

    if let Some(windows) = &settings.windows {
        if let Some(color) = &windows.tile_color {
            tags.push(meta("msapplication-TileColor", color.clone()));
        }
        tags.push(meta(
            "msapplication-TileImage",
            cfg.url(&png_name(TILE_IMAGE_SIZE)),
        ));
        tags.push(meta("msapplication-config", cfg.url("browserconfig.xml")));
    }
    tags
}

pub fn html_snippet(cfg: &Config) -> String {
    head_tags(cfg)
        .iter()
        .map(|tag| {
            let name = match tag.kind {
                TagKind::Link => "link",
                TagKind::Meta => "meta",
            };
            let attrs: String = tag
                .attrs
                .iter()
                .map(|(k, v)| format!(" {k}=\"{}\"", escape(v)))
                .collect();
            format!("<{name}{attrs}>\n")
        })
        .collect()
}

/// The `<head>` tags as `{ "link": [...], "meta": [...] }` attribute objects.
fn head_json(cfg: &Config) -> Value {
    let (mut links, mut metas) = (Vec::new(), Vec::new());
    for tag in head_tags(cfg) {
        let object: serde_json::Map<String, Value> = tag
            .attrs
            .into_iter()
            .map(|(k, v)| (k.to_owned(), Value::String(v)))
            .collect();
        match tag.kind {
            TagKind::Link => links.push(Value::Object(object)),
            TagKind::Meta => metas.push(Value::Object(object)),
        }
    }
    json!({ "link": links, "meta": metas })
}

/// The `<head>` tags as JSON, for tools that inject them (e.g. the Nuxt module).
pub fn json_snippet(cfg: &Config) -> String {
    serde_json::to_string_pretty(&head_json(cfg)).expect("head serializes") + "\n"
}

/// A TypeScript module to spread into `app.head` of a `nuxt.config.ts`.
pub fn nuxt_snippet(cfg: &Config) -> String {
    format!(
        "// Generated by favicon-generator.\n\
         // Usage in nuxt.config.ts:\n\
         //   app: {{ head: {{ link: [...faviconHead.link], meta: [...faviconHead.meta] }} }}\n\
         export const faviconHead = {}\n",
        serde_json::to_string_pretty(&head_json(cfg)).expect("head serializes")
    )
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
