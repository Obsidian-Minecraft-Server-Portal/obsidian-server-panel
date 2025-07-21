use serde::{Deserialize, Deserializer};

fn json_array_string<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    serde_json::from_str(&s).map_err(serde::de::Error::custom)
}

#[derive(Deserialize)]
pub struct DownloadParameters {
    #[serde(deserialize_with = "json_array_string")]
    pub items: Vec<String>,
}
