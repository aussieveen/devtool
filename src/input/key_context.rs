use crate::app::Tool;
use crate::state::token_generator::Focus as TokenFocus;

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub enum KeyContext {
    Global,
    List,
    ListSpecific(Tool),
    ListIgnore(Tool),
    Tool(Tool),
    ToolIgnore(Tool),
    Popup(Tool),
    TokenGen(TokenFocus),
}
