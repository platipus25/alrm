use structopt::StructOpt;

/// Search for a pattern in a file and display the lines that contain it.
/*#[derive(StructOpt)]
struct Cli {
    /// The pattern to look for
    pattern: String,
    /// The path to the file to read
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}*/

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let args = Cli::from_args();
    let string = "01:05";
    let date = parse_time(string)?;
    println!("Hello, world! {:?}", date);
    Ok(())
}

fn parse_time(s: &str) -> Result<time::Time, time::ParseError> {
    match time::Time::parse(s, "%-H:%-M") {
        Ok(date) => { Ok(date) },
        Err(err) => { Ok(time::Time::parse(s, "%-H")?) },
    }
}
