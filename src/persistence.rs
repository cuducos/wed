use std::{fs::File, io::BufReader, path::PathBuf};

use anyhow::{anyhow, Result};
use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::{date_format, Event};

const OLD_FILE_NAME: &str = ".wed";
const FILE_NAME: &str = ".wed.json";

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct SavedEvent {
    pub name: String,
    pub location: String,
    pub latitude: f64,
    pub longitude: f64,

    #[serde(with = "date_format")]
    pub when: NaiveDateTime,
}

impl SavedEvent {
    pub fn to_event(&self) -> Event {
        Event {
            name: Some(self.name.clone()),
            when: self.when,
            location: self.location.clone(),
            latitude: self.latitude,
            longitude: self.longitude,
            days: (self.when - chrono::Local::now().naive_local()).num_days(),
        }
    }

    pub fn from_event(event: &Event) -> Result<Self> {
        let name = match &event.name {
            Some(name) => name.clone(),
            None => return Err(anyhow!("Cannot create an event without a name")),
        };

        Ok(Self {
            name,
            location: event.location.clone(),
            latitude: event.latitude,
            longitude: event.longitude,
            when: event.when,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SavedEvents {
    pub events: Vec<SavedEvent>,
}

fn default_storage_path() -> Result<PathBuf> {
    let mut path = home::home_dir().ok_or(anyhow!("Couldn't find home directory"))?;
    path.push(FILE_NAME);
    Ok(path)
}

fn update_file_name(path: &PathBuf) -> Result<()> {
    if let Some(dir) = path.parent() {
        let old = dir.join(OLD_FILE_NAME);
        if old.exists() {
            std::fs::rename(&old, path)?;
        }
    }
    Ok(())
}

impl SavedEvents {
    pub fn new() -> Self {
        Self { events: vec![] }
    }

    pub fn from_file_path(path: &PathBuf) -> Result<Self> {
        update_file_name(path)?;
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut saved: Self = serde_json::from_reader(reader)?;

        let count = saved.events.len();
        saved.cleanup();
        if saved.events.len() != count {
            saved.to_file_path(path)?;
        }

        Ok(saved)
    }

    pub fn from_file() -> Result<Self> {
        Self::from_file_path(&default_storage_path()?)
    }

    pub fn to_file_path(&mut self, path: &PathBuf) -> Result<()> {
        update_file_name(path)?;
        self.cleanup();
        let file = File::create(path)?;
        Ok(serde_json::to_writer(file, self)?)
    }

    pub fn to_file(&mut self) -> Result<()> {
        self.to_file_path(&default_storage_path()?)
    }

    pub fn add(&mut self, event: SavedEvent) {
        if self.events.contains(&event) {
            return;
        }
        self.events.push(event);
    }

    fn cleanup(&mut self) {
        let now = Local::now().naive_local();
        self.events.retain(|event| event.when > now);
        self.events.sort_by(|a, b| a.when.cmp(&b.when));
    }
}

impl Default for SavedEvents {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use std::io::Write;
    use tempdir::TempDir;

    fn create_temp(name: &str) -> (PathBuf, TempDir) {
        let temp_dir = TempDir::new("wed-test-saved-events").unwrap();
        let file_path = temp_dir.path().join(name);
        (file_path, temp_dir)
    }
    fn create_old_temp_file() -> (PathBuf, TempDir) {
        create_temp(OLD_FILE_NAME)
    }
    fn create_temp_file() -> (PathBuf, TempDir) {
        create_temp(FILE_NAME)
    }

    #[test]
    fn test_saved_event_to_event() {
        let saved_event = SavedEvent {
            name: "Event Name".to_string(),
            location: "Event Location".to_string(),
            latitude: 42.0,
            longitude: -73.0,
            when: NaiveDateTime::parse_from_str("2021-05-20 8:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        };
        let current_time = Local::now().naive_local();
        let expected_event = Event {
            name: Some("Event Name".to_string()),
            when: saved_event.when,
            location: "Event Location".to_string(),
            latitude: 42.0,
            longitude: -73.0,
            days: (saved_event.when - current_time).num_days(),
        };

        assert_eq!(saved_event.to_event(), expected_event);
    }

    #[test]
    fn test_saved_event_from_event_successful() {
        let event = Event {
            name: Some("Event Name".to_string()),
            when: NaiveDateTime::parse_from_str("2021-05-20 8:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            location: "Event Location".to_string(),
            latitude: 42.0,
            longitude: -73.0,
            days: 10,
        };
        let expected_saved_event = SavedEvent {
            name: "Event Name".to_string(),
            location: "Event Location".to_string(),
            latitude: 42.0,
            longitude: -73.0,
            when: NaiveDateTime::parse_from_str("2021-05-20 8:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        };
        let result = SavedEvent::from_event(&event);
        assert!(result.is_ok());

        let saved_event = result.unwrap();
        assert_eq!(saved_event, expected_saved_event);
    }

    #[test]
    fn test_saved_event_from_event_without_name() {
        let event = Event {
            name: None,
            when: NaiveDateTime::parse_from_str("2021-05-20 8:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            location: "Event Location".to_string(),
            latitude: 42.0,
            longitude: -73.0,
            days: 10,
        };

        let result = SavedEvent::from_event(&event);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Cannot create an event without a name"
        );
    }

    #[test]
    fn test_saved_events_from_file_path_successful() {
        let (path, tmp) = create_temp_file();
        let events = vec![
            SavedEvent {
                name: "Event 1".to_string(),
                location: "Location 1".to_string(),
                latitude: 42.0,
                longitude: -73.0,
                when: Local::now().naive_local() + Duration::try_days(1).unwrap(),
            },
            SavedEvent {
                name: "Event 2".to_string(),
                location: "Location 2".to_string(),
                latitude: 42.0,
                longitude: -73.0,
                when: Local::now().naive_local() + Duration::try_days(2).unwrap(),
            },
        ];
        serde_json::to_writer(File::create(&path).unwrap(), &SavedEvents { events }).unwrap();

        let saved_events = SavedEvents::from_file_path(&path).unwrap();
        tmp.close().unwrap();

        assert_eq!(saved_events.events.len(), 2);
        assert_eq!(saved_events.events[0].name, "Event 1");
        assert_eq!(saved_events.events[1].name, "Event 2");
    }

    #[test]
    fn test_saved_events_from_file_path_with_old_event() {
        let (path, tmp) = create_temp_file();
        let events = vec![
            SavedEvent {
                name: "Event 1".to_string(),
                location: "Location 1".to_string(),
                latitude: 42.0,
                longitude: -73.0,
                when: Local::now().naive_local() - Duration::try_days(1).unwrap(),
            },
            SavedEvent {
                name: "Event 2".to_string(),
                location: "Location 2".to_string(),
                latitude: 42.0,
                longitude: -73.0,
                when: Local::now().naive_local() + Duration::try_days(2).unwrap(),
            },
        ];
        serde_json::to_writer(File::create(&path).unwrap(), &SavedEvents { events }).unwrap();

        let saved_events = SavedEvents::from_file_path(&path).unwrap();
        tmp.close().unwrap();

        assert_eq!(saved_events.events.len(), 1);
        assert_eq!(saved_events.events[0].name, "Event 2");
    }

    #[test]
    fn test_saved_events_from_invalid_file() {
        let (_, tmp) = create_temp_file();
        let result = SavedEvents::from_file_path(&tmp.path().join("invalid.json"));
        tmp.close().unwrap();

        assert!(result.is_err());

        let error = result.unwrap_err().to_string();
        let expected = if cfg!(windows) {
            "The system cannot find the file specified. (os error 2)"
        } else {
            "No such file or directory (os error 2)"
        };
        assert_eq!(error, expected);
    }

    #[test]
    fn test_saved_events_to_file_path_successful() {
        let (path, tmp) = create_temp_file();
        let mut saved_events = SavedEvents::new();
        saved_events.add(SavedEvent {
            name: "Event 1".to_string(),
            location: "Location 1".to_string(),
            latitude: 42.0,
            longitude: -73.0,
            when: Local::now().naive_local() + Duration::try_days(1).unwrap(),
        });
        saved_events.add(SavedEvent {
            name: "Event 2".to_string(),
            location: "Location 2".to_string(),
            latitude: 42.0,
            longitude: -73.0,
            when: Local::now().naive_local() + Duration::try_days(2).unwrap(),
        });
        saved_events.to_file_path(&path).unwrap();
        let loaded = SavedEvents::from_file_path(&path).unwrap();
        tmp.close().unwrap();

        assert_eq!(loaded.events.len(), 2);
        assert_eq!(loaded.events[0].name, "Event 1");
        assert_eq!(loaded.events[1].name, "Event 2");
    }

    #[test]
    fn test_saved_events_to_file_path_with_old_event() {
        let (path, tmp) = create_temp_file();
        let mut saved_events = SavedEvents::new();
        saved_events.add(SavedEvent {
            name: "Event 1".to_string(),
            location: "Location 1".to_string(),
            latitude: 42.0,
            longitude: -73.0,
            when: Local::now().naive_local() - Duration::try_days(1).unwrap(),
        });
        saved_events.add(SavedEvent {
            name: "Event 2".to_string(),
            location: "Location 2".to_string(),
            latitude: 42.0,
            longitude: -73.0,
            when: Local::now().naive_local() + Duration::try_days(2).unwrap(),
        });
        saved_events.to_file_path(&path).unwrap();
        let loaded = SavedEvents::from_file_path(&path).unwrap();
        tmp.close().unwrap();

        assert_eq!(loaded.events.len(), 1);
        assert_eq!(loaded.events[0].name, "Event 2");
    }

    #[test]
    fn test_saved_events_add() {
        let mut saved_events = SavedEvents::new();
        let when = Local::now().naive_local() + Duration::try_days(1).unwrap();
        saved_events.add(SavedEvent {
            name: "Event 1".to_string(),
            location: "Location 1".to_string(),
            latitude: 42.0,
            longitude: -73.0,
            when,
        });
        saved_events.add(SavedEvent {
            name: "Event 2".to_string(),
            location: "Location 2".to_string(),
            latitude: 42.0,
            longitude: -73.0,
            when: Local::now().naive_local() + Duration::try_days(2).unwrap(),
        });
        saved_events.add(SavedEvent {
            name: "Event 1".to_string(),
            location: "Location 1".to_string(),
            latitude: 42.0,
            longitude: -73.0,
            when,
        });

        assert_eq!(saved_events.events.len(), 2);
    }

    #[test]
    fn test_saved_events_cleanup() {
        let mut saved_events = SavedEvents::new();

        saved_events.add(SavedEvent {
            name: "Event 1".to_string(),
            location: "Location 1".to_string(),
            latitude: 42.0,
            longitude: -73.0,
            when: Local::now().naive_local() - Duration::try_days(1).unwrap(),
        });
        saved_events.add(SavedEvent {
            name: "Event 2".to_string(),
            location: "Location 2".to_string(),
            latitude: 42.0,
            longitude: -73.0,
            when: Local::now().naive_local() + Duration::try_days(2).unwrap(),
        });

        saved_events.cleanup();

        assert_eq!(saved_events.events.len(), 1);
        assert_eq!(saved_events.events[0].name, "Event 2");
    }

    #[test]
    fn test_old_file_is_renamed_on_load() {
        let (old_path, tmp) = create_old_temp_file();
        let new_path = tmp.path().join(FILE_NAME);
        File::create(&old_path)
            .unwrap()
            .write_all(b"{\"events\":[]}")
            .unwrap();

        assert!(SavedEvents::from_file_path(&new_path).is_ok());
        assert!(new_path.exists());
        assert!(!old_path.exists());
        tmp.close().unwrap();
    }

    #[test]
    fn test_old_file_is_renamed_on_save() {
        let (old_path, tmp) = create_old_temp_file();
        let new_path = tmp.path().join(FILE_NAME);
        File::create(&old_path).unwrap();

        assert!(SavedEvents::new().to_file_path(&new_path).is_ok());
        assert!(new_path.exists());
        assert!(!old_path.exists());
        tmp.close().unwrap();
    }
}
