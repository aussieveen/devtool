use crate::config::loader::ConfigFile;
use crate::config::model::Config;
use crate::event::sender::EventSender;
use crate::popup::model::Popup;
use crate::state::app::AppFocus;

pub struct PluginContext<'a> {
    pub config:        &'a mut Config,
    pub config_loader: &'a ConfigFile,
    pub sender:        &'a EventSender,
    pub popup:         &'a mut Option<Popup>,
    pub focus:         &'a mut AppFocus,
}