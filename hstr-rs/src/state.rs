use std::collections::VecDeque;

use crate::{hstr, io, sort};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use itertools::Itertools;
use regex::{escape, Regex, RegexBuilder};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Clone)]
pub struct State {
    pub case_sensitivity: bool,
    pub search_mode: SearchMode,
    pub view: View,
    pub shell: String,
    pub query: String,
    pub raw_history: Vec<String>,
    pub commands: Commands,
    pub to_restore: Commands,
    pub cursor: usize,
}

impl State {
    pub fn new(query: String) -> Self {
        let shell = setenv::get_shell().get_name();
        let (raw_history, commands) = match shell {
            "bash" => hstr::get_bash_history(),
            "zsh" => hstr::get_zsh_history(),
            _ => panic!("{} is not supported yet.", shell),
        };
        Self {
            case_sensitivity: false,
            search_mode: SearchMode::Exact,
            view: View::Sorted,
            shell: shell.to_string(),
            query,
            raw_history,
            commands: commands.clone(),
            to_restore: commands,
            cursor: 0,
        }
    }

    pub fn commands(&self, view: View) -> &[String] {
        match view {
            View::Sorted => &self.commands.sorted,
            View::Favorites => &self.commands.favorites,
            View::All => &self.commands.all,
        }
    }

    pub fn commands_mut(&mut self, view: View) -> &mut Vec<String> {
        match view {
            View::Sorted => &mut self.commands.sorted,
            View::Favorites => &mut self.commands.favorites,
            View::All => &mut self.commands.all,
        }
    }

    pub fn search(&mut self) {
        match self.search_mode {
            SearchMode::Exact | SearchMode::Regex => {
                let search_regex = match self.create_search_regex() {
                    Some(r) => r,
                    None => {
                        return;
                    }
                };
                self.commands_mut(self.view)
                    .retain(|x| search_regex.is_match(x));
            }
            SearchMode::Fuzzy => {
                let query = self.query.clone();
                if self.case_sensitivity {
                    let matcher = SkimMatcherV2::default().respect_case();
                    self.commands_mut(self.view)
                        .retain(|x| matcher.fuzzy_match(x, query.as_str()).is_some());
                } else {
                    let matcher = SkimMatcherV2::default();
                    self.commands_mut(self.view)
                        .retain(|x| matcher.fuzzy_match(x, query.as_str()).is_some());
                }
            }
        }
    }

    fn create_search_regex(&self) -> Option<Regex> {
        let query = match self.search_mode {
            SearchMode::Regex => self.query.clone(),
            SearchMode::Exact => escape(&self.query),
            _ => unreachable!(),
        };
        RegexBuilder::new(&query)
            .case_insensitive(!self.case_sensitivity)
            .build()
            .ok()
    }

    pub fn add_or_rm_fav(&mut self, command: String) {
        let favorites = self.commands_mut(View::Favorites);
        if !favorites.contains(&command) {
            favorites.push(command);
        } else {
            favorites.retain(|x| *x != command);
        }
    }

    pub fn cmd_in_fav(&self, cmd: &str) -> bool {
        self.commands.favorites.contains(&cmd.to_string())
    }

    pub fn delete_from_history(&mut self, command: String) {
        View::iter().for_each(|view| {
            self.commands_mut(view).retain(|x| *x != command);
        });
        self.raw_history.retain(|x| *x != command);
    }

    pub fn reload_history(&mut self) {
        let commands = Commands {
            sorted: sort::sort(self.raw_history.clone()),
            favorites: io::read_from_home(&format!(".config/hstr-rs/.{}_favorites", self.shell))
                .unwrap(),
            all: self.raw_history.clone().into_iter().unique().collect(),
        };
        self.to_restore = commands;
        self.commands = self.to_restore.clone();
    }

    pub fn toggle_case(&mut self) {
        self.case_sensitivity = !self.case_sensitivity;
    }

    pub fn toggle_search_mode(&mut self) {
        self.search_mode = match (self.search_mode as u8 + 1) % 3 {
            0 => SearchMode::Exact,
            1 => SearchMode::Regex,
            2 => SearchMode::Fuzzy,
            _ => unreachable!(),
        }
    }

