mod application;
mod config;
mod daemon;
mod errors;
mod info;
mod ui;

use crate::config::Config;
use crate::errors::Result;
use application::Application;
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;

fn main() -> Result<()> {
    let config = Config::parse();
    if config.handle_command().unwrap_or(false) {
        return Ok(());
    }

    let mut stdout = std::io::stdout();

    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    let app_result = Application::init(config)?.run(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    app_result
}
