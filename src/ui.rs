use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
};

use crate::app::{App, AppMode, InputField, StationViewMode};

const NEON_CYAN: Color = Color::Cyan;
const NEON_MAGENTA: Color = Color::Magenta;
const SELECTED_BG: Color = Color::Rgb(40, 0, 60);

pub fn draw(frame: &mut Frame, app: &App, table_state: &mut TableState) {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(size);

    draw_header(frame, chunks[0]);
    draw_now_playing(frame, app, chunks[1]);
    draw_filters(frame, app, chunks[2]);
    draw_station_list(frame, app, table_state, chunks[3]);
    draw_footer(frame, app, chunks[4]);
}

fn draw_header(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled("🎵 ", Style::default().fg(NEON_CYAN)),
        Span::styled(
            "cradio",
            Style::default()
                .fg(NEON_MAGENTA)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " — Internet Radio",
            Style::default().fg(NEON_CYAN).add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(NEON_MAGENTA)),
    );
    frame.render_widget(title, area);
}

fn draw_now_playing(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(err) = app.now_playing_error() {
        Line::from(vec![
            Span::styled(
                "Playback failed: ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(err, Style::default().fg(Color::White)),
        ])
    } else if let Some(station) = &app.current_station {
        let country = display_country(station);
        let language = display_language(station);
        let tags = display_tags(station, 24);
        let bitrate = display_bitrate(station);
        Line::from(vec![
            Span::styled(
                "▶ ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                truncate(&station.name, 40),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" | ", Style::default().fg(Color::DarkGray)),
            Span::styled(country, Style::default().fg(NEON_CYAN)),
            Span::styled(" | ", Style::default().fg(Color::DarkGray)),
            Span::styled(language, Style::default().fg(Color::White)),
            Span::styled(" | ", Style::default().fg(Color::DarkGray)),
            Span::styled(tags, Style::default().fg(NEON_MAGENTA)),
            Span::styled(" | ", Style::default().fg(Color::DarkGray)),
            Span::styled(bitrate, Style::default().fg(NEON_CYAN)),
        ])
    } else {
        Line::from(vec![Span::styled(
            "No station playing",
            Style::default().fg(Color::DarkGray),
        )])
    };

    let player_widget = Paragraph::new(content).alignment(Alignment::Left).block(
        Block::default()
            .title(" Now Playing ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if app.now_playing_error().is_some() {
                Color::Red
            } else {
                Color::Green
            })),
    );
    frame.render_widget(player_widget, area);
}

fn draw_filters(frame: &mut Frame, app: &App, area: Rect) {
    let filter_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(28),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(30),
            Constraint::Percentage(14),
        ])
        .split(area);

    let fields = [
        ("Name", &app.draft_name, InputField::Name),
        ("Country", &app.draft_country, InputField::Country),
        ("Lang", &app.draft_language, InputField::Language),
        ("Tags", &app.draft_tags, InputField::Tags),
        ("Bitrate", &app.draft_bitrate, InputField::Bitrate),
    ];

    for (i, (label, value, field)) in fields.iter().enumerate() {
        let is_active = matches!(&app.mode, AppMode::Filtering(f) if f == field);
        let border_style = if is_active {
            Style::default().fg(NEON_MAGENTA)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let value_style = if is_active {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::Gray)
        };

        let display = if is_active {
            format!("{}█", value)
        } else {
            value.to_string()
        };

        let widget = Paragraph::new(Span::styled(display, value_style)).block(
            Block::default()
                .title(Span::styled(
                    format!(" {} ", label),
                    Style::default().fg(NEON_CYAN),
                ))
                .borders(Borders::ALL)
                .border_style(border_style),
        );
        frame.render_widget(widget, filter_layout[i]);
    }
}

