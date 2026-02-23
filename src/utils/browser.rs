pub fn open_link_in_browser(link: String) -> Result<(), String> {
    webbrowser::open(link.as_str()).map_err(|e| e.to_string())
}
