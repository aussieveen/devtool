pub fn open_link_in_browser(link: String) {
    webbrowser::open(link.as_str()).expect("Failed to open link");
}
