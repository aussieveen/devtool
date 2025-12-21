use ratatui::widgets::ListState;

#[derive(Debug)]
pub struct MenuState{
    pub items: Vec<&'static str>,
    pub state: ListState
}

#[derive(Debug)]
pub enum ContentState{
    Home,
    DiffChecker,
    TokenGenerator
}

impl ContentState {
    fn as_str(&self) -> &'static str {
        match self {
            ContentState::Home => "Home",
            ContentState::DiffChecker => "Diff Checker",
            ContentState::TokenGenerator => "Token Generator"
        }
    }
}

#[derive(Debug)]
pub enum BlockState{
    Menu,
    Content
}

#[derive(Debug)]
pub struct State{
    pub menu: MenuState,
    pub content: ContentState,
    pub block: BlockState
}

impl State {
    pub(crate) fn default() -> State {
        Self {
            menu: MenuState {
                items: vec![
                    ContentState::Home.as_str(),
                    ContentState::TokenGenerator.as_str(),
                    ContentState::DiffChecker.as_str()
                ],
                state: ListState::default().with_selected(Some(0)),
            },
            content: ContentState::Home,
            block: BlockState::Menu
        }
    }
}