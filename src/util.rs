pub fn get_doto_path() -> String {
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
pub fn get_today_todo_file_path() -> String {
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let path = format!("{}/{}.md", get_doto_path(), date);
    path
}

#[allow(dead_code)]
pub fn get_today_filename() -> String {
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    return format!("{}", date);
}
