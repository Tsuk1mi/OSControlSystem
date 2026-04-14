use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::gesture_os_control::domain::entities::command::OsCommand;
use crate::gesture_os_control::domain::services::command_mapper::GestureCommandMap;
use crate::gesture_os_control::domain::value_objects::gesture_id::GestureId;

const FILE_NAME: &str = "gesture_bindings.json";

#[derive(Serialize, Deserialize, Default)]
struct GestureBindingsFile {
    #[serde(default)]
    desktop: HashMap<String, String>,
    #[serde(default)]
    media: HashMap<String, String>,
    #[serde(default)]
    browser: HashMap<String, String>,
}

pub fn bindings_path() -> Option<PathBuf> {
    let mut dir = std::env::current_exe().ok()?;
    dir.pop();
    Some(dir.join(FILE_NAME))
}

fn apply_overrides(table: &mut HashMap<GestureId, OsCommand>, raw: &HashMap<String, String>) {
    for (gk, ck) in raw {
        let Some(gesture) = GestureId::parse_wire_key(gk) else {
            continue;
        };
        let Some(cmd) = OsCommand::parse_wire_key(ck) else {
            continue;
        };
        table.insert(gesture, cmd);
    }
}

pub fn load_merged_with_defaults() -> GestureCommandMap {
    let mut map = GestureCommandMap::app_defaults();
    let Some(path) = bindings_path() else {
        return map;
    };
    let Ok(text) = fs::read_to_string(&path) else {
        return map;
    };
    let Ok(file) = serde_json::from_str::<GestureBindingsFile>(&text) else {
        return map;
    };
    apply_overrides(&mut map.desktop, &file.desktop);
    apply_overrides(&mut map.media, &file.media);
    apply_overrides(&mut map.browser, &file.browser);
    map
}

fn table_to_strings(table: &HashMap<GestureId, OsCommand>) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for (&g, &c) in table {
        if let Some(gk) = g.wire_key() {
            out.insert(gk.to_owned(), c.wire_key().to_owned());
        }
    }
    out
}

pub fn save(map: &GestureCommandMap) -> Result<(), String> {
    let path = bindings_path().ok_or_else(|| "Не удалось определить путь к exe.".to_owned())?;
    save_to_path(map, &path)
}

pub fn save_to_path(map: &GestureCommandMap, path: &Path) -> Result<(), String> {
    let file = GestureBindingsFile {
        desktop: table_to_strings(&map.desktop),
        media: table_to_strings(&map.media),
        browser: table_to_strings(&map.browser),
    };
    let text = serde_json::to_string_pretty(&file).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| e.to_string())
}
