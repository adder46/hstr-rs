use ncurses::*;
use regex::Regex;
use crate::app::Application;
use crate::util::get_shell_prompt;

const LABEL: &str = "Type to filter, UP/DOWN move, RET/TAB select, DEL remove, ESC quit, C-f add/rm fav";

pub struct UserInterface {
    page: i32,
    selected: i32
}

impl UserInterface {

    pub fn new() -> Self {
        Self {
            page: 1,
            selected: 0
         }
    }

    pub fn set_page(&mut self, val: i32) {
        self.page = val;
    }

    pub fn set_selected(&mut self, val: i32) {
        self.selected = val;
    }

    pub fn init_color_pairs(&self) {
        start_color();
        init_pair(1, COLOR_WHITE, COLOR_BLACK); // normal
        init_pair(2, 15, COLOR_GREEN); // highlighted-green (selected item)
        init_pair(3, 0, 15); // highlighted-white (status)
        init_pair(4, 15, 0); // white (favorites)
        init_pair(5, COLOR_RED, 0); // red (searched items)
        init_pair(6, 15, COLOR_RED); // higlighted-red
    }

    pub fn populate_screen(&self, app: &Application) {
        clear();
        let entries = self.get_page(app.get_entries(app.view()));
        for (index, entry) in entries.iter().enumerate() {
            mvaddstr(index as i32 + 3, 0, &format!(" {1:0$}", COLS() as usize - 1, entry));
            let substring_indexes = self.get_substring_indexes(&entry, &app.search_string());
            if !substring_indexes.is_empty() {
                for (idx, letter) in entry.chars().enumerate() {
                    if substring_indexes.contains(&idx) {
                        attron(COLOR_PAIR(5) | A_BOLD());
                        mvaddch(index as i32 + 3, idx as i32 + 1, letter as chtype);
                        attroff(COLOR_PAIR(5) | A_BOLD());
                    } else {
                        mvaddch(index as i32 + 3, idx as i32 + 1, letter as chtype);
                    }
                }
            }
            if app.get_entries(1).contains(&entry) {
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
        let status = format!(" - view:{} (C-/) - match:{} (C-e) - case:{} (C-t) - page {}/{} -",
            self.display_view(app.view()),
            self.display_match(app.regex_match()),
            self.display_case(app.case_sensitivity()),
            self.page,
            self.total_pages(app.get_entries(app.view())) 
        );
        mvaddstr(1, 0, LABEL);
        attron(COLOR_PAIR(3));
        mvaddstr(2, 0, &format!("{1:0$}", COLS() as usize, status));
        attroff(COLOR_PAIR(3));
        mvaddstr(0, 0, &format!("{} {}", get_shell_prompt(), app.search_string()));
    }

    pub fn move_selected(&mut self, entries: &[String], direction: i32) {
        let page_size = self.get_page_size(entries);
        self.selected += direction;
        self.selected = i32::rem_euclid(self.selected, page_size);
        if direction == 1 {
            if self.selected == 0 {
                self.turn_page(entries, 1);
            }    
        } else if direction == -1 {
            if self.selected == (page_size - 1) {
                self.turn_page(entries, -1);
                self.selected = self.get_page_size(entries) - 1;
            }    
        }
    }

    pub fn get_selected(&self, entries: &[String]) -> String {
        String::from(self.get_page(&entries).get(self.selected as usize).unwrap())
    }

    fn display_view(&self, value: u8) -> &str {
        match value {
            0 => "sorted",
            1 => "favorites",
            2 => "history",
            _ => "n/a"
        }
    }

    fn display_case(&self, value: bool) -> &str {
        match value {
            false => "insensitive",
            true => "sensitive",
        }
    }

    fn display_match(&self, value: bool) -> &str {
        match value {
            false => "exact",
            true => "regex",
        }
    }

    fn get_substring_indexes<'a>(&self, string: &'a str, substring: &'a str) -> Vec<usize> {
        let regex = Regex::new(substring);
        let bla = match regex {
            Ok(r) => r,
            Err(_) => { return vec![]; }
        };
        let indexes = bla.find_iter(string)
            .flat_map(|mat| mat.range())
            .collect();
        indexes
    }

    fn turn_page(&mut self, entries: &[String], direction: i32) {
        self.page = i32::rem_euclid(self.page - 1 + direction, self.total_pages(entries)) + 1;
    }

    fn total_pages(&self, entries: &[String]) -> i32 {
        entries.chunks(LINES() as usize - 3).len() as i32
    }

    fn get_page(&self, entries: &[String]) -> Vec<String> {
        match entries.chunks(LINES() as usize - 3).nth(self.page as usize - 1) { 
            Some(val) => val.to_vec(),
            None => Vec::new()
        }
    }

    fn get_page_size(&self, entries: &[String]) -> i32 {
        self.get_page(entries).len() as i32
    }

    pub fn prompt_for_deletion(&self, command: &str) {
        let prompt = format!("Do you want to delete all occurences of {}? y/n", command);
        mvaddstr(1, 0, &format!("{1:0$}", COLS() as usize, ""));
        attron(COLOR_PAIR(6));
        mvaddstr(1, 0, &prompt);
        attroff(COLOR_PAIR(6));
    }
}