fn draw_station_list(frame: &mut Frame, app: &App, table_state: &mut TableState, area: Rect) {
    let header_cells = ["Name", "Country", "Language", "Tags", "Bitrate"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(NEON_CYAN)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )
        });
    let header = Row::new(header_cells).height(1).bottom_margin(0);

    let station_list = app.current_station_list();

    let rows: Vec<Row> = if app.view_mode == StationViewMode::Favorites && app.favorites_loading {
        vec![Row::new(vec![Cell::from(Span::styled(
            "Loading favorites...",
            Style::default().fg(Color::Yellow),
        ))])]
    } else if app.view_mode == StationViewMode::AllStations && app.loading {
        vec![Row::new(vec![Cell::from(Span::styled(
            "Loading stations...",
            Style::default().fg(Color::Yellow),
        ))])]
    } else if let Some(err) = app.active_error() {
        vec![Row::new(vec![Cell::from(Span::styled(
            format!("Error: {}", err),
            Style::default().fg(Color::Red),
        ))])]
    } else if station_list.is_empty() {
        let message = if app.view_mode == StationViewMode::Favorites {
            "No favorites yet. Press Space to add one."
        } else {
            "No stations found. Try different filters."
        };
        vec![Row::new(vec![Cell::from(Span::styled(
            message,
            Style::default().fg(Color::DarkGray),
        ))])]
    } else {
        station_list
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let is_playing = app
                    .current_station
                    .as_ref()
                    .map(|cs| cs.stationuuid == s.stationuuid)
                    .unwrap_or(false);
                let is_favorite = app.is_favorite(&s.stationuuid);

                let playing_prefix = if is_playing { "▶ " } else { "  " };
                let favorite_prefix = if is_favorite { "★ " } else { "" };
                let name = format!(
                    "{}{}{}",
                    playing_prefix,
                    favorite_prefix,
                    truncate(&s.name, 32)
                );
                let country = display_country(s);
                let language = display_language(s);
                let tags = display_tags(s, 30);
                let bitrate = display_bitrate(s);

                let style = if i == app.selected {
                    Style::default()
                        .bg(SELECTED_BG)
                        .fg(NEON_MAGENTA)
                        .add_modifier(Modifier::BOLD)
                } else if is_playing {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                };

                Row::new(vec![
                    Cell::from(name).style(style),
                    Cell::from(country).style(style),
                    Cell::from(language).style(style),
                    Cell::from(tags).style(style),
                    Cell::from(bitrate).style(style),
                ])
                .height(1)
            })
            .collect()
    };

    let title = app.stations_title();

    let table = Table::new(
        rows,
        [
            Constraint::Min(20),
            Constraint::Length(8),
            Constraint::Length(14),
            Constraint::Min(20),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(Span::styled(
                title,
                Style::default().fg(NEON_CYAN).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(NEON_MAGENTA)),
    )
    .row_highlight_style(
        Style::default()
            .bg(SELECTED_BG)
            .fg(NEON_MAGENTA)
            .add_modifier(Modifier::BOLD),
    );

    *table_state = TableState::default()
        .with_offset(app.scroll_offset)
        .with_selected(Some(app.selected));
    frame.render_stateful_widget(table, area, table_state);
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let keys = if matches!(app.mode, AppMode::Filtering(_)) {
        vec![
            key("Tab", "Next Field"),
            key("Enter", "Apply & Search"),
            key("Esc", "Cancel"),
        ]
    } else {
        vec![
            key("↑↓", "Navigate"),
            key("Enter", "Play"),
            key("Space", "Favorite"),
            key("f", "Favorites"),
            key("/", "Filter"),
            key("n/p", "Next/Prev Page"),
            key("+/-", "Volume"),
            key("s", "Stop"),
            key("q", "Quit"),
        ]
    };

    let split_index = keys.len().div_ceil(2);
    let (first_row, second_row) = keys.split_at(split_index);

    let mut first_spans: Vec<Span> = Vec::new();
    for (i, (k, desc)) in first_row.iter().enumerate() {
        if i > 0 {
            first_spans.push(Span::styled("  ", Style::default()));
        }
        first_spans.push(Span::styled(
            k.to_string(),
            Style::default()
                .fg(Color::Black)
                .bg(NEON_CYAN)
                .add_modifier(Modifier::BOLD),
        ));
        first_spans.push(Span::styled(
            format!(" {}", desc),
            Style::default().fg(Color::Gray),
        ));
    }

    let mut second_spans: Vec<Span> = Vec::new();
    for (i, (k, desc)) in second_row.iter().enumerate() {
        if i > 0 {
            second_spans.push(Span::styled("  ", Style::default()));
        }
        second_spans.push(Span::styled(
            k.to_string(),
            Style::default()
                .fg(Color::Black)
                .bg(NEON_CYAN)
                .add_modifier(Modifier::BOLD),
        ));
        second_spans.push(Span::styled(
            format!(" {}", desc),
            Style::default().fg(Color::Gray),
        ));
    }
    second_spans.push(Span::styled("  │  ", Style::default().fg(Color::DarkGray)));
    second_spans.push(Span::styled(
        format!("Vol: {}%", app.volume_display()),
        Style::default().fg(NEON_CYAN),
    ));

    let footer = Paragraph::new(vec![Line::from(first_spans), Line::from(second_spans)])
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(footer, area);
}

fn key<'a>(k: &'a str, desc: &'a str) -> (&'a str, &'a str) {
    (k, desc)
}

