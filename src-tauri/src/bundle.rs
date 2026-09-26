use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error;

/// FNV-1a, used only to name the extracted file after its contents.
fn fingerprint(bytes: &[u8]) -> u32 {
    let mut hash: u32 = 0x811c_9dc5;
    for byte in bytes {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

/// The shipped experience is one executable: gotree is embedded in PhyloPanel and unpacked
/// into app data on first launch, so users never have to install the CLI separately.
pub fn ensure_bundled(root: &Path) -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        const BYTES: &[u8] = include_bytes!("../tools/gotree.exe");
        let dir = root.join("tools");
        std::fs::create_dir_all(&dir).map_err(|e| error("toolDirCreate", e))?;
        let name = format!("gotree-{:08x}.exe", fingerprint(BYTES));
        let target = dir.join(&name);
        let fresh = std::fs::metadata(&target).map(|m| m.len() as usize == BYTES.len()).unwrap_or(false);
        if !fresh {
            // Write to a scratch name first: a crashed install must never look like a valid tool.
            let temp = dir.join(format!("{}.part", name));
            let mut file = std::fs::File::create(&temp).map_err(|e| error("toolWrite", e))?;
            file.write_all(BYTES).map_err(|e| error("toolWrite", e))?;
            file.flush().map_err(|e| error("toolWrite", e))?;
            drop(file);
            std::fs::rename(&temp, &target).map_err(|e| error("toolPlace", e))?;
        }
        Ok(target)
    }
    #[cfg(not(windows))]
    {
        let _ = root;
        Err(error("bundledWindowsOnly", ""))
    }
}
