mod api;
mod app;
mod favorites;
mod player;
mod ui;

use std::{
    collections::HashSet,
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend, widgets::TableState};
use tokio::sync::mpsc;

use app::{App, AppMode, InputField, StationViewMode};
use favorites::FavoriteEntry;
use player::Player;

#[derive(Debug)]
enum AppEvent {
    StationsLoaded(Vec<api::Station>),
    LoadError(String),
    FavoritesLoaded(Vec<api::Station>, Vec<String>),
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), String> {
    let mut app = App::new();
    let mut player = Player::new();
    let mut table_state = TableState::default();

    match favorites::load_favorites() {
        Ok(entries) => app.set_favorites(entries),
        Err(err) => app.set_error(err),
    }

    let (tx, mut rx) = mpsc::unbounded_channel::<AppEvent>();
    let http_client = reqwest::Client::new();

    app.loading = true;
    trigger_load(&tx, &http_client, &app);

    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();

    loop {
        while let Ok(event) = rx.try_recv() {
            match event {
                AppEvent::StationsLoaded(stations) => {
                    app.set_stations(stations);
                }
                AppEvent::LoadError(err) => {
                    app.set_error(err);
                }
                AppEvent::FavoritesLoaded(mut stations, failed_uuids) => {
                    let mut seen: HashSet<String> =
                        stations.iter().map(|s| s.stationuuid.clone()).collect();
                    for fallback in fallback_stations_from_cached(&app.favorites, &failed_uuids) {
                        if seen.insert(fallback.stationuuid.clone()) {
                            stations.push(fallback);
                        }
                    }
                    stations.sort_by_cached_key(|s| s.name.to_lowercase());
                    app.set_favorite_stations(stations);
                    if !failed_uuids.is_empty() {
                        app.set_favorites_error(format!(
                            "Some favorites could not be refreshed from API ({}). Showing cached entries.",
                            failed_uuids.len()
                        ));
                    }
                }
            }
        }

        terminal
            .draw(|f| ui::draw(f, &app, &mut table_state))
            .map_err(|e| e.to_string())?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_default();

        if event::poll(timeout).map_err(|e| e.to_string())?
            && let Event::Key(key) = event::read().map_err(|e| e.to_string())?
        {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match &app.mode {
                AppMode::Normal => match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,
                    KeyCode::Down => {
                        let visible = terminal
                            .size()
                            .map(|s| s.height as usize)
                            .unwrap_or(20)
                            .saturating_sub(15);
                        app.select_next(visible.max(5));
                    }
                    KeyCode::Up => app.select_prev(),
                    KeyCode::Enter => {
                        if let Some(station) = app.selected_station().cloned() {
                            let url = if !station.url_resolved.is_empty() {
                                station.url_resolved.clone()
                            } else {
                                station.url.clone()
                            };
                            if let Some(err) = player.play(&url) {
                                app.error = Some(err);
                            } else {
                                app.current_station = Some(station);
                                app.error = None;
                            }
                        }
                    }
                    KeyCode::Char('s') => {
                        player.stop();
                        app.current_station = None;
                    }
                    KeyCode::Char('/') => {
                        app.mode = AppMode::Filtering(InputField::Name);
                    }
                    KeyCode::Char(' ') => {
                        if app.toggle_favorite_for_selected().is_some() {
                            if let Err(err) = favorites::save_favorites(&app.favorites) {
                                app.set_error(err);
                            } else {
                                app.error = None;
                            }
                        }
                    }
                    KeyCode::Char('f') => {
                        if app.view_mode == StationViewMode::AllStations {
                            app.set_view_mode(StationViewMode::Favorites);
                            app.favorites_error = None;
                            if app.favorites.is_empty() {
                                app.set_favorite_stations(Vec::new());
                            } else {
                                app.favorites_loading = true;
                                let uuids: Vec<String> = app
                                    .favorites
                                    .iter()
                                    .map(|f| f.stationuuid.clone())
                                    .collect();
                                trigger_load_favorites(&tx, &http_client, uuids);
                            }
                        } else {
                            app.set_view_mode(StationViewMode::AllStations);
                        }
                    }
                    KeyCode::Char('n') => {
                        if !app.loading && app.view_mode == StationViewMode::AllStations {
                            app.next_page();
                            trigger_load(&tx, &http_client, &app);
                        }
                    }
                    KeyCode::Char('p') => {
                        if !app.loading && app.view_mode == StationViewMode::AllStations {
                            app.prev_page();
                            trigger_load(&tx, &http_client, &app);
                        }
                    }
                    KeyCode::Char('+') => {
                        player.volume_up();
                        app.volume = player.volume;
                    }
                    KeyCode::Char('-') => {
                        player.volume_down();
                        app.volume = player.volume;
                    }
                    _ => {}
                },
                AppMode::Filtering(_) => match key.code {
                    KeyCode::Esc => {
                        app.mode = AppMode::Normal;
                    }
                    KeyCode::Tab => {
                        app.next_field();
                    }
                    KeyCode::Enter => {
                        app.update_params_from_drafts();
                        app.mode = AppMode::Normal;
                        app.loading = true;
                        app.set_view_mode(StationViewMode::AllStations);
                        trigger_load(&tx, &http_client, &app);
                    }
                    KeyCode::Backspace => {
                        if let Some(field) = app.active_field_mut() {
                            field.pop();
                        }
                    }
                    KeyCode::Char(c) => {
                        if let Some(field) = app.active_field_mut() {
                            field.push(c);
                        }
                    }
                    _ => {}
                },
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    Ok(())
}

fn fallback_stations_from_cached(
    favorites: &[FavoriteEntry],
    failed_uuids: &[String],
) -> Vec<api::Station> {
    let failed_set: HashSet<&str> = failed_uuids.iter().map(String::as_str).collect();

    favorites
        .iter()
        .filter(|fav| failed_set.contains(fav.stationuuid.as_str()))
        .map(|fav| api::Station {
            stationuuid: fav.stationuuid.clone(),
            name: fav.name.clone(),
            url: fav.url.clone(),
            url_resolved: String::new(),
            tags: String::new(),
            country_code: String::new(),
            language: String::new(),
            bitrate: 0,
        })
        .collect()
}

fn trigger_load(tx: &mpsc::UnboundedSender<AppEvent>, client: &reqwest::Client, app: &App) {
    let tx = tx.clone();
    let client = client.clone();
    let params = app.params.clone();
    tokio::spawn(async move {
        match api::search_stations(&client, &params).await {
            Ok(stations) => {
                let _ = tx.send(AppEvent::StationsLoaded(stations));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::LoadError(e));
            }
        }
    });
}

fn trigger_load_favorites(
    tx: &mpsc::UnboundedSender<AppEvent>,
    client: &reqwest::Client,
    uuids: Vec<String>,
) {
    let tx = tx.clone();
    let client = client.clone();
    tokio::spawn(async move {
        let (stations, failed_uuids) = api::fetch_stations_by_uuids(&client, uuids).await;
        let _ = tx.send(AppEvent::FavoritesLoaded(stations, failed_uuids));
    });
}
