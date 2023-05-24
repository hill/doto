use chrono::{Datelike, Duration, NaiveDate};
use clap::Parser;
use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::Command,
};

fn get_doto_path() -> String {
    let doto_path = std::env::var("DOTO_PATH").unwrap_or(format!(
        "{}/.doto",
        std::env::var("HOME").expect("Could not get $HOME or $DOTO_PATH")
    ));
    // create doto directory if it doesn't exist
    if !std::path::Path::new(&doto_path).exists() {
        println!("Creating doto directory at {}", doto_path);
        std::fs::create_dir(doto_path.clone()).expect("Could not create doto directory");
    }
    doto_path
}

// NOTE: does not include ".md"
fn get_today_todo_file_path() -> String {
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let path = format!("{}/{}.md", get_doto_path(), date);
    path
}

#[allow(dead_code)]
fn get_today_filename() -> String {
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    return format!("{}", date);
}

fn line_is_todo(l: &str) -> bool {
    return l.trim().starts_with("- [ ]") || l.trim().starts_with("- []");
}

fn move_undone() {
    // move all undone tasks to today's todo file

    // get all todo files in the past
    let todo_files = std::fs::read_dir(get_doto_path())
        .expect("Could not read doto directory")
        .filter(|f| f.is_ok())
        .map(|f| f.expect("Unable to read file").path())
        .filter(|f| f.is_file())
        .filter(
            |f| match f.extension().expect("Could not get file extension") {
                ext if ext == "md" => true,
                _ => false,
            },
        )
        .filter(|f| {
            let file_date = match f.file_name().and_then(|name| name.to_str()) {
                Some(name) if name.ends_with(".md") => {
                    match chrono::NaiveDate::parse_from_str(&name[..name.len() - 3], "%Y-%m-%d") {
                        Ok(date) => Some(date),
                        Err(_) => None,
                    }
                }
                _ => None,
            };

            if let Some(file_date) = file_date {
                return file_date < chrono::Local::now().naive_local().date();
            }
            return false; // ignore files that don't have a date in their name
        })
        .collect::<Vec<_>>();

    // loop through all todo files and find lines starting with "- [ ]"
    let mut undone_task_count = 0;
    for file in todo_files {
        let file_content = std::fs::read_to_string(file.clone()).unwrap();
        let lines = file_content.lines().collect::<Vec<_>>();
        let undone_tasks = lines
            .iter()
            .filter(|l| line_is_todo(l)) // append undone tasks to today's todo file
            .map(|l| {
                let filename = file.file_name().unwrap().to_str().unwrap();
                let truncated_file_name = &filename[..filename.len() - 3]; // Remove last three characters

                format!("{} ({})", l, truncated_file_name)
            })
            .collect::<Vec<_>>();

        let updated_file_content = lines
            .iter()
            .map(|l| {
                if line_is_todo(l) {
                    let whitespace = l
                        .chars()
                        .take_while(|c| c.is_whitespace())
                        .collect::<String>();

                    // add date to line
                    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
                    format!("{}- [>] ({}) {}", whitespace, date, &l[6..])
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        std::fs::write(file.clone(), updated_file_content).unwrap();

        undone_task_count += undone_tasks.len();

        if undone_tasks.len() == 0 {
            continue;
        }

        // append undone tasks to today's todo file
        let today_todo_file = get_today_todo_file_path();
        let mut today_todo_content = std::fs::read_to_string(today_todo_file.clone()).unwrap();
        today_todo_content.push_str("\n");
        today_todo_content.push_str(&undone_tasks.join("\n"));
        std::fs::write(today_todo_file, today_todo_content).unwrap();
    }

    println!(
        "Moved {} undone tasks to today's todo file",
        undone_task_count
    );
}

fn get_or_make_file(filename: String) -> String {
    let doto_path = get_doto_path();
    let todo_file = format!("{}/{}.md", doto_path, filename);

    // create todo file if it doesn't exist
    if !std::path::Path::new(&todo_file).exists() {
        std::fs::write(&todo_file, format!("# {}", filename))
            .expect("Unable to write a new todo file");
    }

    return todo_file;
}

fn open_file(filename: String) {
    let todo_file = get_or_make_file(filename);

    // open today's todo file in user's default editor
    let editor = std::env::var("EDITOR").unwrap_or("vim".to_string());
    Command::new(editor)
        .arg(&todo_file)
        .status()
        .expect("Could not open file");
}

fn open_today() {
    let today_todo_file = get_today_filename();
    open_file(today_todo_file);
}

#[allow(unused_assignments)]
fn parse_day_string(date: String) -> Option<NaiveDate> {
    let today = chrono::Local::now().date_naive();
    let mut parsed_date: Option<NaiveDate> = None;
    if date.chars().all(|c| c.is_alphabetic()) {
        match date.to_lowercase().as_str() {
            "now" | "t" | "today" => parsed_date = Some(today),
            "prev" | "yes" | "yesterday" => parsed_date = Some(today.pred_opt().unwrap()),
            "next" | "tom" | "tomorrow" => parsed_date = Some(today.succ_opt().unwrap()),
            _ => {
                parsed_date = None;
            }
        }

        if parsed_date.is_some() {
            return parsed_date;
        }

        // parse as a wed/wednesday string
        let target_weekday = match date.to_lowercase().as_str() {
            "mon" | "monday" => chrono::Weekday::Mon,
            "tue" | "tuesday" => chrono::Weekday::Tue,
            "wed" | "wednesday" => chrono::Weekday::Wed,
            "thu" | "thursday" => chrono::Weekday::Thu,
            "fri" | "friday" => chrono::Weekday::Fri,
            "sat" | "saturday" => chrono::Weekday::Sat,
            "sun" | "sunday" => chrono::Weekday::Sun,
            _ => {
                eprintln!("Invalid weekday. Expects mon, tue, wed, thu, fri, sat or sun");
                return None;
            }
        };

        let mut target_date = today;
        let days_until_monday = (today.weekday().num_days_from_monday() + 7 - 1) % 7;
        target_date = target_date - Duration::days(days_until_monday as i64);
        let days_until_target = (target_weekday.num_days_from_monday() + 7 - 1) % 7;
        target_date = target_date + Duration::days(days_until_target as i64);

        parsed_date = Some(target_date);
    } else {
        // parse the date for one or two -
        let date = match date.split("-").collect::<Vec<_>>().len() {
            3 => date,
            2 => format!("{}-{}", today.year().to_string(), date),
            1 => format!(
                "{}-{}-{}",
                today.year().to_string(),
                today.month().to_string(),
                date
            ),
            _ => {
                eprintln!("Invalid number of '-'. Expects YYYY-MM-DD, MM-DD or DD");
                return None;
            }
        };

        parsed_date = match chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
            Ok(date) => Some(date),
            Err(err) => {
                eprintln!("err: {:?}", err);
                eprintln!("Invalid date. Expects YYYY-MM-DD, MM-DD or DD");
                return None;
            }
        };
    }

    return parsed_date;
}

fn open_date(date: String) {
    let parsed_date = parse_day_string(date);
    let date = parsed_date
        .expect("Invalid date")
        .format("%Y-%m-%d")
        .to_string();
    open_file(date);
}

fn open_week() {
    let doto_path = get_doto_path();
    let combined_path = format!("{}/todo.md", doto_path);
    let today = chrono::Local::now().naive_local();
    // let start_of_week = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    let start = today;
    let start_of_range = start - Duration::days(4);
    let end_of_range = start + Duration::days(3);

    let mut combined_file =
        File::create(combined_path.clone()).expect("Failed to create combined file");
    let mut day = start_of_range;
    while day < end_of_range {
        let date = day.format("%Y-%m-%d").to_string();
        // TODO: make function that will create the file
        let path = PathBuf::from(get_or_make_file(date.clone()));
        if path.exists() {
            let file = File::open(path).expect("Failed to open file");
            let reader = BufReader::new(file);
            reader.lines().for_each(|line| {
                let line = line.expect("Failed to read line");
                combined_file
                    .write_fmt(format_args!("{}\n", line))
                    .expect("failed to write line");
            });
            combined_file
                .write_all(b"---\n")
                .expect("failed to write line");
        }
        day = day + Duration::days(1);
    }

    open_file("todo".to_string());

    let combined_file = File::open(combined_path).expect("Failed to open combined file");
    let reader = BufReader::new(combined_file);
    let mut current_file: Option<File> = None;

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        if line.eq("---") {
            continue; // dont' write the --- line
        }
        if line.starts_with("# ") {
            if let Some(mut file) = current_file {
                file.flush().expect("Failed to flush file");
            }
            let date_str = line.trim_start_matches("# ").trim();
            let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").expect("Invalid date");
            let path = PathBuf::from(format!("{}/{}.md", doto_path, date.format("%Y-%m-%d")));
            let file = File::create(path).expect("Failed to create file");
            current_file = Some(file);
        }
        if let Some(file) = &mut current_file {
            writeln!(file, "{}", line).expect("Failed to write line");
        }
    }

    if let Some(mut file) = current_file {
        file.flush().expect("Failed to flush file");
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Move all undone tasks to today's file
    #[arg(short, long)]
    undone: bool,

    /// Open file with date
    ///
    /// Expects YYYY-MM-DD, MM-DD or DD
    /// Will default to the current year and month if only DD is provided etc
    /// Alternatively you can pass a weekday (mon, tue, wed, thu, fri, sat or sun) from the last week
    #[arg(index = 1)]
    date: Option<String>,
}

fn main() {
    let args = Args::parse();

    if args.undone {
        move_undone();
    } else {
        match args.date {
            Some(date) => open_date(date),
            None => open_week(),
        }
    }
}
