use crate::app::Tool;
use crate::state::token_generator::Focus as TokenFocus;

#[derive(Eq, Hash, PartialEq, Clone)]
pub enum KeyContext {
    ErrorPopUp,
    Global,
    List,
    Config,
    ToolConfig(Tool),
    Tool(Tool),
    ToolIgnore(Tool),
    Popup(Tool),
    ToolConfigPopup(Tool),
    TokenGen(TokenFocus),
}
