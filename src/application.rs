use crate::config::Config;
use crate::errors::{Error, Result};
use crate::ui::BatteryInfo;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
};

#[derive(Debug)]
pub struct Application {
    config: Config,
    exit: bool,
    battery_info: BatteryInfo,
}

impl Application {
    pub fn init(config: Config) -> Result<Self> {
        let battery_info = BatteryInfo::new()?;

        Ok(Self {
            config,
            exit: false,
            battery_info,
        })
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            if event::poll(*self.config.delay())
                .map_err(|e| Error::Crossterm(format!("Event poll error: {}", e)))?
            {
                self.handle_events()?;
            } else {
                self.battery_info.refresh()?;
            }
        }
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read().map_err(|e| Error::Crossterm(format!("Event read error: {}", e)))? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => self.exit(),
            KeyCode::Char('c')
                if key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                self.exit()
            }
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let main_columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(40), Constraint::Min(20)])
            .split(frame.area());

        let left_column = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(10),
                Constraint::Length(9),
                Constraint::Length(5),
                Constraint::Min(4),
            ])
            .split(main_columns[0]);

        self.battery_info
            .draw_state_of_charge_bar(frame, left_column[0]);
        self.battery_info.draw_common_info(frame, left_column[1]);
        self.battery_info
            .draw_energy_info(frame, left_column[2], self.config.units());
        self.battery_info.draw_timing_info(frame, left_column[3]);
        self.battery_info
            .draw_environment_info(frame, left_column[4], self.config.units());
        self.battery_info.draw_drain_graph(frame, main_columns[1]);
    }
}
