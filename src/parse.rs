use ariadne::{Cache, Color, Label, Report, ReportKind, Source};
use chrono::{Duration, NaiveTime};
use lazy_static::lazy_static;
use range_check::{Check, OutOfRangeError};
use regex::{Match, Regex};
use std::error::Error;
use std::fmt;
use std::num::IntErrorKind;
use std::ops::Range;

#[derive(Debug)]
pub struct StringSection {
    text: String,
    start: usize,
    end: usize,
}

impl StringSection {
    fn new(s: &str, range: Range<usize>) -> Self {
        StringSection {
            text: s.into(),
            start: range.start,
            end: range.end,
        }
    }

    fn range(&self) -> Range<usize> {
        self.start..self.end
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Field {
    Overall,
    Hour,
    Minute,
    Second,
    Pm,
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Overall => "overall",
                Self::Hour => "hour",
                Self::Minute => "minute",
                Self::Second => "second",
                Self::Pm => "am/pm",
            }
        )
    }
}

#[derive(Debug)]
pub enum TimeParseError {
    IncompleteField(Field, StringSection),
    OutOfRange(Field, StringSection, OutOfRangeError<u32>),
    InvalidFormat(Field, StringSection),
    Overconstrained {
        hour: StringSection,
        pm: StringSection,
    },
}

impl TimeParseError {
    fn text(&self) -> &str {
        match self {
            Self::IncompleteField(_, section) => &section.text,
            Self::OutOfRange(_, section, _) => &section.text,
            Self::InvalidFormat(_, section) => &section.text,
            Self::Overconstrained { hour, pm: _ } => &hour.text,
        }
    }

    fn index(&self) -> usize {
        match self {
            Self::IncompleteField(_, section) => section.start,
            Self::OutOfRange(_, section, _) => section.start,
            Self::InvalidFormat(_, section) => section.start,
            Self::Overconstrained { hour: _, pm } => pm.start,
        }
    }
}

struct StringSource(Source, String);

impl Cache<()> for StringSource {
    fn fetch(&mut self, _id: &'_ ()) -> Result<&Source, Box<dyn fmt::Debug + '_>> {
        Ok(&self.0)
    }

    fn display<'a>(&self, _id: &'a ()) -> Option<Box<dyn fmt::Display + 'a>> {
        Some(Box::new(self.1.clone()))
    }
}

impl fmt::Display for TimeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = Vec::new();
        let green = Color::Green.style().bold();
        let builder = Report::build(ReportKind::Error, (), self.index());
        match self {
            Self::IncompleteField(field, section) => {
                if matches!(field, Field::Overall) {
                    builder.with_message("Expected time, instead got empty string")
                } else {
                    builder
                        .with_message(format!("{} field is incomplete", field))
                        .with_label(
                            Label::new(section.range())
                                .with_message(format!("{} is missing", green.paint(field)))
                                .with_color(Color::Yellow),
                        )
                }
            }
            Self::OutOfRange(field, section, err) => builder
                .with_message(format!("{} field is out of range", field))
                .with_label(Label::new(section.range()).with_message(format!(
                    "this is not in the proper range ({}) for {}",
                    Color::White.style().bold().paint(&err.allowed_range),
                    green.paint(field)
                ))),
            Self::InvalidFormat(field, section) => {
                if matches!(field, Field::Overall) {
                    builder
                        .with_message("Invalid format")
                        .with_note("expected a time")
                        .with_label(
                            Label::new(section.range())
                                .with_message("could not make sense of this"),
                        )
                } else {
                    builder.with_message("Invalid format").with_label(
                        Label::new(section.range())
                            .with_message(format!("{} has invalid format", green.paint(field))),
                    )
                }
            }
            Self::Overconstrained { hour, pm } => {
                builder
                    .with_message("Time is overconstrained")
                    .with_label(Label::new(hour.range()).with_message("this is already 24-hour"))
                    .with_label(
                        Label::new(pm.range()).with_message("so this is too much information"),
                    )
                //write!(f, "`{}` is already 24-hour, adding `{}` is too much information", hour.as_str(), pm.as_str())                }
            }
        }
        .finish()
        .write(
            StringSource(Source::from(self.text().to_string()), "time".to_string()),
            &mut buf,
        )
        .unwrap();
        write!(f, "{}", String::from_utf8(buf).unwrap())
    }
}

impl Error for TimeParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::OutOfRange(_, _, err) => Some(err),
            _ => None,
        }
    }
}

/**
 * We can parse
 * HH
 * HHp
 * HH p
 * HH:MM
 * HH:MMp
 * HH:MM p
 * HH:MM:SS
 * HH:MM:SSp
 * HH:MM:SS p
 *
 * If the minutes or seconds are ommitted, they are assumed to be zero
 * If the am/pm is ommitted, it is interpeted as 24-hour time
 *
 * All numeric fields can be zero-padded, or not
 */
