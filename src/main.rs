use crate::gui::LspInspector;
use log::LevelFilter;
use simplelog::{CombinedLogger, Config, WriteLogger};
use std::env;
use std::fs::File;

mod gui;
mod lsp;

fn main() -> iced::Result {
    let current_dir = env::current_dir().unwrap();
    let log_file_path = current_dir.join("lsp-inspector-debug.log");
    let log_file = File::create(log_file_path).unwrap();
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Info,
        Config::default(),
        log_file,
    )])
    .unwrap();

    iced::application(LspInspector::new, LspInspector::update, LspInspector::view)
        .subscription(LspInspector::subscription)
        .run()
}
