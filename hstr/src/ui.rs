use crate::app::{Application, View};
use crate::util::get_shell_prompt;

#[cfg(test)]
use fake_ncurses as nc;
#[cfg(not(test))]
use ncurses as nc;

use regex::Regex;

const LABEL: &str = "Type to filter, UP/DOWN move, RET/TAB select, ESC quit, C-f add/rm fav";

pub struct UserInterface {
    pub page: i32,
    pub selected: i32,
}

impl UserInterface {
    pub fn new() -> Self {
        Self {
            page: 1,
            selected: 0,
        }
    }

    pub fn init_color_pairs(&self) {
        nc::start_color();
        nc::init_pair(1, nc::COLOR_WHITE, nc::COLOR_BLACK); // normal
        nc::init_pair(2, nc::COLOR_WHITE, nc::COLOR_GREEN); // highlighted-green (selected item)
        nc::init_pair(3, nc::COLOR_BLACK, nc::COLOR_WHITE); // highlighted-white (status)
        nc::init_pair(4, nc::COLOR_CYAN, nc::COLOR_BLACK); // white (favorites)
        nc::init_pair(5, nc::COLOR_RED, nc::COLOR_BLACK); // red (searched items)
        nc::init_pair(6, nc::COLOR_WHITE, nc::COLOR_RED); // higlighted-red
    }

    pub fn populate_screen(&self, app: &Application) {
        let commands = self.get_page(app.get_commands());
        for (index, entry) in commands.iter().enumerate() {
            nc::mvaddstr(
                index as i32 + 3,
                1,
                &format!("{1:0$}", nc::COLS() as usize - 1, entry),
            );
            let substring_indexes = self.get_substring_indexes(&entry, &app.search_string);
            if !substring_indexes.is_empty() {
                for (idx, letter) in entry.chars().enumerate() {
                    if substring_indexes.contains(&idx) {
                        nc::attron(nc::COLOR_PAIR(5) | nc::A_BOLD());
                        nc::mvaddch(index as i32 + 3, idx as i32 + 1, letter as nc::chtype);
                        nc::attroff(nc::COLOR_PAIR(5) | nc::A_BOLD());
                    } else {
                        nc::mvaddch(index as i32 + 3, idx as i32 + 1, letter as nc::chtype);
                    }
                }
            }
            if app
                .commands
                .as_ref()
                .unwrap()
                .get(&View::Favorites)
                .unwrap()
                .contains(&entry)
            {
                nc::attron(nc::COLOR_PAIR(4));
                nc::mvaddstr(
                    index as i32 + 3,
                    1,
                    &format!("{1:0$}", nc::COLS() as usize - 1, entry),
                );
                nc::attroff(nc::COLOR_PAIR(4));
            }
            if index == self.selected as usize {
                nc::attron(nc::COLOR_PAIR(2));
                nc::mvaddstr(
                    index as i32 + 3,
                    1,
                    &format!("{1:0$}", nc::COLS() as usize - 1, entry),
                );
                nc::attroff(nc::COLOR_PAIR(2));
            }
        }
        nc::mvaddstr(1, 1, LABEL);
        nc::attron(nc::COLOR_PAIR(3));
        nc::mvaddstr(
            2,
            1,
            &format!(
                "{1:0$}",
                nc::COLS() as usize - 1,
                format!(
                    "- view:{} (C-/) - regex:{} (C-e) - case:{} (C-t) - page {}/{} -",
                    self.display_view(app.view),
                    self.display_regex_mode(app.regex_mode),
                    self.display_case(app.case_sensitivity),
                    self.page,
                    self.total_pages(app.get_commands())
                )
            ),
        );
        nc::attroff(nc::COLOR_PAIR(3));
        nc::mvaddstr(
            0,
            1,
            &format!("{} {}", get_shell_prompt(), app.search_string),
        );
    }

    pub fn turn_page(&mut self, commands: &[String], direction: i32) {
        /* Turning the page essentially works as follows:
         *
         *  We are getting the potential page by subtracting 1
         *  from the page number, because pages are 1-based, and
         *  we need them to be 0-based for the calculation to work.
         *  Then we apply the direction which is always +1 or -1.
         *
         *  We then use the remainder part of Euclidean division of
         *  potential page over total number of pages, in order to
         *  wrap the page number around the total number of pages.
         *
         *  This means that if we are on page 4, and there are 4 pages in total,
         *  the command to go to the next page would result in rem(4, 4),
         *  which is 0, and by adjusting the page number to be 1-based,
         *  we get back to page 1, as desired.
         *
         *  This also works in the opposite direction:
         *
         *  If there are 4 total pages, and we are on page 1, and we issue
         *  the command to go to the previous page, we are doing: rem(-1, 4),
         *  which is 3. By adjusting the page number to be 1-based,
         *  we get to the 4th page.
         *
         *  The total number of pages being 0, which is the case when there
         *  are no commands in the history, means that we are dividing by 0,
         *  which is undefined, and rem() returns None, which means that we are
         *  on page 1.
         */
        nc::clear();
        let potential_page = self.page - 1 + direction;
        self.page = match i32::checked_rem_euclid(potential_page, self.total_pages(commands)) {
            Some(x) => x + 1,
            None => 1,
        }
    }

    pub fn move_selected(&mut self, commands: &[String], direction: i32) {
        let page_size = self.get_page_size(commands);
        self.selected += direction;
        if let Some(x) = i32::checked_rem_euclid(self.selected, page_size) {
            self.selected = x;
            if direction == 1 && self.selected == 0 {
                self.turn_page(commands, 1);
            } else if direction == -1 && self.selected == (page_size - 1) {
                self.turn_page(commands, -1);
                self.selected = self.get_page_size(commands) - 1;
            }
        }
    }

