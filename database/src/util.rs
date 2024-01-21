use crate::prelude::*;
use ini::Ini;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// get uuid
pub fn generate_uuid() -> Uuid {
    Uuid::new_v4()
}

/// return time since epoch
pub fn get_unix_epoch_time() -> u64 {
    let current_time = SystemTime::now();
    let duration_since_epoch = current_time.duration_since(UNIX_EPOCH).unwrap_or_default();
    duration_since_epoch.as_secs()
}

pub fn is_directory_within_parent(child: &str, parent: &str) -> bool {
    let child_path = Path::new(child).canonicalize().ok();
    let parent_path = Path::new(parent).canonicalize().ok();

    match (child_path, parent_path) {
        (Some(child_canonical), Some(parent_canonical)) => {
            child_canonical.starts_with(parent_canonical)
        }
        _ => false,
    }
}

/// read value from config file
pub fn read_config(section: &str, key: &str) -> String {
    if let Ok(conf) = Ini::load_from_file(CONFIG_FILE_PATH) {
        if let Some(section) = conf.section(Some(section)) {
            if let Some(value) = section.get(key) {
                return value.into();
            }
        }
    }
    Default::default()
}

/// write value into config file
pub fn write_config(section: &str, key: &str, new_value: &str) -> bool {
    if let Ok(mut conf) = Ini::load_from_file(CONFIG_FILE_PATH) {
        // Set a value:
        conf.set_to(Some(section), key.into(), new_value.into());
        if let Ok(_) = conf.write_to_file(CONFIG_FILE_PATH) {
            return true;
        }
    }
    false
}
