use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use clap::Parser;
use wed::Event;

const DATE_INPUT_FORMAT: &str = "%Y-%m-%d %H:%M";

/// Weather on the Event Day
#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    #[arg(
        short,
        long,
        value_parser = |value: &str| ->Result<NaiveDateTime> {
            NaiveDateTime::parse_from_str(value, DATE_INPUT_FORMAT).with_context(|| {
                format!("Failed to parse date, it should be in the format {DATE_INPUT_FORMAT}: {value}")
            })
        },
        help= format!("Event date in the {DATE_INPUT_FORMAT} format"),
    )]
    when: NaiveDateTime,

    /// Event location (city and country; province or state is optional)
    #[arg(short, long)]
    location: String,

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
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let event = Event::new(args.when, args.location).await?;
    if !event.has_weather_forcast(args.verbose) {
        return Ok(());
    }

    println!("{}", event.weather(args.json).await?);
    Ok(())
}
