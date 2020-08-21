use std::fs::File;
use std::io::{BufReader, BufRead};
use std::path::PathBuf;

#[derive(Default,Debug)]
struct Event {
    time: String,
    title: String,
    description: String,
    organizer: String,
    location: String
}

fn convert(file: &mut File) -> Vec<Event> {
    let mut events: Vec<Event> = Vec::new();
    let mut reader = BufReader::new(file);

    events.push(Event::default());
    let mut event = events.last_mut().unwrap();


    for line_op in reader.lines() {
        // println!("Reading line: {}", line);
        if let Ok(line) = line_op {
            match line.as_str() {
                "END:VEVENT" => {
                    events.push(Event::default());
                    event = events.last_mut().unwrap();
                },
                line => {
                    if line.starts_with("DTSTAMP") {
                        event.time = line.to_string();
                    } else if line.starts_with("DESCRIPTION") {
                        event.description = line.to_string();
                    } else if line.starts_with("LOCATION") {
                        event.location = line.to_string();
                    } else if line.starts_with("SUMMARY") {
                        event.title = line.to_string();
                    } else if line.starts_with("ORGANIZER") {
                        event.organizer = line.to_string();
                    }
                }
            }
        }
    }

    events
}


fn main() {
    let mut args = std::env::args();

    if args.len() == 0 {
        eprintln!("Please provide the path to the .ics file.");
        std::process::exit(1);    
    }

    args.next();
    let file_path_buf = PathBuf::from(args.next().unwrap());

    if !file_path_buf.exists() {
        eprintln!("Error: File {:?} does not exist.", file_path_buf);
        std::process::exit(1);
    } else {
        println!("File exists at: {:?}", file_path_buf);
    }

    let mut file: File = File::open(file_path_buf).unwrap();

    let rs = convert(&mut file);

    println!("{:?}", rs);
}
