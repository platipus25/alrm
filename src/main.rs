use structopt::StructOpt;
use chrono::prelude::*;
use hhmmss::Hhmmss;
use console::{ Term, Style };
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

    let yellow = Style::new().bright().yellow();
    loop {
        let time_left = date - Local::now();
        let relative_day = if date.date() == Local::today() { "today" } else { "tomorrow" };
        let output = format!("{} until {} {}", yellow.apply_to(time_left.hhmmss()), args.time.format("%-I:%M%P"), relative_day);
        term.write_line(&output)?;

        if !args.repeat || date < Local::now() { break }

        thread::sleep(std::time::Duration::from_millis(1000));
        term.clear_last_lines(1)?;
    }
    Ok(())
}

/// Parse a time from a string.
/// First look for 12 hour format (%-I:%-M%p)
/// Then for 24-hour format  ("%-H:%-M")
/// Then for just an hour (%-H)
fn parse_time(s: &str) -> Result<NaiveTime, chrono::ParseError> {
    NaiveTime::parse_from_str(s, "%-I:%M:%S%p")
        .or_else(|_e| NaiveTime::parse_from_str(s, "%-I:%M%p"))
        .or_else(|_e| NaiveTime::parse_from_str(s, "%-H:%M"))
        .or_else(|e| match s.parse::<u32>() {
            Ok(hour) => match NaiveTime::from_hms_opt(hour, 0, 0) {
                Some(time) => Ok(time),
                None => Err(e)
            },
            Err(_err) => Err(e)
        })
}
