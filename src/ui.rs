use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState},
};

use crate::app::{App, AppMode, InputField};

const NEON_CYAN: Color = Color::Cyan;
const NEON_MAGENTA: Color = Color::Magenta;
const SELECTED_BG: Color = Color::Rgb(40, 0, 60);

pub fn draw(frame: &mut Frame, app: &App, table_state: &mut TableState) {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Length(3), // now playing
            Constraint::Length(5), // filters
            Constraint::Min(5),    // station list
            Constraint::Length(2), // footer / keybindings
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
    let content = if let Some(station) = &app.current_station {
        let tags = truncate(&station.tags, 30);
        let country = if station.country_code.is_empty() {
            "N/A".to_string()
        } else {
            station.country_code.clone()
        };
        Line::from(vec![
            Span::styled("▶ ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(
                truncate(&station.name, 40),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
            Span::styled(country, Style::default().fg(NEON_CYAN)),
            Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
            Span::styled(tags, Style::default().fg(NEON_MAGENTA)),
        ])
    } else {
        Line::from(vec![Span::styled(
            "No station playing",
            Style::default().fg(Color::DarkGray),
        )])
    };

    let block_title = " Now Playing ";

    let player_widget = Paragraph::new(content)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .title(block_title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        );
    frame.render_widget(player_widget, area);
}

fn draw_filters(frame: &mut Frame, app: &App, area: Rect) {
    // Split filter area into label row + input row
    let filter_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    let fields = [
        ("Name", &app.draft_name, InputField::Name),
        ("Tags", &app.draft_tags, InputField::Tags),
        ("Country (ISO)", &app.draft_country, InputField::Country),
        ("Language (ISO)", &app.draft_language, InputField::Language),
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

fn draw_station_list(
    frame: &mut Frame,
    app: &App,
    table_state: &mut TableState,
    area: Rect,
) {
    let header_cells = ["Station Name", "Country", "Language", "Tags", "Bitrate"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(NEON_CYAN)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )
        });
    let header = Row::new(header_cells).height(1).bottom_margin(0);

    let rows: Vec<Row> = if app.loading {
        vec![Row::new(vec![Cell::from(Span::styled(
            "Loading stations...",
            Style::default().fg(Color::Yellow),
        ))])]
    } else if let Some(err) = &app.error {
        vec![Row::new(vec![Cell::from(Span::styled(
            format!("Error: {}", err),
            Style::default().fg(Color::Red),
        ))])]
    } else if app.stations.is_empty() {
        vec![Row::new(vec![Cell::from(Span::styled(
            "No stations found. Try different filters.",
            Style::default().fg(Color::DarkGray),
        ))])]
    } else {
        app.stations
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let is_playing = app
                    .current_station
                    .as_ref()
                    .map(|cs| cs.stationuuid == s.stationuuid)
                    .unwrap_or(false);

                let name_prefix = if is_playing { "▶ " } else { "  " };
                let name = format!("{}{}", name_prefix, truncate(&s.name, 35));
                let country = if s.country_code.is_empty() {
                    "N/A".to_string()
                } else {
                    s.country_code.clone()
                };
                let language = if s.language.is_empty() {
                    "N/A".to_string()
                } else {
                    truncate(&s.language, 12)
                };
                let tags = truncate(&s.tags, 30);
                let bitrate = if s.bitrate > 0 {
                    format!("{} kbps", s.bitrate)
                } else {
                    String::from("N/A")
                };

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

    let page_title = if app.total_pages > 0 {
        format!(" Stations — Page {}/{} ", app.page, app.total_pages)
    } else {
        " Stations ".to_string()
    };

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
                page_title,
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

    *table_state = TableState::default().with_selected(Some(app.selected));
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
            key("/", "Filter"),
            key("n/p", "Next/Prev Page"),
            key("+/-", "Volume"),
            key("s", "Stop"),
            key("q", "Quit"),
        ]
    };

    let mut spans: Vec<Span> = Vec::new();
    for (i, (k, desc)) in keys.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ", Style::default()));
        }
        spans.push(Span::styled(
            k.to_string(),
            Style::default()
                .fg(Color::Black)
                .bg(NEON_CYAN)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {}", desc),
            Style::default().fg(Color::Gray),
        ));
    }

    // Volume indicator
    spans.push(Span::styled("  │  ", Style::default().fg(Color::DarkGray)));
    spans.push(Span::styled(
        format!("Vol: {}%", app.volume_display()),
        Style::default().fg(NEON_CYAN),
    ));

    let footer = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(footer, area);

    // Render error/status overlay if needed
    if let Some(err) = &app.error {
        let popup_area = centered_rect(60, 20, frame.area());
        frame.render_widget(Clear, popup_area);
        let popup = Paragraph::new(err.as_str())
            .style(Style::default().fg(Color::Red))
            .block(
                Block::default()
                    .title(" Error ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red)),
            );
        frame.render_widget(popup, popup_area);
    }
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

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
