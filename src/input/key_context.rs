use crate::app::Tool;
use crate::state::token_generator::Focus as TokenFocus;

#[derive(Eq, Hash, PartialEq, Clone)]
pub enum KeyContext {
    ErrorPopUp,
    Global,
    List,
    ListIgnore(Tool),
    Tool(Tool),
    ToolIgnore(Tool),
    Popup(Tool),
    TokenGen(TokenFocus),
}
