use std::collections::HashSet;

use crate::{
    api::{SearchParams, Station},
    favorites::FavoriteEntry,
};

#[derive(Debug, Clone, PartialEq)]
pub enum InputField {
    Name,
    Country,
    Language,
    Bitrate,
    Tags,
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
    pub has_next_page: bool,
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
    pub draft_bitrate: String,
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
            has_next_page: false,
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
            draft_bitrate: String::new(),
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
        self.params.bitrate = self.draft_bitrate.trim().parse::<u32>().ok();
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
        self.has_next_page = count == self.params.limit;
    }

    pub fn set_favorite_stations(&mut self, stations: Vec<Station>) {
        self.favorite_stations = stations;
        self.favorites_loading = false;
        self.favorites_error = None;
        self.selected = 0;
        self.scroll_offset = 0;
    }

    pub fn set_error(&mut self, err: String) {
        self.error = Some(err);
        self.loading = false;
        self.favorites_loading = false;
    }

    pub fn set_favorites_error(&mut self, err: String) {
        self.favorites_error = Some(err);
        self.favorites_loading = false;
    }

    pub fn active_error(&self) -> Option<&str> {
        match self.view_mode {
            StationViewMode::AllStations => self.error.as_deref(),
            StationViewMode::Favorites => self.favorites_error.as_deref(),
        }
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

    pub fn next_page(&mut self) -> bool {
        if self.view_mode != StationViewMode::AllStations {
            return false;
        }
        if self.has_next_page {
            self.page += 1;
            self.params.offset = (self.page - 1) * self.params.limit;
            self.loading = true;
            return true;
        }

        false
    }

    pub fn prev_page(&mut self) -> bool {
        if self.view_mode != StationViewMode::AllStations {
            return false;
        }
        if self.page > 1 {
            self.page -= 1;
            self.params.offset = (self.page - 1) * self.params.limit;
            self.loading = true;
            return true;
        }

        false
    }

    pub fn active_field_mut(&mut self) -> Option<&mut String> {
        match &self.mode {
            AppMode::Filtering(InputField::Name) => Some(&mut self.draft_name),
            AppMode::Filtering(InputField::Country) => Some(&mut self.draft_country),
            AppMode::Filtering(InputField::Language) => Some(&mut self.draft_language),
            AppMode::Filtering(InputField::Bitrate) => Some(&mut self.draft_bitrate),
            AppMode::Filtering(InputField::Tags) => Some(&mut self.draft_tags),
            AppMode::Normal => None,
        }
    }

    pub fn next_field(&mut self) {
        self.mode = match &self.mode {
            AppMode::Filtering(InputField::Name) => AppMode::Filtering(InputField::Country),
            AppMode::Filtering(InputField::Country) => AppMode::Filtering(InputField::Language),
            AppMode::Filtering(InputField::Language) => AppMode::Filtering(InputField::Tags),
            AppMode::Filtering(InputField::Tags) => AppMode::Filtering(InputField::Bitrate),
            AppMode::Filtering(InputField::Bitrate) => AppMode::Filtering(InputField::Name),
            AppMode::Normal => AppMode::Normal,
        };
    }

    pub fn volume_display(&self) -> u8 {
        self.volume
    }

    pub fn stations_title(&self) -> String {
        match self.view_mode {
            StationViewMode::AllStations => {
                let suffix = if self.has_next_page {
                    " - more available"
                } else {
                    " - end reached"
                };
                format!(" Stations - Page {}{} ", self.page, suffix)
            }
            StationViewMode::Favorites => " Favorites ".to_string(),
        }
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

    #[test]
    fn removing_favorite_in_favorites_view_updates_selection_safely() {
        let mut app = App::new();
        let first = station("id-1", "One", "https://one");
        let second = station("id-2", "Two", "https://two");

        app.stations = vec![first.clone(), second.clone()];
        let _ = app.toggle_favorite_for_selected();
        app.selected = 1;
        let _ = app.toggle_favorite_for_selected();

        app.favorite_stations = vec![first, second];
        app.set_view_mode(StationViewMode::Favorites);
        app.selected = 1;

        let removed = app.toggle_favorite_for_selected();

        assert_eq!(removed, Some(false));
        assert_eq!(app.favorite_stations.len(), 1);
        assert_eq!(app.selected, 0);
        assert_eq!(app.favorite_stations[0].stationuuid, "id-1");
    }

    #[test]
    fn favorites_errors_stay_scoped_to_favorites_view() {
        let mut app = App::new();
        app.set_error("station search failed".to_string());
        app.set_view_mode(StationViewMode::Favorites);
        app.set_favorites_error("favorites refresh failed".to_string());

        assert_eq!(app.active_error(), Some("favorites refresh failed"));

        app.set_view_mode(StationViewMode::AllStations);
        assert_eq!(app.active_error(), Some("station search failed"));
    }

    #[test]
    fn stations_title_uses_has_next_page_instead_of_speculative_total() {
        let mut app = App::new();
        app.page = 3;
        app.has_next_page = true;
        assert_eq!(app.stations_title(), " Stations - Page 3 - more available ");

        app.has_next_page = false;
        assert_eq!(app.stations_title(), " Stations - Page 3 - end reached ");
    }

    #[test]
    fn next_page_only_advances_when_next_page_is_available() {
        let mut app = App::new();
        app.page = 2;
        app.params.limit = 50;
        app.params.offset = 50;
        app.has_next_page = false;

        assert!(!app.next_page());
        assert_eq!(app.page, 2);
        assert_eq!(app.params.offset, 50);
        assert!(!app.loading);

        app.has_next_page = true;

        assert!(app.next_page());
        assert_eq!(app.page, 3);
        assert_eq!(app.params.offset, 100);
        assert!(app.loading);
    }

    #[test]
    fn prev_page_only_moves_back_from_page_after_first() {
        let mut app = App::new();
        app.page = 1;
        app.params.limit = 50;
        app.params.offset = 0;

        assert!(!app.prev_page());
        assert_eq!(app.page, 1);
        assert_eq!(app.params.offset, 0);
        assert!(!app.loading);

        app.page = 3;
        app.params.offset = 100;

        assert!(app.prev_page());
        assert_eq!(app.page, 2);
        assert_eq!(app.params.offset, 50);
        assert!(app.loading);
    }
}
