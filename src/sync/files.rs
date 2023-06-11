pub fn sync_files() {
    let Some(access_token) = crate::sync::auth::get_access_token() else {
        println!("No access token found, please login");
        return;
    };
    let todo_files = crate::todo::get_all_files();

    println!("Using access token: {}", access_token);
    println!("Found {} todo files to sync", todo_files.len());
}
