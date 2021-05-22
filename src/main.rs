use structopt::StructOpt;
use chrono::prelude::*;
use hhmmss::Hhmmss;

/// Take a time and print the time remaining until that time. If `time` has already passed today,
/// then count to `time` tomorrow.
#[derive(StructOpt)]
struct Cli {
    /// The pattern to look for
    #[structopt(parse(try_from_str = parse_time))]
    time: NaiveTime,
}

fn main() {
    let args = Cli::from_args();

    let mut date = Local::today();

    if args.time < Local::now().time() {
        date = date.succ();
    }

    let date = date.and_time(args.time).unwrap();
    let time_left = date - Local::now();

    let relative_day = if date.date() == Local::today() { "today" } else { "tomorrow" };
    println!("{} until {} {}", time_left.hhmmss(), args.time.format("%r"), relative_day);
}

/// Parse a time from a string.
/// First look for 12 hour format (%-I:%-M%p)
/// Then for 24-hour format  ("%-H:%-M")
/// Then for just an hour (%-H)
fn parse_time(s: &str) -> Result<NaiveTime, chrono::ParseError> {
    NaiveTime::parse_from_str(s, "%-I:%-M%p")
        .or_else(|_e| NaiveTime::parse_from_str(s, "%-H:%-M"))
        .or_else(|e| match s.parse::<u32>() {
            Ok(hour) => Ok(NaiveTime::from_hms(hour, 0, 0)),
            Err(_err) => Err(e)
        })
}
