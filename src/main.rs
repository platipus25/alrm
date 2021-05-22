use structopt::StructOpt;
use chrono::prelude::*;
use hhmmss::Hhmmss;
use console::Term;
use std::thread;

/// Take a time and print the time remaining until that time. If `time` has already passed today,
/// then count to `time` tomorrow.
#[derive(StructOpt)]
struct Cli {
    /// The time to count down to
    #[structopt(parse(try_from_str = parse_time))]
    time: NaiveTime,

    #[structopt(short = "u")]
    repeat: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let args = Cli::from_args();
    let term = Term::stdout();

    let mut date = Local::today();

    if args.time < Local::now().time() {
        date = date.succ();
    }

    let date = date.and_time(args.time).unwrap();
    let time_left = date - Local::now();

    let relative_day = if date.date() == Local::today() { "today" } else { "tomorrow" };
    if args.repeat {
        println!("Time left until {} {}", args.time.format("%r"), relative_day);
        while date > Local::now() {
            term.write_line(&(date - Local::now()).hhmmssxxx())?;
            thread::sleep(std::time::Duration::from_millis(10));
            term.clear_last_lines(1)?;
        }
    } else {
        println!("{} until {} {}", time_left.hhmmss(), args.time.format("%r"), relative_day);
    }
    Ok(())
}

/// Parse a time from a string.
/// First look for 12 hour format (%-I:%-M%p)
/// Then for 24-hour format  ("%-H:%-M")
/// Then for just an hour (%-H)
fn parse_time(s: &str) -> Result<NaiveTime, chrono::ParseError> {
    NaiveTime::parse_from_str(s, "%-I:%M%p")
        .or_else(|_e| NaiveTime::parse_from_str(s, "%-H:%M"))
        .or_else(|e| match s.parse::<u32>() {
            Ok(hour) => Ok(NaiveTime::from_hms(hour, 0, 0)),
            Err(_err) => Err(e)
        })
}
