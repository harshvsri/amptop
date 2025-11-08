use crate::errors::{Error, Result};
use battery::{Manager, State};
use chrono::Utc;
use daemonize::Daemonize;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use std::{thread, time::Duration};

#[derive(Debug, Clone)]
pub struct BatterySnapshot {
    pub percent: f32,
    pub timestamp: i64,
    pub status: String,
}

pub struct BatteryDaemon {
    db_path: PathBuf,
    interval_secs: u64,
}

impl BatteryDaemon {
    pub fn new(interval_secs: u64) -> Self {
        Self {
            db_path: Self::get_db_path(),
            interval_secs,
        }
    }

    fn get_db_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let data_dir = PathBuf::from(home).join(".local/share/amptop");
        fs::create_dir_all(&data_dir).ok();
        data_dir.join("battery.db")
    }

    fn init_database(&self) -> Result<Connection> {
        let conn = Connection::open(&self.db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS battery_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                percent REAL NOT NULL,
                timestamp INTEGER NOT NULL,
                status TEXT NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON battery_logs(timestamp)",
            [],
        )?;
        Ok(conn)
    }

    fn collect_snapshot(&self) -> Result<Option<BatterySnapshot>> {
        if let Some(battery) = Manager::new()?.batteries()?.next() {
            let battery = battery?;

            let percent = battery
                .state_of_charge()
                .get::<battery::units::ratio::percent>();

            let timestamp = Utc::now().timestamp();

            let status = match battery.state() {
                State::Charging => "charging",
                State::Discharging => "discharging",
                State::Full => "full",
                State::Empty => "empty",
                _ => "unknown",
            };

            return Ok(Some(BatterySnapshot {
                percent,
                timestamp,
                status: status.to_string(),
            }));
        }
        Ok(None)
    }

    fn store_snapshot(&self, conn: &Connection, snapshot: &BatterySnapshot) -> Result<()> {
        conn.execute(
            "INSERT INTO battery_logs (percent, timestamp, status) VALUES (?1, ?2, ?3)",
            (&snapshot.percent, &snapshot.timestamp, &snapshot.status),
        )?;
        Ok(())
    }

    fn monitoring_loop(&self) -> Result<()> {
        let conn = self.init_database()?;

        loop {
            if let Some(snapshot) = self.collect_snapshot()? {
                self.store_snapshot(&conn, &snapshot)?;
            }
            thread::sleep(Duration::from_secs(self.interval_secs));
        }
    }

    pub fn start_daemon(&self) -> Result<()> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let pid_dir = PathBuf::from(home).join(".local/share/amptop");
        let pid_file = pid_dir.join("daemon.pid");

        fs::create_dir_all(&pid_dir)?;

        if pid_file.exists() {
            return Err(Error::DaemonAlreadyRunning);
        }

        let stdout = fs::File::create(pid_dir.join("daemon.out"))?;
        let stderr = fs::File::create(pid_dir.join("daemon.err"))?;

        let daemonize = Daemonize::new()
            .pid_file(pid_file)
            .working_directory(pid_dir)
            .stdout(stdout)
            .stderr(stderr);

        daemonize
            .start()
            .map_err(|e| Error::Daemonize(format!("{}", e)))?;
        self.monitoring_loop()
    }

    pub fn get_logs(limit: Option<usize>) -> Result<Vec<BatterySnapshot>> {
        let conn = Connection::open(Self::get_db_path())?;
        let mut stmt = if let Some(limit) = limit {
            conn.prepare(&format!(
                "SELECT percent, timestamp, status FROM battery_logs ORDER BY timestamp DESC LIMIT {}",
                limit
            ))?
        } else {
            conn.prepare(
                "SELECT percent, timestamp, status FROM battery_logs ORDER BY timestamp DESC",
            )?
        };

        let logs = stmt
            .query_map([], |row| {
                Ok(BatterySnapshot {
                    percent: row.get(0)?,
                    timestamp: row.get(1)?,
                    status: row.get(2)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(logs)
    }

    pub fn is_running() -> bool {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let pid_file = PathBuf::from(home).join(".local/share/amptop/daemon.pid");

        if !pid_file.exists() {
            return false;
        }

        if let Ok(pid_str) = fs::read_to_string(&pid_file) {
            if let Ok(pid) = pid_str.trim().parse::<i32>() {
                unsafe {
                    return libc::kill(pid, 0) == 0;
                }
            }
        }
        false
    }

    pub fn stop_daemon() -> Result<()> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let pid_file = PathBuf::from(home).join(".local/share/amptop/daemon.pid");

        if pid_file.exists() {
            let pid_str = fs::read_to_string(&pid_file)?;
            let pid: i32 = pid_str.trim().parse()?;
            unsafe {
                libc::kill(pid, libc::SIGTERM);
            }
            fs::remove_file(pid_file)?;
            Ok(())
        } else {
            Err(Error::DaemonNotRunning)
        }
    }
}
