use crate::app::Tool;
use crate::tools::token_generator::state::Focus as TokenFocus;

#[derive(Eq, Hash, PartialEq, Clone)]
pub enum KeyContext {
    Popup,
    Global,
    List,
    Config,
    Logs,
    ToolConfig(Tool),
    Tool(Tool),
    ToolIgnore(Tool),
    Editing(Tool),
    ToolConfigEditing(Tool),
    TokenGen(TokenFocus),
}
