//! Locations of the modern save files (quick save, autosave).

use std::path::PathBuf;

pub fn quick_path() -> PathBuf {
    crate::sys::user_dir().join("quicksave.sav")
}

pub fn auto_path() -> PathBuf {
    crate::sys::user_dir().join("autosave.sav")
}

/// Most recent of quick save and autosave, for CONTINUE.
pub fn latest() -> Option<PathBuf> {
    [quick_path(), auto_path()]
        .into_iter()
        .filter_map(|p| std::fs::metadata(&p).and_then(|m| m.modified()).ok().map(|t| (t, p)))
        .max_by_key(|(t, _)| *t)
        .map(|(_, p)| p)
}
