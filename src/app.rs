use ncurses::*;
use regex::{escape, Regex, RegexBuilder};
use crate::sort::sort;
use crate::util::{read_file, write_file};

const HISTORY: &str = ".bash_history";
const FAVORITES: &str = ".config/hstr-rs/favorites";
const Y: i32 = 121;

#[derive(Clone)]
pub struct Entries {
    all: Vec<String>,
    sorted: Vec<String>,
    favorites: Vec<String>
}

pub struct Application {
    all_entries: Entries,
    to_restore: Entries,
    view: u8,
    regex_match: bool,
    case_sensitivity: bool,
    search_string: String
}

impl Application {

    pub fn new(view: u8, regex_match: bool, case_sensitivity: bool, search_string: String) -> Self {
        let history = read_file(HISTORY).unwrap();
        let all_entries = Entries {
            all: history.clone(),
            sorted: sort(history.clone()),
            favorites: read_file(FAVORITES).unwrap()
        };
        Self { 
            all_entries: all_entries.clone(),
            to_restore: all_entries.clone(),
            view,
            regex_match,
            case_sensitivity,
            search_string
        }
    }

    pub fn set_all_entries(&mut self, val: Entries) {
        self.all_entries = val;
    }

    pub fn to_restore(&self) -> &Entries {
        &self.to_restore
    }

    pub fn view(&self) -> u8 {
        self.view
    }

    pub fn regex_match(&self) -> bool {
        self.regex_match
    }
    pub fn case_sensitivity(&self) -> bool {
        self.case_sensitivity
    }

    pub fn search_string_mut(&mut self) -> &mut String {
        &mut self.search_string
    }

    pub fn search_string(&self) -> &str {
        &self.search_string
    }

    pub fn get_entries_mut(&mut self, view: u8) -> &mut Vec<String> {
        match view {
            0 => &mut self.all_entries.sorted,
            1 => &mut self.all_entries.favorites,
            2 => &mut self.all_entries.all,
            _ => &mut self.all_entries.sorted
        }
    }

    pub fn get_entries(&self, view: u8) -> &[String] {
        match view {
            0 => &self.all_entries.sorted,
            1 => &self.all_entries.favorites,
            2 => &self.all_entries.all,
            _ => &self.all_entries.sorted
        }
    }

    fn create_search_string_regex(&self) -> Option<Regex> {
        match self.case_sensitivity {
            true => {
                match self.regex_match {
                    true => {
                        let regex = Regex::new(&self.search_string);
                        match regex {
                            Ok(r) => Some(r),
                            Err(_) => None
                        }
                    },
                    false => Some(Regex::new(&escape(&self.search_string)).unwrap())
                }
            },
            false => {
                match self.regex_match {
                    true => {
                        let regex = RegexBuilder::new(&self.search_string)
                            .case_insensitive(true)
                            .build();
                        match regex {
                            Ok(r) => Some(r),
                            Err(_) => None 
                        }
                    },
                    false => {
                        let regex = RegexBuilder::new(&escape(&self.search_string))
                            .case_insensitive(true)
                            .build()
                            .unwrap();
                        return Some(regex);
                    }
                }
            }
        }
    }

    pub fn search(&mut self) {
        let search_string_regex = match self.create_search_string_regex() {
            Some(r) => r,
            None => { return; }
        };
        self.get_entries_mut(self.view)
            .retain(|x| search_string_regex.is_match(x));
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
            Y => {
                let all_history = self.get_entries_mut(2);
                all_history.retain(|x| x != &command);
                write_file(HISTORY, &all_history);
            },
            _ => {}
        }
    }

    pub fn toggle_case(&mut self) {
        self.case_sensitivity = !self.case_sensitivity;
    }

    pub fn toggle_match(&mut self) {
        self.regex_match = !self.regex_match;
    }

    pub fn toggle_view(&mut self) {
        self.view = (self.view + 1) % 3;
    }
}