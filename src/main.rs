use std::process::Command;

use anyhow::Result;
use clap::{Parser, Subcommand};
use wed::persistence::{SavedEvent, SavedEvents};
use wed::units::Units;
use wed::weather::Notification;
use wed::Event;

const MACOS_NOTIFICATION_COMMAND: &str = "on run argv
  display notification (item 3 of argv) with title (item 1 of argv) subtitle (item 2 of argv)
end run";

/// Weather on the Event Day
#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct Args {
    /// Outputs the weather forecast in JSON format (instead of the human-readable version)
    #[arg(short, long)]
    json: bool,

    /// Output more information about the internal state of the application
    #[arg(short, long)]
    verbose: bool,

    /// Units to use for the weather forecast
    #[arg(short, long)]
    units: Option<Units>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List saved events
    List {},

    /// Delete a saved event
    Delete { name: String },

    /// Save an event
    Save {
        name: String,
        location: String,
        when: String,
    },

    /// Show the forecast for a given location, date and time
    Forecast { location: String, when: String },

    /// Display a notification on macOS (defaults to JSON output on other OS)
    Notify {},
}

async fn load_saved_events(verbose: bool) -> Result<SavedEvents> {
    let saved = match SavedEvents::from_file() {
        Ok(events) => events,
        Err(_) => SavedEvents::new(),
    };
    if saved.events.is_empty() && verbose {
        println!("No events saved.");
    }
    Ok(saved)
}

async fn list_saved_events(verbose: bool) -> Result<()> {
    for event in load_saved_events(verbose).await?.events {
        println!(
            "{} {}, {}",
            event.when.format(wed::DATE_INPUT_FORMAT),
            event.name,
            event.location
        );
    }
    Ok(())
}

async fn forecast_for_saved_events(units: &Units, verbose: bool, json: bool) -> Result<()> {
    let saved = load_saved_events(verbose)
        .await?
        .events
        .into_iter()
        .map(|data| data.to_event())
        .filter(|event| event.has_weather_forecast(verbose));

    let mut output: Vec<String> = Vec::new();
    let mut tasks = vec![];

    for event in saved {
        let unit = units.clone();
        tasks.push(tokio::spawn(async move {
            event.weather(&unit).await?.as_string(json)
        }));
    }

    for task in tasks {
        let result: Result<String> = task.await?;
        output.push(result?);
    }

    if !output.is_empty() {
        if json {
            println!("[{}]", output.join(","));
        } else {
            println!("{}", output.join("\n\n"));
        }
    }
    Ok(())
}

async fn forecast_for(event: &Event, units: &Units, json: bool, verbose: bool) -> Result<()> {
    if event.has_weather_forecast(verbose) {
        println!("{}", event.weather(units).await?.as_string(json)?);
    }
    Ok(())
}

async fn save_event(event: &Event) -> Result<()> {
    let mut events = match SavedEvents::from_file() {
        Ok(events) => events,
        Err(_) => SavedEvents::new(),
    };
    events.add(SavedEvent::from_event(event)?);
    events.to_file()
}

async fn delete_event(name: &str, verbose: bool) -> Result<()> {
    let mut saved = load_saved_events(verbose).await?;
    saved.events.retain(|event| event.name != name);
    saved.to_file()
}

async fn load_notification(units: &Units, verbose: bool) -> Option<Notification> {
    let events = load_saved_events(verbose)
        .await
        .ok()?
        .events
        .into_iter()
        .map(|data| data.to_event())
        .filter(|event| event.has_weather_forecast(verbose))
        .collect::<Vec<Event>>();
    if events.is_empty() {
        return None;
    }
    events[0].weather(units).await.ok()?.as_notification().ok()
}

async fn json_notification(units: &Units, verbose: bool) -> Result<()> {
    if let Some(notification) = load_notification(units, verbose).await {
        println!("{}", serde_json::to_string(&notification)?);
    }
    Ok(())
}

async fn macos_notification(units: &Units) -> Result<()> {
    if let Some(notification) = load_notification(units, false).await {
        Command::new("osascript")
            .arg("-e")
            .arg(MACOS_NOTIFICATION_COMMAND)
            .arg(notification.title)
            .arg(notification.subtitle)
            .arg(notification.body)
            .output()?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let units = args.units.unwrap_or(Units::Metric);
    match &args.command {
        None => forecast_for_saved_events(&units, args.verbose, args.json).await,
        Some(Commands::List {}) => list_saved_events(args.verbose).await,
        Some(Commands::Delete { name }) => delete_event(name, args.verbose).await,
        Some(Commands::Forecast { location, when }) => {
            let event = Event::new(None, when.clone(), location.clone()).await?;
            forecast_for(&event, &units, args.json, args.verbose).await
        }
        Some(Commands::Save {
            name,
            location,
            when,
        }) => {
            let event = Event::new(Some(name.clone()), when.clone(), location.clone()).await?;
            forecast_for(&event, &units, args.json, args.verbose).await?;
            save_event(&event).await
        }

        Some(Commands::Notify {}) => {
            if cfg!(target_os = "macos") && !args.json {
                macos_notification(&units).await
            } else {
                json_notification(&units, args.verbose).await
            }
        }
    }
}
