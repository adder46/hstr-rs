use crate::{hstr, io, sort};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use itertools::Itertools;
use regex::{escape, Regex, RegexBuilder};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Clone)]
pub struct Application {
    pub case_sensitivity: bool,
    pub search_mode: Search,
    pub view: View,
    pub shell: String,
    pub search_string: String,
    pub raw_history: Vec<String>,
    pub commands: Commands,
    pub to_restore: Commands,
}

impl Application {
    pub fn new(search_string: String) -> Self {
        let shell = setenv::get_shell().get_name();
        let (raw_history, commands) = match shell {
            "bash" => hstr::get_bash_history(),
            "zsh" => hstr::get_zsh_history(),
            _ => unreachable!(),
        };
        Self {
            case_sensitivity: false,
            search_mode: Search::Exact,
            view: View::Sorted,
            shell: shell.to_string(),
            search_string,
            raw_history,
            commands: commands.clone(),
            to_restore: commands,
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
            Search::Exact | Search::Regex => {
                let search_regex = match self.create_search_regex() {
                    Some(r) => r,
                    None => {
                        return;
                    }
                };
                self.commands_mut(self.view)
                    .retain(|x| search_regex.is_match(x));
            }
            Search::Fuzzy => {
                let matcher = SkimMatcherV2::default();
                let search_string = self.search_string.clone();
                self.commands_mut(self.view)
                    .retain(|x| matcher.fuzzy_match(x, search_string.as_str()).is_some());
            }
        }
    }

    fn create_search_regex(&self) -> Option<Regex> {
        let search_string = match self.search_mode {
            Search::Regex => self.search_string.clone(),
            Search::Exact => escape(&self.search_string),
            _ => unreachable!(),
        };
        RegexBuilder::new(&search_string)
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
            0 => Search::Exact,
            1 => Search::Regex,
            2 => Search::Fuzzy,
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
pub enum Search {
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
    pub fn fake_app(fake_history: Vec<String>) -> Application {
        let mut app = Application::new(String::new());
        let fake_commands = Commands {
            all: fake_history.clone(),
            favorites: Vec::new(),
            sorted: fake_history,
        };
        app.commands = fake_commands;
        app
    }
}

#[cfg(test)]
mod tests {
    use super::{fixtures::*, *};
    use rstest::rstest;

    #[rstest(
        search_string,
        expected,
        search_mode,
        case_sensitivity,
        case("cat", vec!["cat spam", "cat SPAM"], Search::Exact, false),
        case("spam", vec!["cat spam", "cat SPAM", "grep -r spam ."], Search::Exact, false),
        case("SPAM", vec!["cat SPAM"], Search::Exact, true),
        case("[0-9]+", vec!["git rebase -i HEAD~2", "ping -c 10 www.google.com", "xfce4-panel -r", "make -j4"], Search::Regex, false),
        case("šp", vec!["echo šampion"], Search::Fuzzy, false),
        case("hwk", vec!["nano .github/workflows/build.yml", "cd /home/bwk/"], Search::Fuzzy, false)
    )]
    fn search(
        search_string: &str,
        expected: Vec<&str>,
        search_mode: Search,
        case_sensitivity: bool,
        mut fake_app: Application,
    ) {
        fake_app.search_mode = search_mode;
        fake_app.case_sensitivity = case_sensitivity;
        fake_app.search_string = String::from(search_string);
        fake_app.search();
        assert_eq!(fake_app.commands(fake_app.view), expected);
    }

    #[rstest(
        view,
        expected,
        case(View::Sorted, fake_history()),
        case(View::Favorites, Vec::new()),
        case(View::All, fake_history())
    )]
    fn get_commands(view: View, expected: Vec<String>, mut fake_app: Application) {
        fake_app.view = view;
        let commands = fake_app.commands(fake_app.view);
        assert_eq!(commands, expected);
    }

    #[rstest(
        search_string,
        search_mode,
        case_sensitivity,
        expected,
        case(String::from("print("), Search::Exact, false, "print\\("),
        case(String::from("print("), Search::Regex, false, ""),
        case(String::from("print("), Search::Exact, true, "print\\("),
        case(String::from("print("), Search::Regex, true, "")
    )]
    fn create_search_regex(
        search_string: String,
        search_mode: Search,
        case_sensitivity: bool,
        expected: &str,
        mut fake_app: Application,
    ) {
        fake_app.search_string = search_string;
        fake_app.search_mode = search_mode;
        fake_app.case_sensitivity = case_sensitivity;
        let regex = fake_app.create_search_regex();
        assert_eq!(regex.unwrap_or(Regex::new("").unwrap()).as_str(), expected);
    }

    #[rstest(
        command,
        case(String::from("cat spam")),
        case(String::from("grep -r spam .")),
        case(String::from("ping -c 10 www.google.com"))
    )]
    fn add_or_rm_fav(command: String, mut fake_app: Application) {
        fake_app.add_or_rm_fav(command.clone());
        assert!(fake_app.commands(View::Favorites).contains(&command));
        fake_app.add_or_rm_fav(command.clone());
        assert!(!fake_app.commands(View::Favorites).contains(&command));
    }

    #[rstest(
        command,
        case(String::from("cat spam")),
        case(String::from("grep -r spam .")),
        case(String::from("ping -c 10 www.google.com"))
    )]
    fn delete_from_history(command: String, mut fake_app: Application) {
        fake_app.delete_from_history(command.clone());
        assert!(!fake_app.commands(fake_app.view).contains(&command));
    }

    #[rstest(
        before,
        after,
        case(View::Sorted, View::Favorites),
        case(View::Favorites, View::All),
        case(View::All, View::Sorted)
    )]
    fn toggle_view(before: View, after: View) {
        let mut app = Application::new(String::new());
        app.view = before;
        app.toggle_view();
        assert_eq!(app.view, after);
    }

    #[rstest(
        before,
        after,
        case(Search::Exact, Search::Regex),
        case(Search::Regex, Search::Fuzzy),
        case(Search::Fuzzy, Search::Exact)
    )]
    fn toggle_search_mode(before: Search, after: Search) {
        let mut app = Application::new(String::new());
        app.search_mode = before;
        app.toggle_search_mode();
        assert_eq!(app.search_mode, after);
    }

    #[rstest(case_sensitivity, case(true), case(false))]
    fn toggle_case(case_sensitivity: bool) {
        let mut app = Application::new(String::new());
        app.case_sensitivity = case_sensitivity;
        app.toggle_case();
        assert_eq!(app.case_sensitivity, !case_sensitivity);
    }
}
