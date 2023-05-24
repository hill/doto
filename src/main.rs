use chrono::{Datelike, NaiveDate};
use clap::Parser;
use std::process::Command;

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

fn get_today_filename() -> String {
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    format!("{}", date)
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
            let file_date = chrono::NaiveDate::parse_from_str(
                f.file_name()
                    .expect("Unable to read file name")
                    .to_str()
                    .unwrap(),
                "%Y-%m-%d.md",
            )
            .unwrap();
            file_date < chrono::Local::now().naive_local().date()
        })
        .collect::<Vec<_>>();

    // loop through all todo files and find lines starting with "- [ ]"
    let mut undone_task_count = 0;
    for file in todo_files {
        let file_content = std::fs::read_to_string(file.clone()).unwrap();
        let lines = file_content.lines().collect::<Vec<_>>();
        let undone_tasks = lines
            .iter()
            .filter(|l| l.starts_with("- [ ]")) // append undone tasks to today's todo file
            .map(|l| {
                let filename = file.file_name().unwrap().to_str().unwrap();
                let truncated_file_name = &filename[..filename.len() - 3]; // Remove last three characters

                format!("{} ({})", l, truncated_file_name)
            })
            .collect::<Vec<_>>();

        let updated_file_content = lines
            .iter()
            .map(|l| {
                if l.starts_with("- [ ]") {
                    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
                    format!("- [>] ({}) {}", date, &l[6..])
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

fn open_file(filename: String) {
    let doto_path = get_doto_path();
    let todo_file = format!("{}/{}.md", doto_path, filename);

    // create todo file if it doesn't exist
    if !std::path::Path::new(&todo_file).exists() {
        std::fs::write(&todo_file, format!("# {}", filename))
            .expect("Unable to write a new todo file");
    }

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
            "today" => parsed_date = Some(today),
            "prev" | "yes" | "yesterday" => parsed_date = Some(today.pred_opt().unwrap()),
            "next" | "tom" | "tomorrow" => parsed_date = Some(today.succ_opt().unwrap()),
            _ => {
                return None;
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
        while target_date.weekday() != target_weekday {
            if let Some(date) = target_date.pred_opt() {
                target_date = date;
            }
        }

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
            None => open_today(),
        }
    }
}
