use anyhow::{anyhow, Context, Result};
use chrono::NaiveDateTime;
use clap::Parser;
use wed::forecast::Units;
use wed::persistence::{SavedEvent, SavedEvents};
use wed::Event;

const DATE_INPUT_FORMAT: &str = "%Y-%m-%d %H:%M";

fn date_parser(value: &str) -> Result<NaiveDateTime> {
    NaiveDateTime::parse_from_str(value, DATE_INPUT_FORMAT).with_context(|| {
        format!("Failed to parse date, it should be in the format {DATE_INPUT_FORMAT}: {value}")
    })
}

/// Weather on the Event Day
#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    #[arg( short, long, value_parser = date_parser , help= format!("Event date in the {DATE_INPUT_FORMAT} format"))]
    when: Option<NaiveDateTime>,

    /// Event location (city and country; province or state is optional)
    #[arg(short, long)]
    location: Option<String>,

    /// Event name
    #[arg(short, long)]
    name: Option<String>,

    /// Outputs the weather forcast for the event day in JSON format (instead
    /// of the human-readable version)
    #[arg(short, long)]
    json: bool,

    // Output more information about the internal state of the application
    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long)]
    units: Option<Units>,
}

async fn show_existing_events(units: Units, verbose: bool, json: bool) -> Result<()> {
    let saved = match SavedEvents::from_file() {
        Ok(events) => events,
        Err(_) => SavedEvents::new(),
    };
    if saved.events.is_empty() {
        return Err(anyhow!("No events saved."));
    }
    for data in saved.events {
        let event = data.to_event();
        if event.has_weather_forcast(verbose) {
            println!("{}", event.weather(&units, json).await?);
        }
    }
    Ok(())
}

async fn show_adhoc_event(event: &Event, units: &Units, verbose: bool, json: bool) -> Result<()> {
    if event.has_weather_forcast(verbose) {
        println!("{}", event.weather(units, json).await?);
    }
    Ok(())
}

async fn show_and_save_event(event: Event, units: &Units, verbose: bool, json: bool) -> Result<()> {
    show_adhoc_event(&event, units, verbose, json).await?;
    if event.name.is_some() {
        let mut events = match SavedEvents::from_file() {
            Ok(events) => events,
            Err(_) => SavedEvents::new(),
        };
        events.add(SavedEvent::from_event(&event)?);
        events.to_file()?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let units = args.units.unwrap_or(Units::Metric);
    if args.name.is_none() && args.when.is_none() && args.location.is_none() {
        return show_existing_events(units, args.verbose, args.json).await;
    }
    if args.when.is_none() || args.location.is_none() {
        return Err(anyhow!(
            "Event date and location are required when using an event name."
        ));
    }

    let event = Event::new(
        args.name.clone(),
        args.when.unwrap(),
        args.location.unwrap(),
    )
    .await?;
    match args.name {
        Some(_) => show_and_save_event(event, &units, args.verbose, args.json).await,
        None => show_adhoc_event(&event, &units, args.verbose, args.json).await,
    }?;
    Ok(())
}
