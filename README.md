# Weather on the Event Day

**:warning: THIS IS A WORK IN PROGRESS :warning:**

`wed`'s idea is to automate something I do pretty often: once I register for a run or triathlon event, I frequently check the city's weather forecast for the race's date and time in the previous weeks and days.

## Use cases (in the API-driven development style)

Try `wed --help`.

### Query a date and time followed by a location

Run `wed` with two arguments:

```console
$ wed --when "2022-07-09" --location "Ottawa, ON"
🌤 26°C (feels like 27°C)  ☔ 40% chance of rain & 10% humidity 💨 4.2 km/h W
```

Or with a JSON output:

```console
$ wed --when "2022-07-09" --location "Ottawa, ON" --json
{
    "temperature": 26,
    "feels_like": 27,
    "chance_of_rain": 0.4,
    "humidity": 0.1,
    "wind_speed": 4.2,
    "wind_direction": "W",
    "description": "scattered clouds",
}
```

### Save an event

Run `wed` with three arguments:

```console
$ wed --name "National Capital Triathlon" --when "2022-07-09 09:00" --location "Ottawa, ON, Canada"
```

### Query saved events

Run `wed` with no arguments:

```console
$ wed
🗓 National Capital Triathlon (Jul 7, 09:00) 🌐 Ottawa, ON, Canada
🌤 26°C (feels like 27°C) ☔ 40% chance of rain & 10% humidity 💨 4.2 km/h W
```

Or with a JSON output:

```console
$ wed --json
[
    {
        "name": "National Capital Triathlon",
        "when": "2022-07-09 09:00:00",
        "location": "Ottawa, ON, Canada",
        "weather": {
            "temperature": 26,
            "feels_like": 27,
            "chance_of_rain": 0.4,
            "humidity": 0.1,
            "wind_speed": 4.2,
            "wind_direction": "W",
            "description": "scattered clouds",
        }
    }
]
```

## Data sources

* Convertion of location (city/country) to a latitude and longitude: [Nominatin](https://wiki.openstreetmap.org/wiki/Nominatim)
* Weather forecast:
   * If _t - 4 days_: [Hourly forecast 4 days](https://openweathermap.org/api/hourly-forecast)
   * If _t - 16 days_: [Daily forecast 16 days](https://openweathermap.org/forecast16)
   * It _t - 30 days_: [Climatic forecast 30 days](https://openweathermap.org/api/forecast30)

## Data management and persistence

* The app automaticaly deletes past events when the app is run
* The app igonres events that are more than 30 days ahead
* Data is persisted in simple file, like a `~/.wed`
