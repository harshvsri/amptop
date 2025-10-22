use crate::config::Units;
use crate::errors::Result;
use battery::units::{
    Unit,
    electric_potential::volt,
    energy::{joule, watt_hour},
    power::watt,
    ratio::{percent, ratio},
    thermodynamic_temperature::{degree_celsius, kelvin},
    time::second,
};
use log::error;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Gauge, Paragraph, Row, Table},
};
use std::time::Duration;

#[derive(Debug)]
pub struct BatteryInfo {
    battery: Option<battery::Battery>,
    manager: battery::Manager,
}

impl BatteryInfo {
    pub fn new() -> Result<Self> {
        let manager = battery::Manager::new()?;

        let battery = manager.batteries()?.flatten().next();

        if battery.is_none() {
            error!("No battery found in system");
        }

        Ok(Self { battery, manager })
    }

    pub fn refresh(&mut self) -> Result<()> {
        if let Some(ref mut battery) = self.battery {
            self.manager.refresh(battery)?;
        }
        Ok(())
    }

    pub fn has_battery(&self) -> bool {
        self.battery.is_some()
    }

    pub fn state_of_charge(&self) -> Option<(f64, f64)> {
        self.battery.as_ref().map(|b| {
            let ratio_value = f64::from(b.state_of_charge().get::<ratio>());
            let percent_value = f64::from(b.state_of_charge().get::<percent>());
            (ratio_value, percent_value)
        })
    }

    pub fn vendor(&self) -> Option<&str> {
        self.battery.as_ref().and_then(|b| b.vendor())
    }

    pub fn model(&self) -> Option<&str> {
        self.battery.as_ref().and_then(|b| b.model())
    }

    pub fn serial_number(&self) -> Option<&str> {
        self.battery.as_ref().and_then(|b| b.serial_number())
    }

    pub fn technology(&self) -> Option<String> {
        self.battery.as_ref().map(|b| format!("{}", b.technology()))
    }

    pub fn state(&self) -> Option<String> {
        self.battery.as_ref().map(|b| format!("{}", b.state()))
    }

    fn battery_state(&self) -> Option<battery::State> {
        self.battery.as_ref().map(|b| b.state())
    }

    pub fn cycle_count(&self) -> Option<u32> {
        self.battery.as_ref().and_then(|b| b.cycle_count())
    }

    fn energy_rate(&self) -> Option<String> {
        self.battery.as_ref().map(|b| {
            format!(
                "{:.2} {}",
                b.energy_rate().get::<watt>(),
                watt::abbreviation()
            )
        })
    }

    fn voltage(&self) -> Option<String> {
        self.battery
            .as_ref()
            .map(|b| format!("{:.2} {}", b.voltage().get::<volt>(), volt::abbreviation()))
    }

    fn capacity(&self) -> Option<String> {
        self.battery.as_ref().map(|b| {
            format!(
                "{:.2} {}",
                b.state_of_health().get::<percent>(),
                percent::abbreviation()
            )
        })
    }

    fn current_energy(&self, units: Units) -> Option<String> {
        self.battery.as_ref().map(|b| match units {
            Units::Human => format!(
                "{:.2} {}",
                b.energy().get::<watt_hour>(),
                watt_hour::abbreviation()
            ),
            Units::Si => format!("{:.2} {}", b.energy().get::<joule>(), joule::abbreviation()),
        })
    }

    fn energy_full(&self, units: Units) -> Option<String> {
        self.battery.as_ref().map(|b| match units {
            Units::Human => format!(
                "{:.2} {}",
                b.energy_full().get::<watt_hour>(),
                watt_hour::abbreviation()
            ),
            Units::Si => format!(
                "{:.2} {}",
                b.energy_full().get::<joule>(),
                joule::abbreviation()
            ),
        })
    }

    fn energy_full_design(&self, units: Units) -> Option<String> {
        self.battery.as_ref().map(|b| match units {
            Units::Human => format!(
                "{:.2} {}",
                b.energy_full_design().get::<watt_hour>(),
                watt_hour::abbreviation()
            ),
            Units::Si => format!(
                "{:.2} {}",
                b.energy_full_design().get::<joule>(),
                joule::abbreviation()
            ),
        })
    }

    fn time_to_full(&self) -> Option<String> {
        self.battery.as_ref().and_then(|b| {
            b.time_to_full().map(|time| {
                humantime::format_duration(Duration::from_secs(time.get::<second>() as u64))
                    .to_string()
            })
        })
    }

    fn time_to_empty(&self) -> Option<String> {
        self.battery.as_ref().and_then(|b| {
            b.time_to_empty().map(|time| {
                humantime::format_duration(Duration::from_secs(time.get::<second>() as u64))
                    .to_string()
            })
        })
    }

    fn temperature(&self, units: Units) -> Option<String> {
        self.battery.as_ref().and_then(|b| {
            b.temperature().map(|temp| match units {
                Units::Human => format!(
                    "{:.2} {}",
                    temp.get::<degree_celsius>(),
                    degree_celsius::abbreviation()
                ),
                Units::Si => format!("{:.2} {}", temp.get::<kelvin>(), kelvin::abbreviation()),
            })
        })
    }

