use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufRead, LineWriter};
use std::path::PathBuf;

use regex::Regex;

enum Token {
    BeginvEvent,
    DESCRIPTION,
    LOCATION,
    SUMMARY,
    NONE
}

#[derive(Default,Debug)]
struct Event {
    start: DateTime,
    end: DateTime,
    title: String,
    description: String,
    organizer: Organizer,
    location: String
}

#[derive(Default, Debug)]
struct Organizer {
    calendar: String,
    mail_to: String
}

#[derive(Debug)]
enum DateTimeFormat {
    Local,
    UTC,
    TimeZone(String),
}
#[derive(Debug)]

struct DateTime {
    format: DateTimeFormat,
    year: u32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8
}

impl Default for DateTime {
    fn default() -> DateTime {
        DateTime {
            format: DateTimeFormat::UTC,
            year: 1970,
            month: 01,
            day: 01,
            hour: 00,
            minute: 00,
            second: 00
        }
    }
}

fn take(token: &str, s: &str) -> Option<String> {
    return if s.starts_with(token) {
        Some(String::from(&s[token.len()..]))
    } else {
        None
    }
}

fn convert(file: &mut File) -> Vec<Event> {
    let mut events: Vec<Event> = Vec::new();
    let reader = BufReader::new(file);

    events.push(Event::default());
    let mut event = events.last_mut().unwrap();
    let mut typ = Token::BeginvEvent;
    
    for line_op in reader.lines() {
        if let Ok(line) = line_op {
            if line.starts_with(" ") {
                match typ {
                    Token::DESCRIPTION => event.description.push_str(&line[1..]),
                    Token::LOCATION => event.location.push_str(&line[1..]),
                    Token::SUMMARY => event.title.push_str(&line[1..]),
                    _ => ()
                }
            }
            else if let Some(_) = take("END:VEVENT", &line) {
                events.push(Event::default());
                event = events.last_mut().unwrap();
            } else if let Some(time) = take("DTSTART", &line) {
                event.start = parse_date_time(&time[1..]);
            } else if let Some(time) = take("DTEND", &line) {
                event.end = parse_date_time(&time[1..]);
            } else if let Some(desc) = take("DESCRIPTION:", &line) {
                event.description = desc;
                typ = Token::DESCRIPTION;
            } else if let Some(loc) = take("LOCATION:", &line) {
                event.location = loc;
                typ = Token::LOCATION;
            } else if let Some(sum) = take("SUMMARY:", &line) {
                event.title = sum;
                typ = Token::SUMMARY;
            } else if let Some(org) = take("ORGANIZER;", &line) {
                event.organizer = parse_organizer(&org);
            } else {
                typ = Token::NONE;
            }
        }
    }

    events.pop();

    events
}


fn parse_organizer(s: &str) -> Organizer {
    let re = Regex::new(r"^.*CN=(?P<cal>.+):mailto:(?P<mail>.+)$").unwrap();
    match re.captures(s) {
        Some(cap) => Organizer {
            calendar: String::from(&cap["cal"]),
            mail_to: String::from(&cap["mail"])
        },
        None => Organizer::default()
    }
}


fn parse_date_time(s: &str) -> DateTime {
    let local = Regex::new(r"^(\d{4})(\d{2})(\d{2})T(\d{2})(\d{2})(\d{2})$").unwrap();
    let utc = Regex::new(r"^(\d{4})(\d{2})(\d{2})T(\d{2})(\d{2})(\d{2})Z$").unwrap();
    let zone = Regex::new(r"^TZID=(.+):(\d{4})(\d{2})(\d{2})T(\d{2})(\d{2})(\d{2})$").unwrap();

    if let Some(cap) = local.captures(s) {
        DateTime {
            format: DateTimeFormat::Local,
            year: cap[1].parse().unwrap(),
            month: cap[2].parse().unwrap(),
            day: cap[3].parse().unwrap(),
            hour: cap[4].parse().unwrap(),
            minute: cap[5].parse().unwrap(),
            second: cap[6].parse().unwrap()
        }
    } else if let Some(cap) = utc.captures(s) {
        DateTime {
            format: DateTimeFormat::UTC,
            year: cap[1].parse().unwrap(),
            month: cap[2].parse().unwrap(),
            day: cap[3].parse().unwrap(),
            hour: cap[4].parse().unwrap(),
            minute: cap[5].parse().unwrap(),
            second: cap[6].parse().unwrap()
        }
    } else if let Some(cap) = zone.captures(s) {
        DateTime {
            format: DateTimeFormat::TimeZone(String::from(&cap[1])),
            year: cap[2].parse().unwrap(),
            month: cap[3].parse().unwrap(),
            day: cap[4].parse().unwrap(),
            hour: cap[5].parse().unwrap(),
            minute: cap[6].parse().unwrap(),
            second: cap[7].parse().unwrap()
        }
    } else {
        eprintln!("Failed to parse DateTime: {}", s);
        DateTime::default()
    }
}

fn write_org(events: Vec<Event>, writer: &mut dyn Write) -> Result<(), std::io::Error> {
    writer.write_all(b"* Google Calendar\n")?;

    for event in events {
        let time = event.start;
        write!(writer, "** {}\nSCHEDULED: <{}-{}-{}>\n", event.title, time.year, time.month, time.day)?;
    }

    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let mut args = std::env::args();

    if args.len() == 0 {
        eprintln!("Please provide the path to the .ics file.");
        std::process::exit(1);    
    }

    args.next();
    let input_file = args.next().unwrap();
    let file_path_buf = PathBuf::from(&input_file);

    if !file_path_buf.exists() {
        eprintln!("Error: File {:?} does not exist.", file_path_buf);
        std::process::exit(1);
    } else {
        println!("File exists at: {:?}", file_path_buf);
    }

    let mut file: File = File::open(file_path_buf).unwrap();

    let mut output_file = File::create(PathBuf::from(&input_file.replace("ics", "org"))).unwrap();

    let events = convert(&mut file);
    write_org(events, &mut output_file)?;

    Ok(())
}
