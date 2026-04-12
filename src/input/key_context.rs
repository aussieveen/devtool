use crate::app::Tool;
use crate::state::token_generator::Focus as TokenFocus;

#[derive(Eq, Hash, PartialEq, Clone)]
pub enum KeyContext {
    Error,
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
