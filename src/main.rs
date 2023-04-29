use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use clap::Parser;
use wed::Forecast;

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
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let forecast = Forecast::new(args.when, args.location).await?;

    // Debug
    println!("{forecast}");
    if let Some(name) = args.name {
        println!("Event\t{name}");
    }
    if forecast.is_in_the_past() {
        println!("Warning: the event date is in the past");
    }
    forecast.api();

    Ok(())
}
