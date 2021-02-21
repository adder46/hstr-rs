use crate::app::Application;
use crate::util::substring_indices;

#[cfg(test)]
use fake_ncurses as nc;
#[cfg(not(test))]
use ncurses as nc;

const LABEL: &str =
    "Type to filter, UP/DOWN move, ENTER/TAB select, DEL remove, ESC quit, C-f add/rm fav";


pub struct UserInterface {
    pub page: Page,
    pub selected: i32,
}

impl UserInterface {
    pub fn new() -> Self {
        Self {
            page: Page::new(),
            selected: 0,
        }
    }

    pub fn populate_screen(&self, app: &Application) {
        let commands = app.get_commands();
        let page_contents = self.page.contents(commands);
        page_contents.iter().enumerate().for_each(|(row_idx, cmd)| {
            /* Print everything first regularly */
            nc::mvaddstr(row_idx as i32 + 3, 1, &formatter::ljust(cmd));
            /* Paint matched chars, if any */
            let matches = substring_indices(cmd, &app.search_string);
            if !matches.is_empty() {
                self.paint_matched_chars(cmd, matches, row_idx);
            }
            /* Paint favorite, if any */
            if app.cmd_in_fav(cmd) {
                self.paint_favorite(cmd.clone(), row_idx);
            }
            /* Finally, paint selection */
            self.paint_selected(cmd, row_idx);
        });
        self.paint_bars(&app, &self);
    }

    fn paint_matched_chars(&self, command: &str, indices: Vec<usize>, row_idx: usize) {
        command.char_indices().for_each(|(char_idx, ch)| {
            if indices.contains(&char_idx) {
                nc::attron(nc::COLOR_PAIR(5) | nc::A_BOLD());
                nc::mvaddstr(row_idx as i32 + 3, char_idx as i32 + 1, &ch.to_string());
                nc::attroff(nc::COLOR_PAIR(5) | nc::A_BOLD());
            }
        });
    }

    fn paint_favorite(&self, entry: String, index: usize) {
        nc::attron(nc::COLOR_PAIR(4));
        nc::mvaddstr(index as i32 + 3, 1, &formatter::ljust(&entry));
        nc::attroff(nc::COLOR_PAIR(4));
    }

    fn paint_selected(&self, entry: &str, index: usize) {
        if index == self.selected as usize {
            nc::attron(nc::COLOR_PAIR(2));
            nc::mvaddstr(index as i32 + 3, 1, &formatter::ljust(&entry));
            nc::attroff(nc::COLOR_PAIR(2));
        }
    }

    fn paint_bars(&self, app: &Application, user_interface: &UserInterface) {
        nc::mvaddstr(1, 1, LABEL);
        nc::attron(nc::COLOR_PAIR(3));
        nc::mvaddstr(2, 1, &formatter::ljust(&formatter::status_bar(&app, user_interface)));
        nc::attroff(nc::COLOR_PAIR(3));
        nc::mvaddstr(0, 1, &formatter::top_bar(&app.search_string));
    }

    pub fn turn_page(&mut self, commands: &[String], direction: i32) {
        /* Turning the page essentially works as follows:
         *
         * We are getting the potential page by subtracting 1
         * from the page number, because pages are 1-based, and
         * we need them to be 0-based for the calculation to work.
         * Then we apply the direction which is always +1 or -1.
         *
         * We then use the remainder part of Euclidean division of
         * potential page over total number of pages, in order to
         * wrap the page number around the total number of pages.
         *
         * This means that if we are on page 4, and there are 4 pages in total,
         * the command to go to the next page would result in rem(4, 4),
         * which is 0, and by adjusting the page number to be 1-based,
         * we get back to page 1, as desired.
         *
         * This also works in the opposite direction:
         *
         * If there are 4 total pages, and we are on page 1, and we issue
         * the command to go to the previous page, we are doing: rem(-1, 4),
         * which is 3. By adjusting the page number to be 1-based,
         * we get to the 4th page.
         *
         * The total number of pages being 0, which is the case when there
         * are no commands in the history, means that we are dividing by 0,
         * which is undefined, and rem() returns None, which means that we are
         * on page 1.
         */
        nc::clear();
        let next = self.page.value - 1 + direction;
        let pages = self.total_pages(commands);
        self.page.value = match i32::checked_rem_euclid(next, pages) {
            Some(x) => x + 1,
            None => 1,
        }
    }

    fn total_pages(&self, commands: &[String]) -> i32 {
        commands.chunks(nc::LINES() as usize - 3).len() as i32
    }

    pub fn move_selected(&mut self, commands: &[String], direction: i32) {
        let page_size = self.page.size(commands);
        self.selected += direction;
        if let Some(wraparound) = i32::checked_rem_euclid(self.selected, page_size) {
            self.selected = wraparound;
            if direction == 1 && self.selected == 0 {
                self.turn_page(commands, 1);
            } else if direction == -1 && self.selected == (page_size - 1) {
                self.turn_page(commands, -1);
                self.selected = self.page.size(commands) - 1;
            }
        }
    }

    pub fn retain_selected(&mut self, commands: &[String]) {
        let page_size = self.page.size(commands);
        if self.selected == page_size - 1 {
            self.selected -= 1;
        }
    }

