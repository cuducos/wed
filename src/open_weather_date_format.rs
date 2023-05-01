use chrono::NaiveDateTime;
use serde::{self, Deserialize, Deserializer, Serializer};

const OPEN_WEATHER_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&s, OPEN_WEATHER_DATE_FORMAT).map_err(serde::de::Error::custom)
}

pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = format!("{}", date.format(OPEN_WEATHER_DATE_FORMAT));
    serializer.serialize_str(&s)
}
