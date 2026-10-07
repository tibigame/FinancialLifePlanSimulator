use serde_json::Value;

pub struct TextResources(Value);

impl TextResources {
    pub fn load() -> Result<Self, serde_json::Error> {
        serde_json::from_str(include_str!("../../resources/ja.json")).map(Self)
    }

    pub fn get<'a>(&'a self, key: &'a str) -> &'a str {
        let mut value = &self.0;
        for part in key.split('.') {
            match value.get(part) {
                Some(next) => value = next,
                None => return key,
            }
        }
        value.as_str().map_or(key, |text| text)
    }
}
