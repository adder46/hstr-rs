use crate::sort::sort;
use crate::util::{self, read_file};
use itertools::Itertools;
use regex::{escape, Regex, RegexBuilder};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Clone)]
pub struct History {
    sorted: Vec<String>,
    favorites: Vec<String>,
    all: Vec<String>,
}

#[derive(Clone)]
pub struct Application {
    pub case_sensitivity: bool,
    pub commands: History,
    pub raw_history: Vec<String>,
    pub regex_mode: bool,
    pub search_string: String,
    pub shell: String,
    pub to_restore: History,
    pub view: View,
}

impl Application {
    pub fn new(shell: &str) -> Self {
        let (raw_history, commands) = match shell {
            "bash" => {
                let history = read_file(".bash_history").unwrap();
                let commands = History {
                    sorted: sort(history.clone()),
                    favorites: read_file(".config/hstr-rs/.bash_favorites").unwrap(),
                    all: history.clone().into_iter().unique().collect(),
                };
                (history, commands)
            }
            "zsh" => {
                let history = util::zsh_history::process()
                    .split('\n')
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>();
                let commands = History {
                    sorted: sort(history.clone()),
                    favorites: read_file(".config/hstr-rs/.zsh_favorites").unwrap(),
                    all: history.clone().into_iter().unique().collect(),
                };
                (history, commands)
            }
            _ => unreachable!(),
        };
        Self {
            case_sensitivity: false,
            commands: commands.clone(),
            raw_history,
            regex_mode: false,
            search_string: String::new(),
            shell: shell.to_string(),
            to_restore: commands,
            view: View::Sorted,
        }
    }

    pub fn reload_history(&mut self) {
        let commands = History {
            sorted: sort(self.raw_history.clone()),
            favorites: read_file(&format!(".config/hstr-rs/.{}_favorites", self.shell)).unwrap(),
            all: self.raw_history.clone().into_iter().unique().collect(),
        };
        self.to_restore = commands;
        self.restore();
    }

    pub fn restore(&mut self) {
        self.commands = self.to_restore.clone();
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
        let search_regex = match self.create_search_regex() {
            Some(r) => r,
            None => {
                return;
            }
        };
        self.commands_mut(self.view)
            .retain(|x| search_regex.is_match(x));
    }

    fn create_search_regex(&self) -> Option<Regex> {
        let search_string = if self.regex_mode {
            self.search_string.clone()
        } else {
            escape(&self.search_string)
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

    pub fn toggle_case(&mut self) {
        self.case_sensitivity = !self.case_sensitivity;
    }

    pub fn toggle_regex_mode(&mut self) {
        self.regex_mode = !self.regex_mode;
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

#[derive(Clone, Copy, Debug, EnumIter, Eq, Hash, PartialEq)]
pub enum View {
    Sorted = 0,
    Favorites = 1,
    All = 2,
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
        ]
        .iter()
        .map(|&x| x.into())
        .collect()
    }

    #[fixture]
    pub fn fake_app(fake_history: Vec<String>) -> Application {
        let mut app = Application::new("bash");
        let fake_commands = History {
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
        regex_mode,
        case_sensitivity,
        case("cat", vec!["cat spam", "cat SPAM"], false, false),
        case("spam", vec!["cat spam", "cat SPAM", "grep -r spam ."], false, false),
        case("SPAM", vec!["cat SPAM"], false, true),
        case("[0-9]+", vec!["git rebase -i HEAD~2", "ping -c 10 www.google.com", "xfce4-panel -r", "make -j4"], true, false)
    )]
    fn search(
        search_string: &str,
        expected: Vec<&str>,
        regex_mode: bool,
        case_sensitivity: bool,
        mut fake_app: Application,
    ) {
        fake_app.regex_mode = regex_mode;
        fake_app.case_sensitivity = case_sensitivity;
        fake_app.search_string = String::from(search_string);
        fake_app.create_search_regex();
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
        regex_mode,
        case_sensitivity,
        expected,
        case(String::from("print("), false, false, "print\\("),
        case(String::from("print("), true, false, ""),
        case(String::from("print("), false, true, "print\\("),
        case(String::from("print("), true, true, "")
    )]
    fn create_search_regex(
        search_string: String,
        regex_mode: bool,
        case_sensitivity: bool,
        expected: &str,
        mut fake_app: Application,
    ) {
        fake_app.search_string = search_string;
        fake_app.regex_mode = regex_mode;
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
        let mut app = Application::new("bash");
        app.view = before;
        app.toggle_view();
        assert_eq!(app.view, after);
    }

    #[rstest(regex_mode, case(true), case(false))]
    fn toggle_regex_mode(regex_mode: bool) {
        let mut app = Application::new("bash");
        app.regex_mode = regex_mode;
        app.toggle_regex_mode();
        assert_eq!(app.regex_mode, !regex_mode);
    }

    #[rstest(case_sensitivity, case(true), case(false))]
    fn toggle_case(case_sensitivity: bool) {
        let mut app = Application::new("bash");
        app.case_sensitivity = case_sensitivity;
        app.toggle_case();
        assert_eq!(app.case_sensitivity, !case_sensitivity);
    }
}