    pub fn get_selected(&self, commands: &[String]) -> String {
        String::from(
            self.get_page(&commands)
                .get(self.selected as usize)
                .unwrap(),
        )
    }

    fn total_pages(&self, commands: &[String]) -> i32 {
        commands.chunks(nc::LINES() as usize - 3).len() as i32
    }

    fn get_page(&self, commands: &[String]) -> Vec<String> {
        match commands
            .chunks(nc::LINES() as usize - 3)
            .nth(self.page as usize - 1)
        {
            Some(cmds) => cmds.to_vec(),
            None => Vec::new(),
        }
    }

    fn get_page_size(&self, commands: &[String]) -> i32 {
        self.get_page(commands).len() as i32
    }

    fn get_substring_indexes<'a>(&self, string: &'a str, substring: &'a str) -> Vec<usize> {
        match Regex::new(substring) {
            Ok(r) => r.find_iter(string).flat_map(|m| m.range()).collect(),
            Err(_) => vec![],
        }
    }

    fn display_view(&self, value: View) -> String {
        match value {
            View::Sorted => String::from("sorted"),
            View::Favorites => String::from("favorites"),
            View::All => String::from("all"),
        }
    }

    fn display_case(&self, value: bool) -> String {
        match value {
            true => String::from("sensitive"),
            false => String::from("insensitive"),
        }
    }

    fn display_regex_mode(&self, value: bool) -> String {
        match value {
            true => String::from("on"),
            false => String::from("off"),
        }
    }

    pub fn prompt_for_deletion(&self, command: &str) {
        nc::mvaddstr(1, 0, &format!("{1:0$}", nc::COLS() as usize, ""));
        nc::attron(nc::COLOR_PAIR(6));
        nc::mvaddstr(
            1,
            1,
            &format!("Do you want to delete all occurences of {}? y/n", command),
        );
        nc::attroff(nc::COLOR_PAIR(6));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::fixtures::*;
    use rstest::rstest;

    #[rstest(
        page,
        expected,
        case(1, vec![
            "cat spam",
            "cat SPAM",
            "git add .",
            "git add . --dry-run",
            "git push origin master",
            "git rebase -i HEAD~2",
            "git checkout -b tests",
        ]),
        case(2, vec![
            "grep -r spam .",
            "ping -c 10 www.google.com",
            "ls -la",
            "lsusb",
            "lspci",
            "sudo reboot",
            "source .venv/bin/activate",
        ]),
        case(3, vec![
            "deactivate",
            "pytest",
            "cargo test",
            "xfce4-panel -r",
            "nano .gitignore",
            "sudo dkms add .",
            "cd ~/Downloads",
        ]),
        case(4, vec![
            "make -j4",
            "gpg --card-status",
        ]),
        case(5, vec![])
    )]
    fn get_page(page: i32, expected: Vec<&str>, app_with_fake_history: Application) {
        let mut user_interface = UserInterface::new();
        let commands = app_with_fake_history.get_commands();
        user_interface.page = page;
        assert_eq!(user_interface.get_page(commands), expected);
    }

    #[rstest(
        current,
        expected,
        direction,
        case(1, 2, 1),
        case(2, 3, 1),
        case(3, 4, 1),
        case(4, 1, 1),
        case(4, 3, -1),
        case(3, 2, -1),
        case(2, 1, -1),
        case(1, 4, -1),
    )]
    fn turn_page(current: i32, expected: i32, direction: i32, app_with_fake_history: Application) {
        let mut user_interface = UserInterface::new();
        let commands = app_with_fake_history.get_commands();
        user_interface.page = current;
        user_interface.turn_page(commands, direction);
        assert_eq!(user_interface.page, expected)
    }

    #[rstest(
        string,
        substring,
        expected,
        case("cat spam", "cat", vec![0, 1, 2]),
        case("make -j4", "[0-9]+", vec![7]),
        case("ping -c 10 www.google.com", "[0-9]+", vec![8, 9])
    )]
    fn get_substring_indexes(string: &str, substring: &str, expected: Vec<usize>) {
        let user_interface = UserInterface::new();
        assert_eq!(
            user_interface.get_substring_indexes(string, substring),
            expected
        );
    }

    #[rstest()]
    fn get_page_size(app_with_fake_history: Application) {
        let user_interface = UserInterface::new();
        let commands = app_with_fake_history.get_commands();
        assert_eq!(user_interface.get_page_size(commands), 7);
    }

    #[rstest()]
    fn total_pages(app_with_fake_history: Application) {
        let user_interface = UserInterface::new();
        let commands = app_with_fake_history.get_commands();
        assert_eq!(user_interface.total_pages(commands), 4);
    }

    #[rstest(
        value,
        expected,
        case(View::Sorted, "sorted"),
        case(View::Favorites, "favorites"),
        case(View::All, "all")
    )]
    fn display_view(value: View, expected: &str) {
        let user_interface = UserInterface::new();
        assert_eq!(user_interface.display_view(value), expected.to_string());
    }

    #[rstest(value, expected, case(true, "sensitive"), case(false, "insensitive"))]
    fn display_case(value: bool, expected: &str) {
        let user_interface = UserInterface::new();
        assert_eq!(user_interface.display_case(value), expected.to_string());
    }

    #[rstest(value, expected, case(true, "on"), case(false, "off"))]
    fn display_regex_mode(value: bool, expected: &str) {
        let user_interface = UserInterface::new();
        assert_eq!(
            user_interface.display_regex_mode(value),
            expected.to_string()
        );
    }
}