    pub fn toggle_view(&mut self) {
        self.view = match (self.view as u8 + 1) % 3 {
            0 => View::Sorted,
            1 => View::Favorites,
            2 => View::All,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
pub struct Commands {
    pub sorted: Vec<String>,
    pub favorites: Vec<String>,
    pub all: Vec<String>,
}

impl Commands {
    pub fn from_history(shell: &str, history: &[String]) -> Self {
        Self {
            sorted: sort::sort(history.to_vec()),
            favorites: io::read_from_home(format!(".config/hstr-rs/.{}_favorites", shell)).unwrap(),
            all: history.to_vec().into_iter().unique().collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, EnumIter, Eq, Hash, PartialEq)]
pub enum View {
    Sorted = 0,
    Favorites = 1,
    All = 2,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SearchMode {
    Exact = 0,
    Regex = 1,
    Fuzzy = 2,
}

#[cfg(test)]
pub mod fixtures {
    use super::*;
    use rstest::fixture;

    #[fixture]
    pub fn fake_history() -> Vec<String> {
        vec![
            "cat spam",
            "cat SPAM",
            "git add .",
            "git add . --dry-run",
            "git push origin master",
            "git rebase -i HEAD~2",
            "git checkout -b tests",
            "grep -r spam .",
            "ping -c 10 www.google.com",
            "ls -la",
            "lsusb",
            "lspci",
            "sudo reboot",
            "source .venv/bin/activate",
            "deactivate",
            "pytest",
            "cargo test",
            "xfce4-panel -r",
            "nano .gitignore",
            "sudo dkms add .",
            "cd ~/Downloads",
            "make -j4",
            "gpg --card-status",
            "echo šampion",
            "nano .github/workflows/build.yml",
            "cd /home/bwk/",
        ]
        .iter()
        .map(|&x| x.into())
        .collect()
    }

    #[fixture]
    pub fn fake_state(fake_history: Vec<String>) -> State {
        let mut state = State::new(String::new());
        let fake_commands = Commands {
            all: fake_history.clone(),
            favorites: Vec::new(),
            sorted: fake_history,
        };
        state.commands = fake_commands;
        state
    }
}

#[cfg(test)]
mod tests {
    use super::{fixtures::*, *};
    use rstest::rstest;

    #[rstest(
        query,
        expected,
        search_mode,
        case_sensitivity,
        case("cat", vec!["cat spam", "cat SPAM"], SearchMode::Exact, false),
        case("spam", vec!["cat spam", "cat SPAM", "grep -r spam ."], SearchMode::Exact, false),
        case("SPAM", vec!["cat SPAM"], SearchMode::Exact, true),
        case("[0-9]+", vec!["git rebase -i HEAD~2", "ping -c 10 www.google.com", "xfce4-panel -r", "make -j4"], SearchMode::Regex, false),
        case("šp", vec!["echo šampion"], SearchMode::Fuzzy, false),
        case("hwk", vec!["nano .github/workflows/build.yml", "cd /home/bwk/"], SearchMode::Fuzzy, false)
    )]
    fn search(
        query: &str,
        expected: Vec<&str>,
        search_mode: SearchMode,
        case_sensitivity: bool,
        mut fake_state: State,
    ) {
        fake_state.search_mode = search_mode;
        fake_state.case_sensitivity = case_sensitivity;
        fake_state.query = String::from(query);
        fake_state.search();
        assert_eq!(fake_state.commands(fake_state.view), expected);
    }

    #[rstest(
        view,
        expected,
        case(View::Sorted, fake_history()),
        case(View::Favorites, Vec::new()),
        case(View::All, fake_history())
    )]
    fn get_commands(view: View, expected: Vec<String>, mut fake_state: State) {
        fake_state.view = view;
        let commands = fake_state.commands(fake_state.view);
        assert_eq!(commands, expected);
    }

    #[rstest(
        query,
        search_mode,
        case_sensitivity,
        expected,
        case(String::from("print("), SearchMode::Exact, false, "print\\("),
        case(String::from("print("), SearchMode::Regex, false, ""),
        case(String::from("print("), SearchMode::Exact, true, "print\\("),
        case(String::from("print("), SearchMode::Regex, true, "")
    )]
    fn create_search_regex(
        query: String,
        search_mode: SearchMode,
        case_sensitivity: bool,
        expected: &str,
        fake_state: State,
    ) {
        let mut state = fake_state;
        state.query = query;
        state.search_mode = search_mode;
        state.case_sensitivity = case_sensitivity;
        let regex = state.create_search_regex();
        assert_eq!(regex.unwrap_or(Regex::new("").unwrap()).as_str(), expected);
    }

    #[rstest(
        command,
        case(String::from("cat spam")),
        case(String::from("grep -r spam .")),
        case(String::from("ping -c 10 www.google.com"))
    )]
    fn add_or_rm_fav(command: String, mut fake_state: State) {
        fake_state.add_or_rm_fav(command.clone());
        assert!(fake_state.commands(View::Favorites).contains(&command));
        fake_state.add_or_rm_fav(command.clone());
        assert!(!fake_state.commands(View::Favorites).contains(&command));
    }

    #[rstest(
        command,
        case(String::from("cat spam")),
        case(String::from("grep -r spam .")),
        case(String::from("ping -c 10 www.google.com"))
    )]
    fn delete_from_history(command: String, mut fake_state: State) {
        fake_state.delete_from_history(command.clone());
        assert!(!fake_state.commands(fake_state.view).contains(&command));
    }

    #[rstest(
        before,
        after,
        case(View::Sorted, View::Favorites),
        case(View::Favorites, View::All),
        case(View::All, View::Sorted)
    )]
    fn toggle_view(before: View, after: View) {
        let mut state = State::new(String::new());
        state.view = before;
        state.toggle_view();
        assert_eq!(state.view, after);
    }

    #[rstest(
        before,
        after,
        case(SearchMode::Exact, SearchMode::Regex),
        case(SearchMode::Regex, SearchMode::Fuzzy),
        case(SearchMode::Fuzzy, SearchMode::Exact)
    )]
    fn toggle_search_mode(before: SearchMode, after: SearchMode) {
        let mut state = State::new(String::new());
        state.search_mode = before;
        state.toggle_search_mode();
        assert_eq!(state.search_mode, after);
    }

    #[rstest(case_sensitivity, case(true), case(false))]
    fn toggle_case(case_sensitivity: bool) {
        let mut state = State::new(String::new());
        state.case_sensitivity = case_sensitivity;
        state.toggle_case();
        assert_eq!(state.case_sensitivity, !case_sensitivity);
    }
}
