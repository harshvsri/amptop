use crate::daemon::BatteryDaemon;
use crate::errors::Result;
use clap::{Parser, Subcommand};
use std::time::Duration;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Units {
    Human,
    Si,
}

#[derive(Parser, Debug)]
#[command(name = "amptop")]
#[command(about = "Interactive battery statistics", long_about = None)]
#[command(version)]
pub struct Config {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(
        short,
        long,
        default_value = "1",
        value_parser = Config::parse_duration
    )]
    /// Delay between updates, in seconds (TUI mode only)
    delay: Duration,

    #[arg(
        short,
        long,
        default_value = "human",
        value_parser = Config::parse_unit
    )]
    /// Measurement units displayed, possible values (human, si) (TUI mode only)
    units: Units,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage the battery monitoring daemon
    #[command(name = "daemon")]
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum DaemonAction {
    /// Start the daemon to collect battery statistics in the background
    Start {
        #[arg(short, long, default_value = "60")]
        /// Interval in seconds between battery readings (recommended: 60-300)
        interval: u64,
    },
    /// Stop the running daemon
    Stop,
    /// Check if daemon is currently running
    Status,
}

impl Config {
    pub fn delay(&self) -> &Duration {
        &self.delay
    }

    pub fn units(&self) -> Units {
        self.units
    }

    fn parse_duration(s: &str) -> std::result::Result<Duration, String> {
        match s.parse::<u64>() {
            Ok(seconds) if seconds > 0 => Ok(Duration::from_secs(seconds)),
            _ => Err(format!("{} isn't a positive number", s)),
        }
    }

    fn parse_unit(s: &str) -> std::result::Result<Units, String> {
        match s {
            _ if s.eq_ignore_ascii_case("human") => Ok(Units::Human),
            _ if s.eq_ignore_ascii_case("SI") => Ok(Units::Si),
            _ => Err(format!("{} isn't a valid unit", s)),
        }
    }

    pub fn handle_command(&self) -> Result<bool> {
        if let Some(ref command) = self.command {
            match command {
                Commands::Daemon { action } => match action {
                    DaemonAction::Start { interval } => {
                        let daemon = BatteryDaemon::new(*interval);
                        match daemon.start_daemon() {
                            Ok(_) => println!("Daemon started successfully"),
                            Err(e) => eprintln!("Failed to start daemon: {}", e),
                        }
                    }
                    DaemonAction::Stop => match BatteryDaemon::stop_daemon() {
                        Ok(_) => println!("Daemon stopped successfully"),
                        Err(e) => eprintln!("Failed to stop daemon: {}", e),
                    },
                    DaemonAction::Status => {
                        if BatteryDaemon::is_running() {
                            println!("Daemon is running");
                        } else {
                            println!("Daemon is not running");
                        }
                    }
                },
            }
            return Ok(true);
        }
        Ok(false)
    }
}
