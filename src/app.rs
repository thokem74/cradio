use crate::api::{SearchParams, Station};

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

pub struct App {
    pub stations: Vec<Station>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub mode: AppMode,
    pub params: SearchParams,
    pub page: u32,
    pub total_pages: u32,
    pub loading: bool,
    pub error: Option<String>,
    pub current_station: Option<Station>,
    pub volume: u8,
    // Draft filter inputs (edited live)
    pub draft_name: String,
    pub draft_tags: String,
    pub draft_country: String,
    pub draft_language: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            stations: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            mode: AppMode::Normal,
            params: SearchParams::default(),
            page: 1,
            total_pages: 1,
            loading: false,
            error: None,
            current_station: None,
            volume: 50,
            draft_name: String::new(),
            draft_tags: String::new(),
            draft_country: String::new(),
            draft_language: String::new(),
        }
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
        // Each page is 50 stations; estimate pages based on whether we got a full page
        let count = stations.len() as u32;
        self.stations = stations;
        self.selected = 0;
        self.scroll_offset = 0;
        self.loading = false;
        self.error = None;
        // If we got a full page, assume there may be a next page;
        // otherwise this is the last page. (previous logic incorrectly
        // left `total_pages` unchanged when `page == total_pages`)
        if count == self.params.limit {
            self.total_pages = self.page + 1;
        } else {
            self.total_pages = self.page;
        }
    }

    pub fn set_error(&mut self, err: String) {
        self.error = Some(err);
        self.loading = false;
    }

    pub fn select_next(&mut self, visible_height: usize) {
        if self.stations.is_empty() {
            return;
        }
        if self.selected + 1 < self.stations.len() {
            self.selected += 1;
            if self.selected >= self.scroll_offset + visible_height {
                self.scroll_offset += 1;
            }
        }
    }

    pub fn select_prev(&mut self) {
        if self.stations.is_empty() {
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
        if self.page < self.total_pages {
            self.page += 1;
            self.params.offset = (self.page - 1) * self.params.limit;
            self.loading = true;
        }
    }

    pub fn prev_page(&mut self) {
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
