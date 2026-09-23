//! Fetches a node as SVG through the Figma REST API
//! (<https://developers.figma.com/docs/rest-api/>).

use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde_json::Value;

const API_BASE: &str = "https://api.figma.com/v1";

/// A node inside a Figma file, parsed from a share link.
#[derive(Debug, PartialEq)]
pub struct NodeRef {
    pub file_key: String,
    /// API form, e.g. `19938:42` (links use `19938-42`).
    pub node_id: String,
}

impl NodeRef {
    /// Parses links like `https://www.figma.com/design/<key>/<name>?node-id=1-2`.
    /// Also accepts `/file/`, `/proto/`, `/board/` and branch links.
    pub fn parse_url(url: &str) -> Result<Self> {
        let rest = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .unwrap_or(url);
        let rest = rest.strip_prefix("www.").unwrap_or(rest);
        let Some(rest) = rest.strip_prefix("figma.com/") else {
            bail!("not a figma.com URL: {url}");
        };

        let (path, query) = rest.split_once('?').unwrap_or((rest, ""));
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let file_key = match segments.as_slice() {
            // Branches have their own file key.
            [_, _, "branch", branch_key, ..] => branch_key,
            [kind, key, ..] if matches!(*kind, "design" | "file" | "proto" | "board") => key,
            _ => bail!("could not find a file key in {url}"),
        };

        let node_id = query
            .split('&')
            .find_map(|pair| pair.strip_prefix("node-id="))
            .map(|id| id.replace("%3A", ":").replace("%3a", ":").replace('-', ":"))
            .with_context(|| {
                format!("{url} has no node-id; select the component in Figma and copy its link")
            })?;

        Ok(NodeRef {
            file_key: (*file_key).to_owned(),
            node_id,
        })
    }
}

/// Where the personal access token comes from, in order of precedence.
pub fn resolve_token(flag: Option<&str>, file: Option<&Path>) -> Result<String> {
    let token = match (flag, file) {
        (Some(token), _) => token.to_owned(),
        (None, Some(path)) => std::fs::read_to_string(path)
            .with_context(|| format!("failed to read Figma token from {}", path.display()))?,
        (None, None) => bail!(
            "a Figma access token is required: set FIGMA_TOKEN, or pass --figma-token / --figma-token-file \
             (create one under Figma → Settings → Security → Personal access tokens, scope `file_content:read`)"
        ),
    };
    let token = token.trim().to_owned();
    if token.is_empty() {
        bail!("the Figma access token is empty");
    }
    Ok(token)
}

/// Exports `node` as SVG and downloads it.
pub fn fetch_svg(node: &NodeRef, token: &str) -> Result<Vec<u8>> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .timeout_global(Some(Duration::from_secs(60)))
        .build()
        .into();

    // Step 1: ask Figma to render the node; it answers with a short-lived download URL.
    let mut response = agent
        .get(format!("{API_BASE}/images/{}", node.file_key))
        .header("X-Figma-Token", token)
        .query("ids", &node.node_id)
        .query("format", "svg")
        .query("svg_outline_text", "true")
        .query("svg_include_id", "false")
        .call()
        .context("failed to reach the Figma API")?;
    let status = response.status();
    let body = response
        .body_mut()
        .read_to_string()
        .context("failed to read the Figma API response")?;
    let json: Value = serde_json::from_str(&body).unwrap_or(Value::Null);

    if !status.is_success() {
        let message = json["err"]
            .as_str()
            .or_else(|| json["message"].as_str())
            .unwrap_or(body.trim());
        let hint = match status.as_u16() {
            403 => " (check that the token is valid and has access to this file)",
            404 => " (check the file key in the URL)",
            429 => " (rate limited; try again later)",
            _ => "",
        };
        bail!("Figma API returned {status}: {message}{hint}");
    }
    if let Some(err) = json["err"].as_str() {
        bail!("Figma API error: {err}");
    }

    let Some(image_url) = json["images"][&node.node_id].as_str() else {
        bail!(
            "Figma could not render node {} (it may not exist or be invisible); response: {body}",
            node.node_id
        );
    };

    // Step 2: download the rendered SVG (a pre-signed URL; no token needed).
    let mut response = agent
        .get(image_url)
        .call()
        .context("failed to download the exported SVG")?;
    if !response.status().is_success() {
        bail!(
            "downloading the exported SVG failed with {}",
            response.status()
        );
    }
    let svg = response
        .body_mut()
        .read_to_vec()
        .context("failed to read the exported SVG")?;
    Ok(svg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_design_url() {
        let node = NodeRef::parse_url(
            "https://www.figma.com/design/77SgSAGXbDv1Eye6htYdCG/ONE---Assets-Library-NEW?node-id=19938-42&t=64SaIOxvfLuKPsWX-1",
        )
        .unwrap();
        assert_eq!(
            node,
            NodeRef {
                file_key: "77SgSAGXbDv1Eye6htYdCG".into(),
                node_id: "19938:42".into()
            }
        );
    }

    #[test]
    fn parses_branch_and_encoded_ids() {
        let node =
            NodeRef::parse_url("figma.com/design/ABC/branch/BR4NCH/Name?node-id=1%3A2").unwrap();
        assert_eq!(
            node,
            NodeRef {
                file_key: "BR4NCH".into(),
                node_id: "1:2".into()
            }
        );
    }

    #[test]
    fn rejects_missing_node_id() {
        assert!(NodeRef::parse_url("https://www.figma.com/design/ABC/Name").is_err());
        assert!(NodeRef::parse_url("https://example.com/design/ABC/Name?node-id=1-2").is_err());
    }
}
