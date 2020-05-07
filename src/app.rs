use ncurses::*;
use regex::Regex;
use crate::sort::sort;
use crate::util::{read_file, write_file};

const HISTORY: &str = ".bash_history";
const FAVORITES: &str = ".config/hstr-rs/favorites";

#[derive(Clone)]
pub struct Entries {
    all: Vec<String>,
    sorted: Vec<String>,
    favorites: Vec<String>
}

pub struct Application {
    pub all_entries: Entries,
    pub to_restore: Entries,
    pub view: u8,
    pub match_: u8,
    pub case_sensitivity: u8,
    pub search_string: String
}

impl Application {
    pub fn new(view: u8, match_: u8, case_sensitivity: u8, search_string: String) -> Self {
        let history = read_file(HISTORY);
        let all_entries = Entries {
            all: history.clone(),
            sorted: sort(history.clone()),
            favorites: read_file(FAVORITES)
        };
        Self { 
            all_entries: all_entries.clone(),
            to_restore: all_entries.clone(),
            view: view,
            match_: match_,
            case_sensitivity: case_sensitivity,
            search_string: search_string
        }
    }

    pub fn get_entries_mut(&mut self, view: u8) -> &mut Vec<String> {
        match view {
            0 => &mut self.all_entries.sorted,
            1 => &mut self.all_entries.favorites,
            2 => &mut self.all_entries.all,
            _ => &mut self.all_entries.sorted
        }
    }

    pub fn get_entries(&self, view: u8) -> &Vec<String> {
        match view {
            0 => &self.all_entries.sorted,
            1 => &self.all_entries.favorites,
            2 => &self.all_entries.all,
            _ => &self.all_entries.sorted
        }
    }

    pub fn search(&mut self) {
        if self.match_ == 0 {
            if self.case_sensitivity == 1 {
                let search_string = self.search_string.clone();
                self.get_entries_mut(self.view).retain(|x| x.contains(&search_string))
            } else {
                let search_string = self.search_string.clone().to_lowercase();
                self.get_entries_mut(self.view).retain(|x| x.to_lowercase().contains(&search_string));
            }
        } else {
            let re = Regex::new(&self.search_string).unwrap();
            self.get_entries_mut(self.view).retain(|x| re.is_match(x));
        }
    }

    pub fn add_to_or_remove_from_favorites(&mut self, command: String) {
        let favorites = self.get_entries_mut(1);
        if !favorites.contains(&command) {
            favorites.push(command);
        } else {
            favorites.retain(|x| x != &command);
        }
        write_file(FAVORITES, &favorites);
    }

    pub fn delete_from_history(&mut self, command: String) {
        let answer = getch();
        match answer {
            121 => { // "y"
                let all_history = self.get_entries_mut(2);
                all_history.retain(|x| x != &command);
                write_file(HISTORY, &all_history);
            },
            _ => {}
        }
    }

    pub fn toggle_case(&mut self) {
        self.case_sensitivity = (self.case_sensitivity + 1) % 2;
    }

    pub fn toggle_match(&mut self) {
        self.match_ = (self.match_ + 1) % 2;
    }

    pub fn toggle_view(&mut self) {
        self.view = (self.view + 1) % 3;
    }
}