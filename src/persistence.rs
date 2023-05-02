use std::{fs::File, io::BufReader, path::PathBuf};

use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{open_weather_date_format, Event};

const FILE_NAME: &str = ".wed";

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct SavedEvent {
    pub name: String,
    pub location: String,
    pub latitude: f64,
    pub longitude: f64,

    #[serde(with = "open_weather_date_format")]
    pub when: NaiveDateTime,
}

impl SavedEvent {
    pub fn to_event(&self) -> Event {
        Event {
            when: self.when,
            location: self.location.clone(),
            latitude: self.latitude,
            longitude: self.longitude,
            days: (self.when - chrono::Local::now().naive_local()).num_days(),
        }
    }

    pub fn from_event(event: &Event, name: String) -> Self {
        Self {
            name,
            location: event.location.clone(),
            latitude: event.latitude,
            longitude: event.longitude,
            when: event.when,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SavedEvents {
    pub events: Vec<SavedEvent>,
}

fn storage_path() -> Result<PathBuf> {
    let mut path = home::home_dir().ok_or(anyhow!("Couldn't find home directory"))?;
    path.push(FILE_NAME);
    Ok(path)
}

impl SavedEvents {
    pub fn new() -> Self {
        Self { events: vec![] }
    }

    pub fn from_file() -> Result<Self> {
        let file = File::open(storage_path()?)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    pub fn to_file(&self) -> Result<()> {
        let file = File::create(storage_path()?)?;
        Ok(serde_json::to_writer(file, self)?)
    }

    pub fn add(&mut self, event: SavedEvent) {
        if self.events.contains(&event) {
            return;
        }
        self.events.push(event);
    }
}

impl Default for SavedEvents {
    fn default() -> Self {
        Self::new()
    }
}
