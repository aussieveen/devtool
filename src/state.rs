use ratatui::widgets::ListState;

#[derive(Debug)]
pub struct MenuState{
    pub items: Vec<&'static str>,
    pub state: ListState
}


#[derive(Debug, Copy)]
#[derive(Clone)]
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

#[derive(Debug)]
pub enum ContentState {
    Home,
    DiffChecker,
    TokenGenerator,
}

impl ContentState {
    pub fn title(&self) -> &'static str {
        match self {
            ContentState::Home => "Home",
            ContentState::DiffChecker => "PR Diff Checker",
            ContentState::TokenGenerator => "M2M Auth0 Token Generator",
        }
    }

    pub fn content(&self) -> &'static str {
        match self {
            ContentState::Home => "This is the home page",
            ContentState::DiffChecker => "This is the PR Diff Checker",
            ContentState::TokenGenerator => "This is the token generator",
        }
    }

    pub fn menu_entry(&self) -> &'static str {
        match self {
            ContentState::Home => "Home",
            ContentState::DiffChecker => "Diff Checker",
            ContentState::TokenGenerator => "Token Generator",
        }
    }
}


impl State {
    pub(crate) fn default() -> State {
        Self {
            menu: MenuState {
                items: vec![
                    ContentState::Home.menu_entry(),
                    ContentState::DiffChecker.menu_entry(),
                    ContentState::TokenGenerator.menu_entry()
                ],
                state: ListState::default().with_selected(Some(0)),
            },
            content: ContentState::Home,
            block: BlockState::Menu
        }
    }
}