pub fn opinionated_time_parsing(s: &str) -> Result<NaiveTime, TimeParseError> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"(?xi)
            (?P<hour>-?\d+)         # the hour (required)
            (?::(?P<minute>-?\d*))? # the minute (optional)
            (?::(?P<second>-?\d*))? # the second (optional)
            (?:\s?(?P<pm>.*(?:am|pm)))? # am or pm (interpreted as 24-hour if ommitted)
        "
        )
        .unwrap();
    }

    if s.is_empty() {
        return Err(TimeParseError::IncompleteField(
            Field::Overall,
            StringSection::new(s, 0..s.len()),
        ));
    }

    let cap = RE.captures(s).ok_or_else(|| {
        TimeParseError::InvalidFormat(Field::Overall, StringSection::new(s, 0..s.len()))
    })?;

    // hour could be 24-hour but there's still an am/pm

    let hour = match cap.name("hour") {
        None => {
            return Err(TimeParseError::IncompleteField(
                Field::Hour,
                StringSection::new(s, 0..s.len()),
            ))
        }
        Some(capture) => parse_field(s, Field::Hour, 0..24, capture)?,
    };
    let minute = match cap.name("minute") {
        None => 0,
        Some(capture) => parse_field(s, Field::Minute, 0..60, capture)?,
    };
    let second = match cap.name("second") {
        None => 0,
        Some(capture) => parse_field(s, Field::Second, 0..60, capture)?,
    };

    let pm = match cap.name("pm") {
        None => None,
        Some(pm) => Some(match pm.as_str().to_ascii_lowercase().as_str() {
            "am" => Duration::zero(),
            "pm" => {
                if hour == 12 {
                    // 12 pm is already correct
                    // we don't need to do anything to convert to 24-hour time
                    Duration::zero()
                } else {
                    Duration::hours(12)
                }
            }
            _ => {
                return Err(TimeParseError::InvalidFormat(
                    Field::Pm,
                    StringSection::new(s, pm.range()),
                ))
            }
        }),
    };

    if hour > 12 && pm.is_some() {
        return Err(TimeParseError::Overconstrained {
            hour: StringSection::new(s, cap.name("hour").unwrap().range()),
            pm: StringSection::new(s, cap.name("pm").unwrap().range()),
        });
    }

    let mut time = NaiveTime::from_hms_opt(hour, minute, second).unwrap();

    if let Some(diff) = pm {
        time += diff;
    }

    Ok(time)
}

fn parse_field(
    s: &str,
    field: Field,
    range: Range<u32>,
    capture: Match,
) -> Result<u32, TimeParseError> {
    capture
        .as_str()
        .parse::<u32>()
        .map_err(|err| match err.kind() {
            IntErrorKind::Empty => {
                TimeParseError::IncompleteField(field, StringSection::new(s, capture.range()))
            }
            IntErrorKind::InvalidDigit => {
                TimeParseError::InvalidFormat(field, StringSection::new(s, capture.range()))
            }
            _ => TimeParseError::InvalidFormat(field, StringSection::new(s, capture.range())),
        })?
        .check_range(range)
        .map_err(|err| {
            TimeParseError::OutOfRange(field, StringSection::new(s, capture.range()), err)
        })
}

#[test]
fn time_parsing_happy_paths() {
    assert_eq!(
        opinionated_time_parsing("6").unwrap(),
        NaiveTime::from_hms(6, 0, 0)
    );
    assert_eq!(
        opinionated_time_parsing("6am").unwrap(),
        NaiveTime::from_hms(6, 0, 0)
    );
    assert_eq!(
        opinionated_time_parsing("6pm").unwrap(),
        NaiveTime::from_hms(18, 0, 0)
    );
    assert_eq!(
        opinionated_time_parsing("6 pm").unwrap(),
        NaiveTime::from_hms(18, 0, 0)
    );
    assert_eq!(
        opinionated_time_parsing("6:30").unwrap(),
        NaiveTime::from_hms(6, 30, 0)
    );
    assert_eq!(
        opinionated_time_parsing("6:30pm").unwrap(),
        NaiveTime::from_hms(18, 30, 0)
    );
    assert_eq!(
        opinionated_time_parsing("6:30 pm").unwrap(),
        NaiveTime::from_hms(18, 30, 0)
    );
    assert_eq!(
        opinionated_time_parsing("6:30:15").unwrap(),
        NaiveTime::from_hms(6, 30, 15)
    );
    assert_eq!(
        opinionated_time_parsing("6:30:15 pm").unwrap(),
        NaiveTime::from_hms(18, 30, 15)
    );
}

#[test]
fn time_parsing_edge_cases() {
    println!(
        "{}",
        opinionated_time_parsing("").expect_err("test string is empty")
    );
    println!(
        "{}",
        opinionated_time_parsing("18:30 pm").expect_err("test string is overconstrained")
    );
    println!(
        "{}",
        opinionated_time_parsing("6:306").expect_err("minutes are out of bounds")
    );
    println!(
        "{}",
        opinionated_time_parsing("6::6").expect_err("minutes are missing")
    );
    println!(
        "{}",
        opinionated_time_parsing("6:0:").expect_err("seconds are missing")
    );
    println!(
        "{}",
        opinionated_time_parsing("20:-30").expect_err("negative numbers are not allowed")
    );
    println!(
        "{}",
        opinionated_time_parsing("63").expect_err("63 hours is greater than maximum of 23 hours")
    );
    println!(
        "{}",
        opinionated_time_parsing("6555")
            .expect_err("6555 hours is greater than maximum of 23 hours")
    );
    println!(
        "{}",
        opinionated_time_parsing("hello").expect_err("`hello` is not a time")
    );
}