    pub fn ask_before_deletion(&self, command: &str) {
        nc::mvaddstr(1, 0, &format!("{1:0$}", nc::COLS() as usize, ""));
        nc::attron(nc::COLOR_PAIR(6));
        nc::mvaddstr(1, 1, &formatter::deletion_prompt(command));
        nc::attroff(nc::COLOR_PAIR(6));
    }
}

pub struct Page {
    pub value: i32,
}

impl Page {
    fn new() -> Self {
        Self { value: 1 }
    }

    pub fn size(&self, commands: &[String]) -> i32 {
        self.contents(commands).len() as i32
    }

    pub fn selected(&self, commands: &[String], index: i32) -> String {
        String::from(self.contents(&commands).get(index as usize).unwrap())
    }

    fn contents(&self, commands: &[String]) -> Vec<String> {
        match commands
            .chunks(nc::LINES() as usize - 3)
            .nth(self.value as usize - 1)
        {
            Some(cmds) => cmds.to_vec(),
            None => Vec::new(),
        }
    }
}

pub mod curses {
    use ncurses as nc;

    pub fn init() {
        nc::setlocale(nc::LcCategory::all, "");
        nc::initscr();
        nc::noecho();
        nc::keypad(nc::stdscr(), true);
    }

    pub fn init_color_pairs() {
        nc::start_color();
        nc::init_pair(1, nc::COLOR_WHITE, nc::COLOR_BLACK); // normal
        nc::init_pair(2, nc::COLOR_WHITE, nc::COLOR_GREEN); // highlighted-green (selected item)
        nc::init_pair(3, nc::COLOR_BLACK, nc::COLOR_WHITE); // highlighted-white (status)
        nc::init_pair(4, nc::COLOR_CYAN, nc::COLOR_BLACK); // white (favorites)
        nc::init_pair(5, nc::COLOR_RED, nc::COLOR_BLACK); // red (searched items)
        nc::init_pair(6, nc::COLOR_WHITE, nc::COLOR_RED); // higlighted-red
    }

    pub fn teardown() {
        nc::clear();
        nc::refresh();
        nc::doupdate();
        nc::endwin();
    }
}

mod formatter {
    use crate::app::{Application, View};
    use crate::ui::UserInterface;
    use crate::util::get_shell_prompt;
    use ncurses as nc;

    pub fn status_bar(app: &Application, user_interface: &UserInterface) -> String {
        format!(
            "- view:{} (C-/) - regex:{} (C-e) - case:{} (C-t) - page {}/{} -",
            view(app.view),
            regex_mode(app.regex_mode),
            case(app.case_sensitivity),
            pages(&app, &user_interface),
            user_interface.total_pages(app.get_commands())
        )
    }

    pub fn top_bar(search_string: &str) -> String {
        format!("{} {}", get_shell_prompt(), search_string)
    }

    pub fn view(value: View) -> String {
        match value {
            View::Sorted => String::from("sorted"),
            View::Favorites => String::from("favorites"),
            View::All => String::from("all"),
        }
    }

    pub fn regex_mode(value: bool) -> String {
        if value {
            String::from("on")
        } else {
            String::from("off")
        }
    }

    pub fn case(value: bool) -> String {
        if value {
            String::from("sensitive")
        } else {
            String::from("insensitive")
        }
    }

    fn pages(app: &Application, user_interface: &UserInterface) -> i32 {
        match user_interface.total_pages(app.get_commands()) {
            0 => 0,
            _ => user_interface.page.value,
        }
    }

    pub fn deletion_prompt(command: &str) -> String {
        format!("Do you want to delete all occurences of {}? y/n", command)
    }

    pub fn ljust(string: &str) -> String {
        format!("{0:1$}", string, nc::COLS() as usize - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{fixtures::*, View};
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
        user_interface.page.value = page;
        assert_eq!(user_interface.page.contents(commands), expected);
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
        user_interface.page.value = current;
        user_interface.turn_page(commands, direction);
        assert_eq!(user_interface.page.value, expected)
    }

    #[rstest(
        string,
        substring,
        expected,
        case("cat spam", "cat", vec![0, 1, 2]),
        case("make -j4", "[0-9]+", vec![7]),
        case("ping -c 10 www.google.com", "[0-9]+", vec![8, 9])
    )]
    fn matched_chars_indices(string: &str, substring: &str, expected: Vec<usize>) {
        assert_eq!(super::substring_indices(string, substring), expected);
    }

    #[rstest()]
    fn get_page_size(app_with_fake_history: Application) {
        let user_interface = UserInterface::new();
        let commands = app_with_fake_history.get_commands();
        assert_eq!(user_interface.page.size(commands), 7);
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
    fn format_view(value: View, expected: &str) {
        assert_eq!(super::formatter::view(value), expected.to_string());
    }

    #[rstest(value, expected, case(true, "sensitive"), case(false, "insensitive"))]
    fn format_case(value: bool, expected: &str) {
        assert_eq!(super::formatter::case(value), expected.to_string());
    }

    #[rstest(value, expected, case(true, "on"), case(false, "off"))]
    fn format_regex_mode(value: bool, expected: &str) {
        assert_eq!(super::formatter::regex_mode(value), expected.to_string());
    }
}
