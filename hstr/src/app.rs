use crate::sort::sort;
use crate::util::{read_file, zsh_process_history};
use itertools::Itertools;
use maplit::hashmap;
use regex::{escape, Regex, RegexBuilder};
use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub struct Application {
    pub case_sensitivity: bool,
    pub commands: Option<HashMap<View, Vec<String>>>,
    pub raw_history: Vec<String>,
    pub regex_mode: bool,
    pub search_string: String,
    pub shell: String,
    pub to_restore: Option<HashMap<View, Vec<String>>>,
    pub view: View,
}

impl Application {
    pub fn new(shell: &str) -> Self {
        Self {
            case_sensitivity: false,
            commands: None,
            raw_history: Vec::new(),
            regex_mode: false,
            search_string: String::new(),
            shell: shell.to_string(),
            to_restore: None,
            view: View::Sorted,
        }
    }

    pub fn load_history(&mut self) {
        match self.shell.as_str() {
            "bash" => self.load_bash_history(),
            "zsh" => self.load_zsh_history(),
            _ => {}
        }
    }

    fn load_bash_history(&mut self) {
        let history = read_file(".bash_history").unwrap();
        let commands = hashmap! {
            View::Sorted => sort(history.clone()),
            View::Favorites => read_file(".config/hstr-rs/.bash_favorites").unwrap(),
            View::All => history.clone().into_iter().unique().collect(),
        };
        self.raw_history = history;
        self.to_restore = Some(commands.clone());
        self.commands = Some(commands);
    }

    fn load_zsh_history(&mut self) {
        let history = zsh_process_history()
            .split('\n')
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        let commands = hashmap! {
            View::Sorted => sort(history.clone()),
            View::Favorites => read_file(".config/hstr-rs/.zsh_favorites").unwrap(),
            View::All => history.clone().into_iter().unique().collect(),
        };
        self.raw_history = history;
        self.to_restore = Some(commands.clone());
        self.commands = Some(commands);
    }

    pub fn reload_history(&mut self) {
        let commands = hashmap! {
            View::Sorted => sort(self.raw_history.clone()),
            View::Favorites => read_file(
                &format!(
                    ".config/hstr-rs/.{}_favorites",
                    self.shell
                )
            ).unwrap(),
            View::All => self.raw_history.clone().into_iter().unique().collect(),
        };
        self.to_restore = Some(commands);
        self.restore();
    }

    pub fn restore(&mut self) {
        self.commands = self.to_restore.clone();
    }

    pub fn get_commands(&self) -> &[String] {
        self.commands.as_ref().unwrap().get(&self.view).unwrap()
    }

    pub fn search(&mut self) {
        let search_regex = match self.create_search_regex() {
            Some(r) => r,
            None => {
                return;
            }
        };
        self.commands
            .as_mut()
            .unwrap()
            .get_mut(&self.view)
            .unwrap()
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
        let favorites = self
            .commands
            .as_mut()
            .unwrap()
            .get_mut(&View::Favorites)
            .unwrap();
        if !favorites.contains(&command) {
            favorites.push(command);
        } else {
            favorites.retain(|x| *x != command);
        }
    }

    pub fn cmd_in_fav(&self, cmd: &str) -> bool {
        self.commands
            .as_ref()
            .unwrap()
            .get(&View::Favorites)
            .unwrap()
            .contains(&cmd.to_string())
    }

    pub fn delete_from_history(&mut self, command: String) {
        View::iter().for_each(|view| {
            self.commands
                .as_mut()
                .unwrap()
                .get_mut(&view)
                .unwrap()
                .retain(|x| *x != command);
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
    pub fn app_with_fake_history(fake_history: Vec<String>) -> Application {
        let mut app = Application::new("bash");
        let fake_commands = hashmap! {
            View::All => fake_history.clone(),
            View::Favorites => Vec::new(),
            View::Sorted => fake_history,
        };
        app.commands = Some(fake_commands);
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
        mut app_with_fake_history: Application,
    ) {
        app_with_fake_history.regex_mode = regex_mode;
        app_with_fake_history.case_sensitivity = case_sensitivity;
        app_with_fake_history.search_string = String::from(search_string);
        app_with_fake_history.create_search_regex();
        app_with_fake_history.search();
        assert_eq!(app_with_fake_history.get_commands(), expected);
    }

    #[rstest(
        view,
        expected,
        case(View::Sorted, fake_history()),
        case(View::Favorites, Vec::new()),
        case(View::All, fake_history())
    )]
    fn get_commands(view: View, expected: Vec<String>, mut app_with_fake_history: Application) {
        app_with_fake_history.view = view;
        let commands = app_with_fake_history.get_commands();
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
        mut app_with_fake_history: Application,
    ) {
        app_with_fake_history.search_string = search_string;
        app_with_fake_history.regex_mode = regex_mode;
        app_with_fake_history.case_sensitivity = case_sensitivity;
        let regex = app_with_fake_history.create_search_regex();
        assert_eq!(regex.unwrap_or(Regex::new("").unwrap()).as_str(), expected);
    }

    #[rstest(
        command,
        case(String::from("cat spam")),
        case(String::from("grep -r spam .")),
        case(String::from("ping -c 10 www.google.com"))
    )]
    fn add_or_rm_fav(command: String, mut app_with_fake_history: Application) {
        app_with_fake_history.add_or_rm_fav(command.clone());
        assert!(app_with_fake_history
            .commands
            .as_ref()
            .unwrap()
            .get(&View::Favorites)
            .unwrap()
            .contains(&command));
        app_with_fake_history.add_or_rm_fav(command.clone());
        assert!(!app_with_fake_history
            .commands
            .unwrap()
            .get(&View::Favorites)
            .unwrap()
            .contains(&command));
    }

    #[rstest(
        command,
        case(String::from("cat spam")),
        case(String::from("grep -r spam .")),
        case(String::from("ping -c 10 www.google.com"))
    )]
    fn delete_from_history(command: String, mut app_with_fake_history: Application) {
        app_with_fake_history.delete_from_history(command.clone());
        assert!(!app_with_fake_history.get_commands().contains(&command));
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