fn truncate(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        let truncated: String = chars[..max.saturating_sub(1)].iter().collect();
        format!("{}…", truncated)
    }
}

fn display_country(station: &crate::api::Station) -> String {
    if station.country_code.is_empty() {
        "N/A".to_string()
    } else {
        station.country_code.clone()
    }
}

fn display_language(station: &crate::api::Station) -> String {
    if station.language.is_empty() {
        "N/A".to_string()
    } else {
        truncate(&station.language, 12)
    }
}

fn display_tags(station: &crate::api::Station, max: usize) -> String {
    truncate(&station.tags, max)
}

fn display_bitrate(station: &crate::api::Station) -> String {
    if station.bitrate > 0 {
        format!("{} kbps", station.bitrate)
    } else {
        String::from("N/A")
    }
}

#[cfg(test)]
mod tests {
    use super::draw;
    use crate::{api::Station, app::App};
    use ratatui::{Terminal, backend::TestBackend, buffer::Buffer, widgets::TableState};

    fn station(id: &str) -> Station {
        Station {
            stationuuid: id.to_string(),
            name: format!("Station {}", id),
            url: format!("https://{}", id),
            url_resolved: String::new(),
            tags: String::new(),
            country_code: String::new(),
            language: String::new(),
            bitrate: 0,
        }
    }

    fn buffer_contains(buffer: &Buffer, needle: &str) -> bool {
        let area = buffer.area();
        let mut text = String::new();
        for y in 0..area.height {
            for x in 0..area.width {
                text.push_str(buffer[(x, y)].symbol());
            }
            text.push('\n');
        }
        text.contains(needle)
    }

    #[test]
    fn draw_renders_inline_playback_error() {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let mut app = App::new();
        let mut table_state = TableState::default();
        app.playback_error = Some("cvlc not found".to_string());

        terminal
            .draw(|frame| draw(frame, &app, &mut table_state))
            .expect("draw");

        let buffer = terminal.backend().buffer().clone();
        assert!(buffer_contains(&buffer, "Playback failed:"));
        assert!(buffer_contains(&buffer, "cvlc not found"));
    }

    #[test]
    fn draw_now_playing_shows_all_station_metadata_inline() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let mut app = App::new();
        let mut table_state = TableState::default();
        app.current_station = Some(Station {
            stationuuid: "id-1".to_string(),
            name: "Classic Vinyl HD".to_string(),
            url: "https://example.com".to_string(),
            url_resolved: String::new(),
            tags: "1930,1940,1950,1960,beautiful".to_string(),
            country_code: "US".to_string(),
            language: "english".to_string(),
            bitrate: 320,
        });

        terminal
            .draw(|frame| draw(frame, &app, &mut table_state))
            .expect("draw");

        let buffer = terminal.backend().buffer().clone();
        assert!(buffer_contains(&buffer, "Classic Vinyl HD"));
        assert!(buffer_contains(&buffer, "US"));
        assert!(buffer_contains(&buffer, "english"));
        assert!(buffer_contains(&buffer, "1930,1940,1950"));
        assert!(buffer_contains(&buffer, "320 kbps"));
    }

    #[test]
    fn draw_now_playing_uses_na_for_missing_metadata() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let mut app = App::new();
        let mut table_state = TableState::default();
        app.current_station = Some(Station {
            stationuuid: "id-2".to_string(),
            name: "Unknown Station".to_string(),
            url: "https://example.com".to_string(),
            url_resolved: String::new(),
            tags: String::new(),
            country_code: String::new(),
            language: String::new(),
            bitrate: 0,
        });

        terminal
            .draw(|frame| draw(frame, &app, &mut table_state))
            .expect("draw");

        let buffer = terminal.backend().buffer().clone();
        assert!(buffer_contains(&buffer, "Unknown Station"));
        assert!(buffer_contains(&buffer, "N/A"));
    }

    #[test]
    fn draw_uses_app_scroll_offset_for_table_state() {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let mut app = App::new();
        let mut table_state = TableState::default();
        app.stations = (0..12).map(|i| station(&i.to_string())).collect();
        app.selected = 8;
        app.scroll_offset = 4;

        terminal
            .draw(|frame| draw(frame, &app, &mut table_state))
            .expect("draw");

        assert_eq!(table_state.selected(), Some(8));
        assert_eq!(table_state.offset(), 4);
    }
}
