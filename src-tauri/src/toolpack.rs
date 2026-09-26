use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

use crate::runner::Step;

/// A command that reads bare arguments. Cobra never advertises this, so it has to be
/// curated per tool: `<exe> somecmd extra` silently works on 4 gotree commands and is
/// silently ignored on the other 78.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionalSpec {
    pub label: String,
    #[serde(default)]
    pub label_zh: Option<String>,
    pub help: String,
    #[serde(default)]
    pub help_zh: Option<String>,
    #[serde(default)]
    pub minimum: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Template {
    pub name: String,
    #[serde(default)]
    pub name_zh: Option<String>,
    pub description: String,
    #[serde(default)]
    pub description_zh: Option<String>,
    pub steps: Vec<Step>,
    /// Command ids that end the chain by consuming the tree instead of passing it on.
    #[serde(default)]
    pub terminal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolPack {
    pub display_name: String,
    #[serde(default)]
    pub positional: BTreeMap<String, PositionalSpec>,
    #[serde(default)]
    pub templates: Vec<Template>,
    /// Command ids whose stdout is a picture rather than a tree or a table.
    #[serde(default)]
    pub image_commands: Vec<String>,
}

impl Default for ToolPack {
    fn default() -> Self {
        Self {
            display_name: String::new(),
            positional: BTreeMap::new(),
            templates: Vec::new(),
            image_commands: vec!["draw svg".into(), "draw png".into(), "draw cyjs".into()],
        }
    }
}

const GOTREE: &str = include_str!("../toolpacks/gotree.json");

/// Picks the pack by matching the binary's file name. Unknown binaries still work --
/// they just get no presets and no positional-argument hints.
pub fn pack_for(binary: &Path) -> ToolPack {
    let stem = binary
        .file_stem()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let mut pack: ToolPack = if stem.starts_with("gotree") {
        serde_json::from_str(GOTREE).unwrap_or_default()
    } else {
        ToolPack::default()
    };
    if pack.display_name.is_empty() {
        pack.display_name = stem;
    }
    pack
}
