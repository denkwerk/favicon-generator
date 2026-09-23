//! Text assets: web app manifest, browserconfig.xml and head snippets.

use serde_json::{Value, json};

use crate::spec::{
    APPLE_TOUCH_LINK_SIZES, ICON_LINK_SIZES, MANIFEST_SIZES, TILE_IMAGE_SIZE, TILE_SIZES, png_name,
};

pub struct Config {
    pub path_prefix: String,
    pub name: String,
    pub short_name: String,
    pub description: String,
    pub theme_color: String,
    pub background_color: String,
    pub tile_color: String,
    pub start_url: String,
    pub scope: String,
    pub display: String,
    pub icon_purpose: String,
    pub manifest_crossorigin: Option<String>,
    pub has_svg: bool,
}

impl Config {
    fn url(&self, file: &str) -> String {
        format!("{}{file}", self.path_prefix)
    }
}

pub fn manifest(cfg: &Config) -> String {
    let icons: Vec<Value> = MANIFEST_SIZES
        .iter()
        .map(|&size| {
            json!({
                "src": cfg.url(&png_name(size)),
                "type": "image/png",
                "sizes": format!("{size}x{size}"),
                "purpose": cfg.icon_purpose,
            })
        })
        .collect();

    let manifest = json!({
        "name": cfg.name,
        "description": cfg.description,
        "short_name": cfg.short_name,
        "icons": icons,
        "scope": cfg.scope,
        "start_url": cfg.start_url,
        "display": cfg.display,
        "theme_color": cfg.theme_color,
        "background_color": cfg.background_color,
    });
    serde_json::to_string_pretty(&manifest).expect("manifest serializes") + "\n"
}

pub fn browserconfig(cfg: &Config) -> String {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<browserconfig>\n\t<msapplication>\n\t\t<tile>\n",
    );
    for size in TILE_SIZES {
        xml += &format!(
            "\t\t\t<square{size}x{size}logo src=\"{}\"/>\n",
            escape(&cfg.url(&png_name(size)))
        );
    }
    xml += &format!("\t\t\t<TileColor>{}</TileColor>\n", escape(&cfg.tile_color));
    xml += "\t\t</tile>\n\t</msapplication>\n</browserconfig>\n";
    xml
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

/// All `<head>` tags, shared by the HTML and Nuxt snippets.
fn head_tags(cfg: &Config) -> Vec<Tag> {
    let mut tags = vec![link(vec![
        ("rel", "apple-touch-icon".into()),
        ("href", cfg.url("apple-touch-icon.png")),
    ])];

    for size in APPLE_TOUCH_LINK_SIZES {
        tags.push(link(vec![
            ("rel", "apple-touch-icon".into()),
            ("sizes", format!("{size}x{size}")),
            ("href", cfg.url(&png_name(size))),
        ]));
    }

    if cfg.has_svg {
        tags.push(link(vec![
            ("rel", "icon".into()),
            ("type", "image/svg+xml".into()),
            ("href", cfg.url("favicon.svg")),
        ]));
    }

    for size in ICON_LINK_SIZES {
        tags.push(link(vec![
            ("rel", "icon".into()),
            ("type", "image/png".into()),
            ("sizes", format!("{size}x{size}")),
            ("href", cfg.url(&png_name(size))),
        ]));
    }

    for rel in ["shortcut icon", "icon"] {
        tags.push(link(vec![
            ("rel", rel.into()),
            ("type", "image/x-icon".into()),
            ("href", cfg.url("favicon.ico")),
        ]));
    }

    let mut manifest = vec![("rel", "manifest".into())];
    if let Some(crossorigin) = &cfg.manifest_crossorigin {
        manifest.push(("crossorigin", crossorigin.clone()));
    }
    manifest.push(("href", cfg.url("manifest.json")));
    tags.push(link(manifest));

    tags.push(meta("msapplication-TileColor", cfg.tile_color.clone()));
    tags.push(meta(
        "msapplication-TileImage",
        cfg.url(&png_name(TILE_IMAGE_SIZE)),
    ));
    tags.push(meta("msapplication-config", cfg.url("browserconfig.xml")));
    tags.push(meta("theme-color", cfg.theme_color.clone()));
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

/// A TypeScript module to spread into `app.head` of a `nuxt.config.ts`.
pub fn nuxt_snippet(cfg: &Config) -> String {
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
    let head = json!({ "link": links, "meta": metas });
    format!(
        "// Generated by favicon-generator.\n\
         // Usage in nuxt.config.ts:\n\
         //   app: {{ head: {{ link: [...faviconHead.link], meta: [...faviconHead.meta] }} }}\n\
         export const faviconHead = {}\n",
        serde_json::to_string_pretty(&head).expect("head serializes")
    )
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
