use crate::config::Unit;
use crate::daemon::BatteryDaemon;
use crate::info::BatteryInfo;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, Gauge, Paragraph, Row, Table},
};

pub fn draw_state_of_charge_bar(battery: &BatteryInfo, frame: &mut Frame, area: Rect) {
    if battery.has_battery() {
        if let Some((ratio_value, percent_value)) = battery.state_of_charge() {
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

pub fn draw_common_info(battery: &BatteryInfo, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Device Information ")
        .borders(Borders::ALL);

    if battery.has_battery() {
        let tech = battery.technology().unwrap_or_else(|| "N/A".to_string());
        let state = battery.state().unwrap_or_else(|| "N/A".to_string());
        let cycles = battery
            .cycle_count()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let items = vec![
            ["Vendor", battery.vendor().unwrap_or("N/A")],
            ["Model", battery.model().unwrap_or("N/A")],
            ["S/N", battery.serial_number().unwrap_or("N/A")],
            ["Technology", &tech],
            ["Charge state", &state],
            ["Cycles count", &cycles],
        ];

        draw_info_list(&items, block, frame, area);
    } else {
        let text = Paragraph::new("No battery detected")
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(text, area);
    }
}

pub fn draw_energy_info(battery: &BatteryInfo, frame: &mut Frame, area: Rect, unit: Unit) {
    let block = Block::default().title(" Energy ").borders(Borders::ALL);

    if battery.has_battery() {
        let consumption = battery.energy_rate().unwrap_or_else(|| "N/A".to_string());
        let voltage = battery.voltage().unwrap_or_else(|| "N/A".to_string());
        let capacity = battery.capacity().unwrap_or_else(|| "N/A".to_string());
        let current = battery
            .current_energy(unit)
            .unwrap_or_else(|| "N/A".to_string());
        let last_full = battery
            .energy_full(unit)
            .unwrap_or_else(|| "N/A".to_string());
        let full_design = battery
            .energy_full_design(unit)
            .unwrap_or_else(|| "N/A".to_string());

        let consumption_label = match battery.battery_state() {
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

        draw_info_list(&items, block, frame, area);
    } else {
        let text = Paragraph::new("No battery detected")
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(text, area);
    }
}

pub fn draw_timing_info(battery: &BatteryInfo, frame: &mut Frame, area: Rect) {
    let block = Block::default().title(" Timings ").borders(Borders::ALL);

    if battery.has_battery() {
        let time_to_full = battery.time_to_full().unwrap_or_else(|| "N/A".to_string());
        let time_to_empty = battery.time_to_empty().unwrap_or_else(|| "N/A".to_string());

        let items = vec![
            ["Time to full", &time_to_full],
            ["Time to empty", &time_to_empty],
        ];

        draw_info_list(&items, block, frame, area);
    } else {
        let text = Paragraph::new("No battery detected")
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(text, area);
    }
}

pub fn draw_environment_info(battery: &BatteryInfo, frame: &mut Frame, area: Rect, unit: Unit) {
    let block = Block::default()
        .title(" Environments ")
        .borders(Borders::ALL);

    if battery.has_battery() {
        let temperature = battery
            .temperature(unit)
            .unwrap_or_else(|| "N/A".to_string());

        let items = vec![["Temperature", &temperature]];

        draw_info_list(&items, block, frame, area);
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

pub fn draw_drain_graph(frame: &mut Frame, area: Rect) {
    use chrono::{Local, TimeZone, Timelike};

    let block = Block::default()
        .title(" Battery History (Green: Charging | Red: Discharging | Blue: Full) ")
        .borders(Borders::ALL);

    let logs_result = BatteryDaemon::get_logs(Some(500));

    match logs_result {
        Ok(mut logs) if !logs.is_empty() => {
            // Reverse to show oldest to newest (left to right)
            logs.reverse();

            // Sample logs to fit available width (account for borders and Y-axis labels)
            let max_points = (area.width.saturating_sub(10)) as usize;
            let sample_step = if logs.len() > max_points && max_points > 0 {
                logs.len() / max_points
            } else {
                1
            };

            let sampled_logs: Vec<_> = logs.iter().step_by(sample_step.max(1)).collect();

            if sampled_logs.is_empty() {
                let text = Paragraph::new("No data to display")
                    .block(block)
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Yellow));
                frame.render_widget(text, area);
                return;
            }

            // Get first and last timestamps for labels
            let first_timestamp = sampled_logs.first().unwrap().timestamp;
            let last_timestamp = sampled_logs.last().unwrap().timestamp;

            let first_dt = Local.timestamp_opt(first_timestamp, 0).unwrap();
            let last_dt = Local.timestamp_opt(last_timestamp, 0).unwrap();

            // Create X-axis labels - only show first and last time, evenly distributed
            let x_labels = vec![
                Span::raw(format!("{:02}:{:02}", first_dt.hour(), first_dt.minute())),
                Span::raw(""),
                Span::raw(""),
                Span::raw(""),
                Span::raw(format!("{:02}:{:02}", last_dt.hour(), last_dt.minute())),
            ];

            // Create X-axis bounds (scaled to 0.0-4.0 for 5 label positions)
            let x_bounds = [0.0, 4.0];

            // Prepare data points - scale x coordinates to 0.0-4.0 range
            let scale_factor = 4.0 / (sampled_logs.len() - 1).max(1) as f64;
            let data_points: Vec<(f64, f64)> = sampled_logs
                .iter()
                .enumerate()
                .map(|(i, log)| (i as f64 * scale_factor, log.percent as f64))
                .collect();

            // Calculate dominant color based on most common status
            let dominant_color = {
                let mut charging_count = 0;
                let mut discharging_count = 0;
                let mut full_count = 0;

                for log in &sampled_logs {
                    match log.status.as_str() {
                        "charging" => charging_count += 1,
                        "discharging" => discharging_count += 1,
                        "full" => full_count += 1,
                        _ => {}
                    }
                }

                if discharging_count > charging_count && discharging_count > full_count {
                    Color::Red
                } else if charging_count > full_count {
                    Color::Green
                } else if full_count > 0 {
                    Color::Blue
                } else {
                    Color::Cyan
                }
            };

            // Create dataset with Bar marker for solid vertical bars
            let dataset = Dataset::default()
                .marker(ratatui::symbols::Marker::Bar)
                .style(Style::default().fg(dominant_color))
                .data(&data_points);

            // Create X-axis without title
            let x_axis = Axis::default()
                .style(Style::default().fg(Color::Gray))
                .bounds(x_bounds)
                .labels(x_labels);

            // Create Y-axis with percentage labels (0%, 10%, 20%, ..., 100%)
            let y_labels = vec![
                Span::raw("0%"),
                Span::raw("10%"),
                Span::raw("20%"),
                Span::raw("30%"),
                Span::raw("40%"),
                Span::raw("50%"),
                Span::raw("60%"),
                Span::raw("70%"),
                Span::raw("80%"),
                Span::raw("90%"),
                Span::raw("100%"),
            ];

            let y_axis = Axis::default()
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, 100.0])
                .labels(y_labels);

            // Create the chart
            let chart = Chart::new(vec![dataset])
                .block(block)
                .x_axis(x_axis)
                .y_axis(y_axis);

            frame.render_widget(chart, area);
        }
        Ok(_) => {
            // No logs available
            let text = Paragraph::new("No historical data available\n\nStart the daemon to collect data:\namptop daemon start --interval 60")
                .block(block)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Yellow));
            frame.render_widget(text, area);
        }
        Err(e) => {
            // Error loading logs
            let text = Paragraph::new(format!("Error loading data:\n{}", e))
                .block(block)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Red));
            frame.render_widget(text, area);
        }
    }
}
