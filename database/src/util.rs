/// Copyright (c) Algorealm, Inc.
use crate::prelude::*;
use ini::Ini;
use rand::Rng;
use rocket::serde::json::Value;
use std::borrow::Cow;
use std::path::Path;
use std::process;
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
pub fn read_config(section: &str, key: &str) -> Cow<'static, str> {
    if let Ok(conf) = Ini::load_from_file(CONFIG_FILE_PATH) {
        if let Some(section) = conf.section(Some(section)) {
            if let Some(value) = section.get(key) {
                return Cow::Owned(value.to_owned());
            }
        }
    }

    "".into()
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

/// generate document rev
pub fn generate_rev(n: u64, data: &str) -> String {
    // Calculate MD5 hash
    let hash = md5::compute(data);

    // Convert the hash to a hexadecimal string
    let hash_str = format!("{:x}", hash);

    // Append the hash to the original number
    let rev = format!("{}-{}", n, hash_str);

    rev
}

/// produce a simple hash
pub fn hash_string(data: &str) -> Cow<'static, str> {
    // Calculate MD5 hash
    let hash = md5::compute(data);

    // Convert the hash to a hexadecimal string
    format!("{:x}", hash).into()
}

/// merge two json values together
pub fn merge_json_values(mut base: Value, mut override_value: Value) -> Value {
    match (base.as_object_mut(), override_value.as_object_mut()) {
        (Some(base_obj), Some(override_obj)) => {
            // Iterate over the fields of the override object
            for (key, value) in override_obj.iter_mut() {
                // Insert or update the field in the base object
                base_obj.insert(key.clone(), value.take());
            }
        }
        _ => {
            // If either of them is not an object, return the override value
            return override_value;
        }
    }

    base
}

/// remove a field from a Value and return it
pub fn remove_field(mut value: Value, field_name: &str) -> (Value, Option<Value>) {
    let mut removed_field = None;

    match value {
        Value::Object(ref mut map) => {
            // Check if the field exists
            if let Some(old_field) = map.remove(field_name) {
                removed_field = Some(old_field);
            }
        }
        _ => {} // Do nothing if the value is not an object
    }

    (value, removed_field)
}

pub fn generate_strong_password(length: usize) -> String {
    let characters = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    let strong_password: String = (0..length)
        .map(|_| {
            characters
                .chars()
                .nth(rng.gen_range(0..characters.len()))
                .unwrap()
        })
        .collect();
    strong_password
}

// check for important config and refuse to start the database if they are not set\
pub fn check_start_config() -> String {
    // read in blockchain config
    let mnemonic = read_config("contract", "mnemonic");
    if mnemonic.is_empty() {
        // kill process
        println!("Please check the config.ini file and input a mnemonic for your funded application account.");
        process::exit(2);
    }

    mnemonic.to_string()
}
