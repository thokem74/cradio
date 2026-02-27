use std::collections::HashSet;

use crate::{
    api::{SearchParams, Station},
    favorites::FavoriteEntry,
};

#[derive(Debug, Clone, PartialEq)]
pub enum InputField {
    Name,
    Tags,
    Country,
    Language,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Filtering(InputField),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StationViewMode {
    AllStations,
    Favorites,
}

pub struct App {
    pub stations: Vec<Station>,
    pub favorite_stations: Vec<Station>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub mode: AppMode,
    pub view_mode: StationViewMode,
    pub params: SearchParams,
    pub page: u32,
    pub total_pages: u32,
    pub loading: bool,
    pub favorites_loading: bool,
    pub error: Option<String>,
    pub favorites_error: Option<String>,
    pub current_station: Option<Station>,
    pub volume: u8,
    pub favorite_ids: HashSet<String>,
    pub favorites: Vec<FavoriteEntry>,
    pub draft_name: String,
    pub draft_tags: String,
    pub draft_country: String,
    pub draft_language: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            stations: Vec::new(),
            favorite_stations: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            mode: AppMode::Normal,
            view_mode: StationViewMode::AllStations,
            params: SearchParams::default(),
            page: 1,
            total_pages: 1,
            loading: false,
            favorites_loading: false,
            error: None,
            favorites_error: None,
            current_station: None,
            volume: 50,
            favorite_ids: HashSet::new(),
            favorites: Vec::new(),
            draft_name: String::new(),
            draft_tags: String::new(),
            draft_country: String::new(),
            draft_language: String::new(),
        }
    }

    pub fn set_favorites(&mut self, favorites: Vec<FavoriteEntry>) {
        self.favorite_ids = favorites.iter().map(|f| f.stationuuid.clone()).collect();
        self.favorites = favorites;
    }

    pub fn update_params_from_drafts(&mut self) {
        self.params.name = self.draft_name.trim().to_string();
        self.params.tags = self.draft_tags.trim().to_string();
        self.params.country = self.draft_country.trim().to_uppercase();
        self.params.language = self.draft_language.trim().to_lowercase();
        self.page = 1;
        self.params.offset = 0;
    }

    pub fn set_stations(&mut self, stations: Vec<Station>) {
        let count = stations.len() as u32;
        self.stations = stations;
        self.selected = 0;
        self.scroll_offset = 0;
        self.loading = false;
        self.error = None;
        if count == self.params.limit {
            self.total_pages = self.page + 1;
        } else {
            self.total_pages = self.page;
        }
    }

    pub fn set_favorite_stations(&mut self, stations: Vec<Station>) {
        self.favorite_stations = stations;
        self.favorites_loading = false;
        self.favorites_error = None;
        self.error = None;
        self.selected = 0;
        self.scroll_offset = 0;
    }

    pub fn set_error(&mut self, err: String) {
        self.error = Some(err);
        self.loading = false;
        self.favorites_loading = false;
    }

    pub fn set_favorites_error(&mut self, err: String) {
        self.favorites_error = Some(err.clone());
        self.error = Some(err);
        self.favorites_loading = false;
    }

    pub fn current_station_list(&self) -> &[Station] {
        match self.view_mode {
            StationViewMode::AllStations => &self.stations,
            StationViewMode::Favorites => &self.favorite_stations,
        }
    }

    pub fn selected_station(&self) -> Option<&Station> {
        self.current_station_list().get(self.selected)
    }

    pub fn is_favorite(&self, stationuuid: &str) -> bool {
        self.favorite_ids.contains(stationuuid)
    }

    pub fn toggle_favorite_for_selected(&mut self) -> Option<bool> {
        let station = self.selected_station()?.clone();
        let now_favorite = if self.favorite_ids.contains(&station.stationuuid) {
            self.favorite_ids.remove(&station.stationuuid);
            self.favorites
                .retain(|fav| fav.stationuuid != station.stationuuid);
            false
        } else {
            self.favorite_ids.insert(station.stationuuid.clone());
            if let Some(existing) = self
                .favorites
                .iter_mut()
                .find(|fav| fav.stationuuid == station.stationuuid)
            {
                existing.name = station.name.clone();
                existing.url = station.url.clone();
            } else {
                self.favorites.push(FavoriteEntry {
                    stationuuid: station.stationuuid.clone(),
                    name: station.name.clone(),
                    url: station.url.clone(),
                });
            }
            true
        };

        if self.view_mode == StationViewMode::Favorites && !now_favorite {
            self.favorite_stations
                .retain(|s| s.stationuuid != station.stationuuid);
            if self.selected >= self.favorite_stations.len() {
                self.selected = self.favorite_stations.len().saturating_sub(1);
            }
            if self.scroll_offset > self.selected {
                self.scroll_offset = self.selected;
            }
        }

        Some(now_favorite)
    }

    pub fn set_view_mode(&mut self, mode: StationViewMode) {
        self.view_mode = mode;
        self.selected = 0;
        self.scroll_offset = 0;
    }

    pub fn select_next(&mut self, visible_height: usize) {
        let station_count = self.current_station_list().len();
        if station_count == 0 {
            return;
        }
        if self.selected + 1 < station_count {
            self.selected += 1;
            if self.selected >= self.scroll_offset + visible_height {
                self.scroll_offset += 1;
            }
        }
    }

    pub fn select_prev(&mut self) {
        if self.current_station_list().is_empty() {
            return;
        }
        if self.selected > 0 {
            self.selected -= 1;
            if self.selected < self.scroll_offset {
                self.scroll_offset = self.selected;
            }
        }
    }

    pub fn next_page(&mut self) {
        if self.view_mode != StationViewMode::AllStations {
            return;
        }
        if self.page < self.total_pages {
            self.page += 1;
            self.params.offset = (self.page - 1) * self.params.limit;
            self.loading = true;
        }
    }

    pub fn prev_page(&mut self) {
        if self.view_mode != StationViewMode::AllStations {
            return;
        }
        if self.page > 1 {
            self.page -= 1;
            self.params.offset = (self.page - 1) * self.params.limit;
            self.loading = true;
        }
    }

    pub fn active_field_mut(&mut self) -> Option<&mut String> {
        match &self.mode {
            AppMode::Filtering(InputField::Name) => Some(&mut self.draft_name),
            AppMode::Filtering(InputField::Tags) => Some(&mut self.draft_tags),
            AppMode::Filtering(InputField::Country) => Some(&mut self.draft_country),
            AppMode::Filtering(InputField::Language) => Some(&mut self.draft_language),
            AppMode::Normal => None,
        }
    }

    pub fn next_field(&mut self) {
        self.mode = match &self.mode {
            AppMode::Filtering(InputField::Name) => AppMode::Filtering(InputField::Tags),
            AppMode::Filtering(InputField::Tags) => AppMode::Filtering(InputField::Country),
            AppMode::Filtering(InputField::Country) => AppMode::Filtering(InputField::Language),
            AppMode::Filtering(InputField::Language) => AppMode::Filtering(InputField::Name),
            AppMode::Normal => AppMode::Normal,
        };
    }

    pub fn volume_display(&self) -> u8 {
        self.volume
    }
}

