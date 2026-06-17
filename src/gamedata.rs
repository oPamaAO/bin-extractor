//! GameData directory discovery and `.bin` file listing.

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Common paths to search for Albion Online GameData on Windows.
const COMMON_PATHS: &[&str] = &[
    // Steam install
    r"C:\Program Files (x86)\Steam\steamapps\common\Albion Online\game\Albion-Online_Data\StreamingAssets\GameData",
    // Standalone launcher
    r"C:\Program Files (x86)\AlbionOnline\game\Albion-Online_Data\StreamingAssets\GameData",
    // Local AppData (Albion Launcher download cache)
    r"C:\Users\secto\AppData\Local\AlbionOnline\game\Albion-Online_Data\StreamingAssets\GameData",
];

/// Auto-detect GameData directory by checking common install paths.
pub fn auto_detect_gamedata() -> Option<PathBuf> {
    for path_str in COMMON_PATHS {
        let p = Path::new(path_str);
        if p.join("items.bin").exists() && p.join("mobs.bin").exists() {
            return Some(p.to_path_buf());
        }
    }
    None
}

/// Information about a single `.bin` file.
#[derive(Debug)]
pub struct BinFileInfo {
    pub name: String,
    #[allow(dead_code)]
    pub stem: String,
    pub size_bytes: u64,
    pub known_type: bool,
}

/// Known extractable types and their human-readable descriptions.
pub const KNOWN_TYPES: &[(&str, &str, &str)] = &[
    ("items.bin",     "items",     "Items database (40k+ items)"),
    ("mobs.bin",      "mobs",      "Mobs database (4k+ creatures)"),
    ("spells.bin",    "spells",    "Spells/abilities database"),
    ("buildings.bin", "buildings", "Building definitions"),
    ("localization.bin", "localization", "Localization strings (11MB TMX)"),
    ("gamedata.bin",  "gamedata",  "Game settings key-value pairs"),
    ("world.bin",     "world",     "World cluster definitions"),
];

/// Check if a filename has a known extractor.
pub fn is_known_bin(name: &str) -> bool {
    KNOWN_TYPES.iter().any(|(filename, _, _)| *filename == name)
}

/// Get the extract command name for a known bin file.
pub fn type_for_bin(name: &str) -> Option<&'static str> {
    KNOWN_TYPES.iter().find(|(f, _, _)| *f == name).map(|(_, cmd, _)| *cmd)
}

/// List all `.bin` files in a GameData directory with metadata.
pub fn list_bin_files(gamedata: &Path) -> Result<Vec<BinFileInfo>> {
    let mut files = Vec::new();

    // Scan root GameData
    if gamedata.is_dir() {
        for entry in std::fs::read_dir(gamedata)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "bin") {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let stem = path.file_stem().unwrap().to_string_lossy().to_string();
                let meta = std::fs::metadata(&path)?;
                let known = is_known_bin(&name);
                files.push(BinFileInfo {
                    name,
                    stem,
                    size_bytes: meta.len(),
                    known_type: known,
                });
            }
        }
    }

    // Also scan cluster/ subdirectory
    let cluster_dir = gamedata.join("cluster");
    if cluster_dir.is_dir() {
        for entry in std::fs::read_dir(&cluster_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "bin") {
                let name = format!("cluster/{}", path.file_name().unwrap().to_string_lossy());
                let stem = path.file_stem().unwrap().to_string_lossy().to_string();
                let meta = std::fs::metadata(&path)?;
                files.push(BinFileInfo {
                    name,
                    stem,
                    size_bytes: meta.len(),
                    known_type: is_known_bin(&path.file_name().unwrap().to_string_lossy()),
                });
            }
        }
    }

    // Sort: known types first, then alphabetically
    files.sort_by(|a, b| {
        let a_known = if a.known_type { 0 } else { 1 };
        let b_known = if b.known_type { 0 } else { 1 };
        a_known.cmp(&b_known).then(a.name.cmp(&b.name))
    });

    Ok(files)
}