    pub fn draw_state_of_charge_bar(&self, frame: &mut Frame, area: Rect) {
        if self.has_battery() {
            if let Some((ratio_value, percent_value)) = self.state_of_charge() {
                let label = format!("{:.1}%", percent_value);

                let gauge_color = match () {
                    _ if ratio_value > 0.3 => Color::Green,
                    _ if ratio_value > 0.15 => Color::Yellow,
                    _ => Color::Red,
                };

                let gauge = Gauge::default()
                    .block(
                        Block::default()
                            .title(" State of charge ")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::White)),
                    )
                    .ratio(ratio_value)
                    .gauge_style(Style::default().fg(gauge_color))
                    .label(Span::styled(
                        label,
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    ));

                frame.render_widget(gauge, area);
            }
        } else {
            let block = Block::default()
                .title(" State of charge ")
                .borders(Borders::ALL);
            let text = Paragraph::new("No battery detected")
                .block(block)
                .alignment(Alignment::Center);
            frame.render_widget(text, area);
        }
    }

    pub fn draw_common_info(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Device Information ")
            .borders(Borders::ALL);

        if self.has_battery() {
            let tech = self.technology().unwrap_or_else(|| "N/A".to_string());
            let state = self.state().unwrap_or_else(|| "N/A".to_string());
            let cycles = self
                .cycle_count()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "N/A".to_string());

            let items = vec![
                ["Vendor", self.vendor().unwrap_or("N/A")],
                ["Model", self.model().unwrap_or("N/A")],
                ["S/N", self.serial_number().unwrap_or("N/A")],
                ["Technology", &tech],
                ["Charge state", &state],
                ["Cycles count", &cycles],
            ];

            Self::draw_info_list(&items, block, frame, area);
        } else {
            let text = Paragraph::new("No battery detected")
                .block(block)
                .alignment(Alignment::Center);
            frame.render_widget(text, area);
        }
    }

    pub fn draw_energy_info(&self, frame: &mut Frame, area: Rect, units: Units) {
        let block = Block::default().title(" Energy ").borders(Borders::ALL);

        if self.has_battery() {
            let consumption = self.energy_rate().unwrap_or_else(|| "N/A".to_string());
            let voltage = self.voltage().unwrap_or_else(|| "N/A".to_string());
            let capacity = self.capacity().unwrap_or_else(|| "N/A".to_string());
            let current = self
                .current_energy(units)
                .unwrap_or_else(|| "N/A".to_string());
            let last_full = self.energy_full(units).unwrap_or_else(|| "N/A".to_string());
            let full_design = self
                .energy_full_design(units)
                .unwrap_or_else(|| "N/A".to_string());

            let consumption_label = match self.battery_state() {
                Some(battery::State::Charging) => "Charging with",
                Some(battery::State::Discharging) => "Discharging with",
                _ => "Consumption",
            };

            let items = vec![
                [consumption_label, &consumption],
                ["Voltage", &voltage],
                ["Capacity", &capacity],
                ["Current", &current],
                ["Last full", &last_full],
                ["Full design", &full_design],
            ];

            Self::draw_info_list(&items, block, frame, area);
        } else {
            let text = Paragraph::new("No battery detected")
                .block(block)
                .alignment(Alignment::Center);
            frame.render_widget(text, area);
        }
    }

    pub fn draw_timing_info(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().title(" Timings ").borders(Borders::ALL);

        if self.has_battery() {
            let time_to_full = self.time_to_full().unwrap_or_else(|| "N/A".to_string());
            let time_to_empty = self.time_to_empty().unwrap_or_else(|| "N/A".to_string());

            let items = vec![
                ["Time to full", &time_to_full],
                ["Time to empty", &time_to_empty],
            ];

            Self::draw_info_list(&items, block, frame, area);
        } else {
            let text = Paragraph::new("No battery detected")
                .block(block)
                .alignment(Alignment::Center);
            frame.render_widget(text, area);
        }
    }

    pub fn draw_environment_info(&self, frame: &mut Frame, area: Rect, units: Units) {
        let block = Block::default()
            .title(" Environments ")
            .borders(Borders::ALL);

        if self.has_battery() {
            let temperature = self.temperature(units).unwrap_or_else(|| "N/A".to_string());

            let items = vec![["Temperature", &temperature]];

            Self::draw_info_list(&items, block, frame, area);
        } else {
            let text = Paragraph::new("No battery detected")
                .block(block)
                .alignment(Alignment::Center);
            frame.render_widget(text, area);
        }
    }

    fn draw_info_list(items: &[[&str; 2]], block: Block, frame: &mut Frame, area: Rect) {
        let rows = items
            .iter()
            .map(|item| Row::new(item.iter().map(|s| s.to_string())));

        let table = Table::new(rows, [Constraint::Length(17), Constraint::Length(17)]).block(block);

        frame.render_widget(table, area);
    }

    pub fn draw_drain_graph(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Drain Graph ")
            .borders(Borders::ALL);

        let text = Paragraph::new("Graph will be implemented here")
            .block(block)
            .alignment(Alignment::Center);

        frame.render_widget(text, area);
    }
}
