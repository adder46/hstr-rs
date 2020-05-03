use ncurses::*;
use crate::app::Application;

const LABEL: &str = "Type to filter, UP/DOWN move, RET/TAB select, DEL remove, ESC quit, C-f add/rm fav";

pub struct UserInterface {
    pub page: i32,
    pub selected: i32
}

impl UserInterface {
    pub fn new() -> Self {
        Self {
            page: 1,
            selected: 0
         }
    }

    fn display(&self, to_get: &str, value: u8) -> &str {
        match to_get {
            "view" => {
                match value {
                    0 => "sorted",
                    1 => "favorites",
                    2 => "history",
                    _ => "n/a"
                }
            },
            "case" => {
                match value {
                    0 => "insensitive",
                    1 => "sensitive",
                    _ => "n/a"
                }
            },
            "match" => {
                match value {
                    0 => "exact",
                    1 => "regex",
                    _ => "n/a"
                }
            },
            _ => "n/a"
        }
    }

    pub fn init_color_pairs(&self) {
        start_color();
        init_pair(1, COLOR_WHITE, COLOR_BLACK); // normal
        init_pair(2, 15, COLOR_GREEN); // highlighted-green
        init_pair(3, 0, 15); // highlighted-white
        init_pair(4, 15, 0); // favorites
        init_pair(5, COLOR_RED, 0); // favorites
    }

    fn get_substring_indexes<'a>(&self, string: &'a str, substring: &'a str) -> Vec<(usize, &'a str)> {
        string.match_indices(substring).collect()
    }

    pub fn populate_screen(&self, app: &Application) {
        clear();
        let entries = self.get_page(
            &app.all_entries.as_ref().unwrap().get(&app.view).unwrap()
        );
        for (index, entry) in entries.iter().enumerate() {
            mvaddstr(index as i32 + 3, 0, &format!(" {1:0$}", COLS() as usize - 1, entry));
            let substring_indexes = self.get_substring_indexes(&entry, &app.search_string);
            if !substring_indexes.is_empty() {
                for (substring_index, _substring) in substring_indexes.iter() {
                    for i in 0..app.search_string.len() {
                        attron(COLOR_PAIR(5) | A_BOLD());
                        mvaddch(index as i32 + 3, (*substring_index + i + 1) as i32, app.search_string.chars().nth(i).unwrap() as u64);
                        attroff(COLOR_PAIR(5) | A_BOLD());    
                    }
                }
            }
            if app.all_entries.as_ref().unwrap().get(&1).unwrap().contains(&entry) {
                attron(COLOR_PAIR(4));
                mvaddstr(index as i32 + 3, 0, &format!(" {1:0$}", COLS() as usize - 1, entry));
                attroff(COLOR_PAIR(4));
            }
            if index == self.selected as usize {
                attron(COLOR_PAIR(2));
                mvaddstr(index as i32 + 3, 0, &format!(" {1:0$}", COLS() as usize - 1, entry));
                attroff(COLOR_PAIR(2));
            }
        }
        let status = format!(" - view:{} (C-/) - match: {} (C-e) - case:{} (C-t) - page {}/{} -",
            self.display("view", app.view),
            self.display("match", app.match_),
            self.display("case", app.case_sensitivity),
            self.page,
            self.total_pages(
                &app.all_entries.as_ref().unwrap().get(&app.view).unwrap()
            ) 
        );
        mvaddstr(1, 0, LABEL);
        attron(COLOR_PAIR(3));
        mvaddstr(2, 0, &format!("{1:0$}", COLS() as usize, status));
        attroff(COLOR_PAIR(3));
        mvaddstr(0, 0, &app.search_string);
    }

    pub fn move_selected(&mut self, all_entries: &Vec<String>, direction: i32) {
        let page_size = self.get_page_size(all_entries);
        self.selected += direction;
        self.selected = ((self.selected % page_size) + page_size) % page_size;
        if direction == 1 {
            if self.selected == 0 {
                self.turn_page(all_entries, 1);
            }    
        } else if direction == -1 {
            if self.selected == (page_size - 1) {
                self.turn_page(all_entries, -1);
                self.selected = self.get_page_size(all_entries) - 1;
            }    
        }
    }

    fn turn_page(&mut self, all_entries: &Vec<String>, direction: i32) {
        self.page = (
            (
                (
                    (self.page - 1 + direction) % self.total_pages(all_entries)
                ) + self.total_pages(all_entries)
            ) % self.total_pages(all_entries)
        ) + 1
    }

    fn total_pages(&self, all_entries: &Vec<String>) -> i32 {
        all_entries.chunks(LINES() as usize - 3).len() as i32
    }

    fn get_page(&self, all_entries: &Vec<String>) -> Vec<String> {
        let all_entries = match all_entries
            .chunks(LINES() as usize - 3)
            .nth(self.page as usize - 1) { 
                Some(val) => val.to_vec(),
                None => Vec::new()
            };
        all_entries
    }

    fn get_page_size(&self, all_entries: &Vec<String>) -> i32 {
        self.get_page(all_entries).len() as i32
    }
}