#[cfg(test)]
mod tests {
    use super::{App, StationViewMode};
    use crate::api::Station;

    fn station(uuid: &str, name: &str, url: &str) -> Station {
        Station {
            stationuuid: uuid.to_string(),
            name: name.to_string(),
            url: url.to_string(),
            url_resolved: "".to_string(),
            tags: "".to_string(),
            country_code: "".to_string(),
            language: "".to_string(),
            bitrate: 0,
        }
    }

    #[test]
    fn toggle_favorite_adds_and_removes_selected_station() {
        let mut app = App::new();
        app.stations = vec![station("id-1", "One", "https://one")];

        let added = app.toggle_favorite_for_selected();
        assert_eq!(added, Some(true));
        assert!(app.is_favorite("id-1"));
        assert_eq!(app.favorites.len(), 1);
        assert_eq!(app.favorites[0].name, "One");
        assert_eq!(app.favorites[0].url, "https://one");

        let removed = app.toggle_favorite_for_selected();
        assert_eq!(removed, Some(false));
        assert!(!app.is_favorite("id-1"));
        assert!(app.favorites.is_empty());
    }

    #[test]
    fn re_favorite_updates_stored_name_and_url() {
        let mut app = App::new();
        app.stations = vec![station("id-1", "Old", "https://old")];
        let _ = app.toggle_favorite_for_selected();
        let _ = app.toggle_favorite_for_selected();

        app.stations = vec![station("id-1", "New", "https://new")];
        let added = app.toggle_favorite_for_selected();

        assert_eq!(added, Some(true));
        assert_eq!(app.favorites.len(), 1);
        assert_eq!(app.favorites[0].name, "New");
        assert_eq!(app.favorites[0].url, "https://new");
    }

    #[test]
    fn set_view_mode_switches_between_all_and_favorites() {
        let mut app = App::new();
        app.set_view_mode(StationViewMode::Favorites);
        assert_eq!(app.view_mode, StationViewMode::Favorites);

        app.set_view_mode(StationViewMode::AllStations);
        assert_eq!(app.view_mode, StationViewMode::AllStations);
    }
}
