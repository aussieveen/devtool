pub fn open_link_in_browser(link: &str) -> Result<(), String> {
    webbrowser::open(link).map_err(|e| e.to_string())
}
