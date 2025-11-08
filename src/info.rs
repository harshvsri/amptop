use crate::config::Unit;
use crate::errors::Result;
use battery::units::{
    Unit as _,
    electric_potential::volt,
    energy::{joule, watt_hour},
    power::watt,
    ratio::{percent, ratio},
    thermodynamic_temperature::{degree_celsius, kelvin},
    time::second,
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

    pub fn battery_state(&self) -> Option<battery::State> {
        self.battery.as_ref().map(|b| b.state())
    }

    pub fn cycle_count(&self) -> Option<u32> {
        self.battery.as_ref().and_then(|b| b.cycle_count())
    }

    pub fn energy_rate(&self) -> Option<String> {
        self.battery.as_ref().map(|b| {
            format!(
                "{:.2} {}",
                b.energy_rate().get::<watt>(),
                watt::abbreviation()
            )
        })
    }

    pub fn voltage(&self) -> Option<String> {
        self.battery
            .as_ref()
            .map(|b| format!("{:.2} {}", b.voltage().get::<volt>(), volt::abbreviation()))
    }

    pub fn capacity(&self) -> Option<String> {
        self.battery.as_ref().map(|b| {
            format!(
                "{:.2} {}",
                b.state_of_health().get::<percent>(),
                percent::abbreviation()
            )
        })
    }

    pub fn current_energy(&self, unit: Unit) -> Option<String> {
        self.battery.as_ref().map(|b| match unit {
            Unit::Human => format!(
                "{:.2} {}",
                b.energy().get::<watt_hour>(),
                watt_hour::abbreviation()
            ),
            Unit::Si => format!("{:.2} {}", b.energy().get::<joule>(), joule::abbreviation()),
        })
    }

    pub fn energy_full(&self, units: Unit) -> Option<String> {
        self.battery.as_ref().map(|b| match units {
            Unit::Human => format!(
                "{:.2} {}",
                b.energy_full().get::<watt_hour>(),
                watt_hour::abbreviation()
            ),
            Unit::Si => format!(
                "{:.2} {}",
                b.energy_full().get::<joule>(),
                joule::abbreviation()
            ),
        })
    }

    pub fn energy_full_design(&self, units: Unit) -> Option<String> {
        self.battery.as_ref().map(|b| match units {
            Unit::Human => format!(
                "{:.2} {}",
                b.energy_full_design().get::<watt_hour>(),
                watt_hour::abbreviation()
            ),
            Unit::Si => format!(
                "{:.2} {}",
                b.energy_full_design().get::<joule>(),
                joule::abbreviation()
            ),
        })
    }

    pub fn time_to_full(&self) -> Option<String> {
        self.battery.as_ref().and_then(|b| {
            b.time_to_full().map(|time| {
                humantime::format_duration(Duration::from_secs(time.get::<second>() as u64))
                    .to_string()
            })
        })
    }

    pub fn time_to_empty(&self) -> Option<String> {
        self.battery.as_ref().and_then(|b| {
            b.time_to_empty().map(|time| {
                humantime::format_duration(Duration::from_secs(time.get::<second>() as u64))
                    .to_string()
            })
        })
    }

    pub fn temperature(&self, units: Unit) -> Option<String> {
        self.battery.as_ref().and_then(|b| {
            b.temperature().map(|temp| match units {
                Unit::Human => format!(
                    "{:.2} {}",
                    temp.get::<degree_celsius>(),
                    degree_celsius::abbreviation()
                ),
                Unit::Si => format!("{:.2} {}", temp.get::<kelvin>(), kelvin::abbreviation()),
            })
        })
    }
}
