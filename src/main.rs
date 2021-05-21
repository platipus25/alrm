use structopt::StructOpt;
use time::Time;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(StructOpt)]
struct Cli {
    /// The pattern to look for
    #[structopt(parse(try_from_str = parse_time))]
    time: Time,
    ///// The path to the file to read
    //#[structopt(parse(from_os_str))]
    //path: std::path::PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::from_args();
    println!("Hello, world! {}", args.time.format("%r"));

    let now = time::OffsetDateTime::try_now_local();
    println!("{:?}", now);

    /*let new_time = now.with_time(args.time);


    println!("{:?}", new_time);*/
    Ok(())
}

fn parse_time(s: &str) -> Result<Time, time::ParseError> {
    Time::parse(s, "%-I:%-M%p")
        .or_else(|_e| Time::parse(s, "%-H:%-M"))
        .or_else(|_e| Time::parse(s, "%-H"))
}
