use std::collections::HashMap;
use crate::util::{read, sort};
use crate::ui::UserInterface;

const FAVORITES: &str = "/home/alex/.config/venom/favorites";

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
        let history = read("/home/alex/.bash_history");
        let mut entries = HashMap::new();
        entries.insert(0, sort(&mut history.clone())); // sorted
        entries.insert(1, read(FAVORITES)); // favorites
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

        }
        user_interface.populate_screen(&self);
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
}