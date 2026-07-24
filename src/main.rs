use anyhow::Result;
use clap::{Parser, Subcommand};
use wed::persistence::{SavedEvent, SavedEvents};
use wed::units::Units;
use wed::weather::Notification;
use wed::{Event, WedClient, DEFAULT_AFTER, DEFAULT_BEFORE};

/// Weather on the Event Day
#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct Args {
    /// Outputs the weather forecast in JSON format (instead of the human-readable version)
    #[arg(short, long, conflicts_with = "chart")]
    json: bool,

    /// Show a chart of the hourly weather forecast
    #[arg(short, long)]
    chart: bool,

    /// Output more information about the internal state of the application
    #[arg(short, long)]
    verbose: bool,

    /// Units to use for the weather forecast
    #[arg(short, long)]
    units: Option<Units>,

    /// Number of retries for rate-limited requests
    #[arg(short, long, default_value_t = 7)]
    retries: u64,

    /// Force a specific terminal width for the chart (overrides detection)
    #[arg(short = 'w', long, global = true)]
    width: Option<usize>,

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
        /// The name of the event
        name: String,
        /// The location of the event (e.g. "Ottawa, ON, Canada")
        location: String,
        /// The date and time of the event (e.g. "2022-07-09 09:00")
        when: String,
        /// Hours before the event to include in the forecast window
        #[arg(short, long, default_value_t = DEFAULT_BEFORE)]
        before: i64,
        /// Hours after the event to include in the forecast window
        #[arg(short, long, default_value_t = DEFAULT_AFTER)]
        after: i64,
    },

    /// Show the forecast for a given location, date and time
    Forecast {
        /// The location (e.g. "Ottawa, ON, Canada")
        location: String,
        /// The date and time (e.g. "2022-07-09 09:00")
        when: String,
        /// Hours before the event to include in the forecast window
        #[arg(short, long, default_value_t = DEFAULT_BEFORE)]
        before: i64,
        /// Hours after the event to include in the forecast window
        #[arg(short, long, default_value_t = DEFAULT_AFTER)]
        after: i64,
    },

    /// Display a desktop notification (defaults to JSON output on unsupported systems or on failure)
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

async fn forecast_for_saved_events(
    client: &WedClient,
    units: &Units,
    verbose: bool,
    json: bool,
    chart: bool,
    width: Option<usize>,
) -> Result<()> {
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
        let client = client.clone();
        let window = if chart { Some(event.window()) } else { None };
        tasks.push(tokio::spawn(async move {
            event
                .weather(&client, &unit, window)
                .await?
                .as_string(json, chart, width)
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

async fn forecast_for(
    client: &WedClient,
    event: &Event,
    units: &Units,
    json: bool,
    verbose: bool,
    chart: bool,
    width: Option<usize>,
) -> Result<()> {
    if event.has_weather_forecast(verbose) {
        let window = if chart { Some(event.window()) } else { None };
        println!(
            "{}",
            event
                .weather(client, units, window)
                .await?
                .as_string(json, chart, width)?
        );
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

async fn load_notification(
    client: &WedClient,
    units: &Units,
    verbose: bool,
) -> Option<Notification> {
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
    events[0]
        .weather(client, units, None)
        .await
        .ok()?
        .as_notification()
        .ok()
}

async fn json_notification(client: &WedClient, units: &Units, verbose: bool) -> Result<()> {
    if let Some(notification) = load_notification(client, units, verbose).await {
        println!("{}", serde_json::to_string(&notification)?);
    }
    Ok(())
}

async fn send_notification(client: &WedClient, units: &Units) -> Result<()> {
    if let Some(notification) = load_notification(client, units, false).await {
        notify_rust::Notification::new()
            .summary(&notification.title)
            .body(&format!("{}\n{}", notification.subtitle, notification.body))
            .show()?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let units = args.units.unwrap_or(Units::Metric);

    let client = WedClient::new(args.retries)?;

    match &args.command {
        None => {
            forecast_for_saved_events(
                &client,
                &units,
                args.verbose,
                args.json,
                args.chart,
                args.width,
            )
            .await
        }
        Some(Commands::List {}) => list_saved_events(args.verbose).await,
        Some(Commands::Delete { name }) => delete_event(name, args.verbose).await,
        Some(Commands::Forecast {
            location,
            when,
            before,
            after,
        }) => {
            let event = Event::new(
                &client,
                None,
                when.clone(),
                Some(*before),
                Some(*after),
                location.clone(),
            )
            .await?;
            forecast_for(
                &client,
                &event,
                &units,
                args.json,
                args.verbose,
                args.chart,
                args.width,
            )
            .await
        }
        Some(Commands::Save {
            name,
            location,
            when,
            before,
            after,
        }) => {
            let event = Event::new(
                &client,
                Some(name.clone()),
                when.clone(),
                Some(*before),
                Some(*after),
                location.clone(),
            )
            .await?;
            forecast_for(
                &client,
                &event,
                &units,
                args.json,
                args.verbose,
                args.chart,
                args.width,
            )
            .await?;
            save_event(&event).await
        }

        Some(Commands::Notify {}) => {
            if !args.json {
                if let Err(e) = send_notification(&client, &units).await {
                    eprintln!("Error displaying notification: {e}");
                    json_notification(&client, &units, args.verbose).await?;
                }
            } else {
                json_notification(&client, &units, args.verbose).await?;
            }
            Ok(())
        }
    }
}
