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

mod parse;

use crate::parse::opinionated_time_parsing;
use chrono::Local;
use clap::Parser;
use console::{Style, Term};
use hhmmss::Hhmmss;
use std::thread;

/// A quick countdown timer
#[derive(Parser, Debug)]
#[clap(version)]
struct Cli {
    /// update console countdown
    #[clap(
        long,
        short,
        long_help = "Update the countdown until once the time has passed and then exit"
    )]
    update: bool,

    /// time to count down to
    #[clap(
        long_help = "Count down to TIME. If TIME has already passed today, then count down the TIME tomorrow.",
        use_value_delimiter = false,
        multiple_values = true
    )]
    time: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let term = Term::stdout();

    let time_str = args.time.join(" ");
    let time = match opinionated_time_parsing(&time_str) {
        Ok(time) => time,
        Err(err) => {
            eprint!("{}", err);
            std::process::exit(1);
        }
    };

    let mut date = Local::today();

    if time < Local::now().time() {
        date = date.succ();
    }

    let date = date.and_time(time).unwrap();

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
            time.format("%-I:%M%P"),
            relative_day
        );
        term.write_line(&output)?;

        if !args.update {
            break;
        }

        thread::sleep(std::time::Duration::from_millis(1000));

        if date < Local::now() {
            break;
        }

        term.clear_last_lines(1)?;
    }
    Ok(())
}
