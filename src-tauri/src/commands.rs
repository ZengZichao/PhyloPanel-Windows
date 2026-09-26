use base64::Engine;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use tauri::{command, AppHandle, Manager};

use crate::bundle;
use crate::error;
use crate::runner::{self, RunResult, Step};
use crate::schema::{Reflector, ToolSchema};
use crate::toolpack::{self, ToolPack};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reflected {
    pub schema: ToolSchema,
    pub pack: ToolPack,
    pub from_cache: bool,
}

/// Reflection means roughly one `--help` process per command, so it is cached against the
/// binary's modification time and reused until the tool is upgraded.
fn cache_path(app: &AppHandle, binary: &Path) -> Result<PathBuf, String> {
    let dir = app.path().app_cache_dir().map_err(|e| error("reflect", e))?;
    std::fs::create_dir_all(&dir).map_err(|e| error("reflect", e))?;
    let stamp = std::fs::metadata(binary)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let name: String = binary
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "tool".into())
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' { c } else { '_' })
        .collect();
    Ok(dir.join(format!("{}-{}.json", name, stamp)))
}

/// Unpacks the embedded gotree and returns its path, so the panel works with zero setup.
#[command]
pub async fn bundled_tool(app: AppHandle) -> Result<String, String> {
    let root = app.path().app_data_dir().map_err(|e| error("bundledExtract", e))?;
    let path = tauri::async_runtime::spawn_blocking(move || bundle::ensure_bundled(&root))
        .await
        .map_err(|e| error("bundledExtract", e))??;
    Ok(path.display().to_string())
}

/// Renders the argv the panel would execute, so the shell line shown in the UI can never
/// drift from what the runner actually does.
#[command]
pub fn preview_command(steps: Vec<Step>) -> String {
    runner::command_line(&steps)
}

#[command]
pub async fn reflect(app: AppHandle, binary: String, force: bool) -> Result<Reflected, String> {
    let path = PathBuf::from(&binary);
    if !path.is_file() {
        return Err(error("binaryMissing", binary));
    }
    let cache = cache_path(&app, &path)?;
    let pack = toolpack::pack_for(&path);

    if !force {
        if let Ok(text) = std::fs::read_to_string(&cache) {
            if let Ok(schema) = serde_json::from_str::<ToolSchema>(&text) {
                return Ok(Reflected { schema, pack, from_cache: true });
            }
        }
    }

    let blocking = path.clone();
    let schema = tauri::async_runtime::spawn_blocking(move || Reflector::new(&blocking).reflect())
        .await
        .map_err(|e| error("reflect", e))?;
    let schema = schema.map_err(|e| error("reflect", e))?;

    if let Ok(text) = serde_json::to_string(&schema) {
        let _ = std::fs::write(&cache, text);
    }
    Ok(Reflected { schema, pack, from_cache: false })
}

#[command]
pub async fn run_pipeline(
    binary: String,
    steps: Vec<Step>,
    input_text: Option<String>,
) -> Result<RunResult, String> {
    let path = PathBuf::from(&binary);
    if !path.is_file() {
        return Err(error("binaryMissing", binary));
    }
    tauri::async_runtime::spawn_blocking(move || {
        runner::run_pipeline(&path, &steps, input_text.as_deref())
    })
    .await
    .map_err(|e| error("execute", e))?
}

#[command]
pub async fn write_text(path: String, text: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        std::fs::write(&path, text).map_err(|e| error("write", e))
    })
    .await
    .map_err(|e| error("write", e))?
}

#[command]
pub async fn write_base64(path: String, data: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&data)
            .map_err(|e| error("base64", e))?;
        std::fs::write(&path, bytes).map_err(|e| error("write", e))
    })
    .await
    .map_err(|e| error("write", e))?
}
