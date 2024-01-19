use rocket::serde::Deserialize;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct DbConfig {
    version: String
}

impl DbConfig {
    pub fn version(&self) -> String {
        self.version.clone()
    }
}
