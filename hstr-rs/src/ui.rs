use crate::state::{SearchMode, State};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use pp::*;
use regex::Regex;

#[cfg(test)]
use fake_ncurses as nc;
#[cfg(not(test))]
use ncurses as nc;

use unicode_width::UnicodeWidthChar as _;

const LABEL: &str =
    "Type to filter, UP/DOWN move, ENTER/TAB select, DEL remove, ESC quit, C-f add/rm fav";

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

    fn page_size(&self, state: &State) -> i32 {
        self.page_contents(state).len() as i32
    }

    pub fn selected(&self, state: &State) -> Option<String> {
        self.page_contents(state)
            .get(self.selected as usize)
            .cloned()
    }

    fn page_contents(&self, state: &State) -> Vec<String> {
        let current_view = state.view;
        let commands = state.commands(current_view);
        match commands
            .chunks(nc::LINES() as usize - 3)
            .nth(self.page as usize - 1)
        {
            Some(cmds) => cmds.to_vec(),
            None => Vec::new(),
        }
    }

    pub fn populate_screen(&self, state: &State) {
        let matcher = SkimMatcherV2::default();
        self.page_contents(state)
            .iter()
            .enumerate()
            .for_each(|(row_idx, cmd)| {
                /* Print everything normally first; then
                 * Paint matched chars, if any; then
                 * Paint favorite, if any; then
                 * Finally, paint selection
                 */
                let cmd = &cmd
                    .chars()
                    .take(nc::COLS() as usize - 2)
                    .collect::<String>();
                nc::mvaddstr(row_idx as i32 + 3, 1, &ljust(cmd));
                match state.search_mode {
                    SearchMode::Exact | SearchMode::Regex => {
                        let matches = self.substring_indices(cmd, &state.query);
                        if !matches.is_empty() {
                            self.paint_matched_chars(cmd, matches, row_idx);
                        }
                    }
                    SearchMode::Fuzzy => {
                        if let Some(matches) = matcher.fuzzy_indices(cmd, &state.query) {
                            self.paint_matched_chars(cmd, matches.1, row_idx);
                        }
                    }
                }
                if state.cmd_in_fav(cmd) {
                    self.paint_favorite(cmd.clone(), row_idx);
                }
                self.paint_selected(cmd, row_idx);
            });
        self.paint_bars(state);
    }

    fn substring_indices<'b>(&self, string: &'b str, substring: &'b str) -> Vec<usize> {
        match Regex::new(substring) {
            Ok(r) => r.find_iter(string).flat_map(|m| m.range()).collect(),
            Err(_) => vec![],
        }
    }

    fn paint_matched_chars(&self, command: &str, indices: Vec<usize>, row_idx: usize) {
        column_indices(command).for_each(|(col_idx, byte_idx, ch)| {
            if indices.contains(&byte_idx) {
                nc::attron(nc::COLOR_PAIR(5) | nc::A_BOLD());
                nc::mvaddstr(row_idx as i32 + 3, col_idx as i32 + 1, &ch.to_string());
                nc::attroff(nc::COLOR_PAIR(5) | nc::A_BOLD());
            }
        });
    }

    fn paint_favorite(&self, entry: String, index: usize) {
        nc::attron(nc::COLOR_PAIR(4));
        nc::mvaddstr(index as i32 + 3, 1, &ljust(&entry));
        nc::attroff(nc::COLOR_PAIR(4));
    }

    fn paint_selected(&self, entry: &str, index: usize) {
        if index == self.selected as usize {
            nc::attron(nc::COLOR_PAIR(2));
            nc::mvaddstr(index as i32 + 3, 1, &ljust(&entry));
            nc::attroff(nc::COLOR_PAIR(2));
        }
    }

    fn paint_bars(&self, state: &State) {
        nc::mvaddstr(1, 1, LABEL);
        nc::attron(nc::COLOR_PAIR(3));
        nc::mvaddstr(2, 1, &ljust(&status_bar(&state, self)));
        nc::attroff(nc::COLOR_PAIR(3));
        nc::mvaddstr(0, 1, &top_bar(&state.query));
    }

    pub fn turn_page(&mut self, state: &State, direction: Direction) {
        /* Turning the page essentially works as follows:
         *
         * We are getting the potential page by subtracting 1
         * from the page number, because pages are 1-based, and
         * we need them to be 0-based for the calculation to work.
         * Then we stately the direction which is always +1 or -1.
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
        let next_page = self.page - 1 + direction as i32;
        let pages = self.total_pages(state);
        self.page = match i32::checked_rem_euclid(next_page, pages) {
            Some(x) => x + 1,
            None => 1,
        }
    }

    pub fn total_pages(&self, state: &State) -> i32 {
        let current_view = state.view;
        let commands = state.commands(current_view);
        commands.chunks(nc::LINES() as usize - 3).len() as i32
    }

    pub fn move_selected(&mut self, state: &State, direction: Direction) {
        let page_size = self.page_size(state);
        self.selected += direction as i32;
        if let Some(wraparound) = i32::checked_rem_euclid(self.selected, page_size) {
            self.selected = wraparound;
            match direction {
                Direction::Forward => {
                    if self.selected == 0 {
                        self.turn_page(state, Direction::Forward);
                    }
                }
                Direction::Backward => {
                    if self.selected == (page_size - 1) {
                        self.turn_page(state, Direction::Backward);
                        self.selected = self.page_size(state) - 1;
                    }
                }
            }
        }
    }

    pub fn retain_selected(&mut self, state: &State) {
        let page_size = self.page_size(state);
        if self.selected == page_size - 1 {
            self.selected -= 1;
        }
    }

    pub fn ask_before_deletion(&self, command: &str) {
        nc::mvaddstr(1, 0, &format!("{1:0$}", nc::COLS() as usize, ""));
        nc::attron(nc::COLOR_PAIR(6));
        nc::mvaddstr(1, 1, &deletion_prompt(command));
        nc::attroff(nc::COLOR_PAIR(6));
    }
}

struct ColumnIndices<'a> {
    inner: std::str::CharIndices<'a>,
    next_col: usize,
}

impl Iterator for ColumnIndices<'_> {
    type Item = (usize, usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        let col_idx = self.next_col;
        let (byte_idx, ch) = self.inner.next()?;

        self.next_col += ch.width().unwrap_or(0);

        Some((col_idx, byte_idx, ch))
    }
}

fn column_indices(s: &str) -> ColumnIndices {
    ColumnIndices {
        inner: s.char_indices(),
        next_col: 0,
    }
}

pub mod curses {
    use ncurses as nc;

    pub fn init() {
        nc::setlocale(nc::LcCategory::all, "");
        nc::initscr();
        nc::noecho();
        nc::keypad(nc::stdscr(), true);
        init_color_pairs();
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

mod pp {
    /* Pretty printer */
    use crate::state::{SearchMode, State, View};
    use crate::ui::UserInterface;
    use ncurses as nc;
    use std::env;
    use unicode_width::UnicodeWidthStr;

    pub fn status_bar(state: &State, user_interface: &UserInterface) -> String {
        let total_pages = user_interface.total_pages(state);
        format!(
            "- view:{} (C-/) - search:{} (C-e) - case:{} (C-t) - page {}/{} -",
            view(state.view),
            search_mode(state.search_mode),
            case(state.case_sensitivity),
            current_page(user_interface.page, total_pages),
            total_pages,
        )
    }

    pub fn top_bar(query: &str) -> String {
        format!("{} {}", get_shell_prompt(), query)
    }

    fn get_shell_prompt() -> String {
        format!(
            "{}@{}$",
            env::var("USER").unwrap(),
            gethostname::gethostname().into_string().unwrap()
        )
    }

    pub fn view(value: View) -> &'static str {
        match value {
            View::Sorted => "sorted",
            View::Favorites => "favorites",
            View::All => "all",
        }
    }

    pub fn search_mode(value: SearchMode) -> &'static str {
        match value {
            SearchMode::Exact => "exact",
            SearchMode::Regex => "regex",
            SearchMode::Fuzzy => "fuzzy",
        }
    }

    pub fn case(value: bool) -> &'static str {
        if value {
            "sensitive"
        } else {
            "insensitive"
        }
    }

    fn current_page(current_page: i32, total_pages: i32) -> i32 {
        match total_pages {
            0 => 0,
            _ => current_page,
        }
    }

    pub fn deletion_prompt(command: &str) -> String {
        format!("Do you want to delete all occurences of {}? y/n", command)
    }

    pub fn ljust(string: &str) -> String {
        let overhead = string.width() - string.chars().count();
        format!("{0:1$}", string, nc::COLS() as usize - 2 - overhead)
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum Direction {
    Forward = 1,
    Backward = -1,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{fixtures::*, SearchMode, View};
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
            "echo šampion",
            "nano .github/workflows/build.yml",
            "cd /home/bwk/",
        ]),
        case(5, vec![])
    )]
    fn get_page(page: i32, expected: Vec<&str>, fake_state: State) {
        let mut user_interface = UserInterface::new();
        user_interface.page = page;
        assert_eq!(user_interface.page_contents(&fake_state), expected);
    }

    #[rstest(
        current,
        expected,
        direction,
        case(1, 2, Direction::Forward),
        case(2, 3, Direction::Forward),
        case(3, 4, Direction::Forward),
        case(4, 1, Direction::Forward),
        case(4, 3, Direction::Backward),
        case(3, 2, Direction::Backward),
        case(2, 1, Direction::Backward),
        case(1, 4, Direction::Backward)
    )]
    fn turn_page(current: i32, expected: i32, direction: Direction, fake_state: State) {
        let mut user_interface = UserInterface::new();
        user_interface.page = current;
        user_interface.turn_page(&fake_state, direction);
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
    fn matched_chars_indices(string: &str, substring: &str, expected: Vec<usize>) {
        let user_interface = UserInterface::new();
        assert_eq!(
            user_interface.substring_indices(string, substring),
            expected
        );
    }

    #[rstest()]
    fn page_size(fake_state: State) {
        let user_interface = UserInterface::new();
        assert_eq!(user_interface.page_size(&fake_state), 7);
    }

    #[rstest()]
    fn total_pages(fake_state: State) {
        let user_interface = UserInterface::new();
        assert_eq!(user_interface.total_pages(&fake_state), 4);
    }

    #[rstest(
        value,
        expected,
        case(View::Sorted, "sorted"),
        case(View::Favorites, "favorites"),
        case(View::All, "all")
    )]
    fn format_view(value: View, expected: &str) {
        assert_eq!(super::pp::view(value), expected);
    }

    #[rstest(value, expected, case(true, "sensitive"), case(false, "insensitive"))]
    fn format_case(value: bool, expected: &str) {
        assert_eq!(super::pp::case(value), expected);
    }

    #[rstest(
        value,
        expected,
        case(SearchMode::Exact, "exact"),
        case(SearchMode::Regex, "regex"),
        case(SearchMode::Fuzzy, "fuzzy")
    )]
    fn format_search_mode(value: SearchMode, expected: &str) {
        assert_eq!(super::pp::search_mode(value), expected);
    }
}
