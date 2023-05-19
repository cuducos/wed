use chrono::NaiveDateTime;
use serde::{self, Deserialize, Deserializer, Serializer};

pub const OPEN_METEO_DATE_FORMAT: &str = "%Y-%m-%dT%H:%M";
const WED_FILE_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&s, WED_FILE_DATE_FORMAT).map_err(serde::de::Error::custom)
}

pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = format!("{}", date.format(WED_FILE_DATE_FORMAT));
    serializer.serialize_str(&s)
}
