use std::collections::HashMap;
use ncurses::*;
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

    pub fn search(&mut self, user_interface: &mut UserInterface) {
        user_interface.selected = 0;
        user_interface.page = 1;
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
            // handle regex searching here
        }
        user_interface.populate_screen(&self);
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

    pub fn toggle_case(&mut self) {
        self.case_sensitivity = (self.case_sensitivity + 1) % 2;
    }

    pub fn toggle_match(&mut self, user_interface: &mut UserInterface) {
        self.match_ = (self.match_ + 1) % 2;
        user_interface.selected = 0;
    }

    pub fn toggle_view(&mut self, user_interface: &mut UserInterface) {
        self.view = (self.view + 1) % 3;
        user_interface.selected = 0;
    }

    pub fn delete_from_history(&mut self, user_interface: &mut UserInterface, command: String) {
        user_interface.prompt_for_deletion(&command);
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
            110 => {}, // "n"
            _ => {}
        }
    }
}