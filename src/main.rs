#![deny(missing_docs)]
//! A quick countdown timer for your terminal.
//!
//! Alarms and timers are useful, but I found myself wanting to know how long I have left until an appointment or how long it is until lunch. Simply give `alrm` a time of day and it will tell you how long you have until then.
//!
//! Example
//! ```bash
//! alrm 9       # prints the time until 9:00 am
//! alrm 9:30pm  # prints the time until 9:30 pm
//! alrm 9:00 -u # counts down to 9:00 am and then exits
//! ```

use chrono::prelude::*;
use clap::Parser;
use console::{Style, Term};
use hhmmss::Hhmmss;
use std::thread;

/// A quick countdown timer
#[derive(Parser, Debug)]
#[clap(version)]
struct Cli {
    /// time to count down to
    #[clap(parse(try_from_str = parse_time), long_help = "Count down to TIME. If TIME has already passed today, then count down the TIME tomorrow.")]
    time: NaiveTime,

    /// update console countdown
    #[clap(
        long,
        short,
        long_help = "Update the countdown until once the time has passed and then exit"
    )]
    update: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let term = Term::stdout();

    let mut date = Local::today();

    if args.time < Local::now().time() {
        date = date.succ();
    }

    let date = date.and_time(args.time).unwrap();

    let yellow = Style::new().bright().yellow();
    loop {
        let time_left = date - Local::now();
        let relative_day = if date.date() == Local::today() {
            "today"
        } else {
            "tomorrow"
        };
        let output = format!(
            "{} until {} {}",
            yellow.apply_to(time_left.hhmmss()),
            args.time.format("%-I:%M%P"),
            relative_day
        );
        term.write_line(&output)?;

        if !args.update || date < Local::now() {
            break;
        }

        thread::sleep(std::time::Duration::from_millis(1000));
        term.clear_last_lines(1)?;
    }
    Ok(())
}

/// Parse a time from a string.
///
/// First look for 12 hour format (%I:%MM%p)
/// Then for 24-hour format  (%H:%MM)
/// Then for just an hour (%H)
fn parse_time(s: &str) -> Result<NaiveTime, chrono::ParseError> {
    NaiveTime::parse_from_str(s, "%-I:%M:%S%p")
        .or_else(|_e| NaiveTime::parse_from_str(s, "%-I:%M%p"))
        .or_else(|_e| NaiveTime::parse_from_str(s, "%-H:%M"))
        .or_else(|e| match s.parse::<u32>() {
            Ok(hour) => match NaiveTime::from_hms_opt(hour, 0, 0) {
                Some(time) => Ok(time),
                None => Err(e),
            },
            Err(_err) => Err(e),
        })
}
