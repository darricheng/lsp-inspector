use crate::gui::LspInspector;
use clap::Parser;
use log::LevelFilter;
use simplelog::{CombinedLogger, Config, WriteLogger};
use std::env;
use std::fs::File;

mod gui;
mod lsp;

#[derive(Parser, Debug)]
#[command(version = "0.01")]
#[command(about = "GUI view of LSP messages being sent", long_about = None)]
struct Cli {
    #[arg(index = 1)]
    lsp_command: String,
}

fn main() -> iced::Result {
    let cli = Cli::parse();
    let lsp_command = cli.lsp_command;

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
        .subscription(LspInspector::subscription(lsp_command))
        .run()
}
