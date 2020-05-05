use std::collections::HashMap;
use ncurses::*;
use regex::Regex;
use crate::sort::sort;
use crate::util::{read_file, write_file};
use crate::ui::UserInterface;

const HISTORY: &str = ".bash_history";
const FAVORITES: &str = ".config/hstr-rs/favorites";

pub struct Application {
    pub all_entries: Option<HashMap<u8, Vec<String>>>,
    pub to_restore: Option<HashMap<u8, Vec<String>>>,
    pub view: u8,
    pub match_: u8,
    pub case_sensitivity: u8,
    pub search_string: String
}

impl Application {
    pub fn new() -> Self {
        Self { 
            all_entries: None,
            to_restore: None,
            view: 0,
            match_: 0,
            case_sensitivity: 0,
            search_string: String::new()
        }
    }

    pub fn load_data(&mut self) {
        let history = read_file(HISTORY);
        let mut entries = HashMap::new();
        entries.insert(0, sort(&mut history.clone())); // sorted
        entries.insert(1, read_file(FAVORITES)); // favorites
        entries.insert(2, history.clone()); // all history
        self.all_entries = Some(entries.clone());
        self.to_restore = Some(entries.clone());
    }

    pub fn search(&mut self) {
        if self.match_ == 0 {
            if self.case_sensitivity == 1 {
                let search_string = &self.search_string;
                self.all_entries
                    .as_mut()
                    .unwrap()
                    .get_mut(&self.view)
                    .unwrap()
                    .retain(|x| x.contains(search_string))
            } else {
                let search_string = &self.search_string.to_lowercase();
                self.all_entries
                    .as_mut()
                    .unwrap()
                    .get_mut(&self.view)
                    .unwrap()
                    .retain(|x| x.to_lowercase().contains(search_string));
            }
        } else {
            let re = Regex::new(&self.search_string).unwrap();
            self.all_entries
                .as_mut()
                .unwrap()
                .get_mut(&self.view)
                .unwrap()
                .retain(|x| re.is_match(x));
        }
    }

    pub fn add_to_or_remove_from_favorites(&mut self, command: String) {
        let favorites = self.all_entries
            .as_mut()
            .unwrap()
            .get_mut(&1)
            .unwrap();
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
                let all_history = self.all_entries
                    .as_mut()
                    .unwrap()
                    .get_mut(&2)
                    .unwrap();
                all_history.retain(|x| x != &command);
                write_file(HISTORY, &all_history);
                self.load_data();
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