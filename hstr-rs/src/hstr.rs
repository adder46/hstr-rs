use crate::{app::Commands, io::read_from_home};

pub fn get_bash_history() -> (Vec<String>, Commands) {
    let history = read_from_home(".bash_history").unwrap();
    let commands = Commands::from_history("bash", &history);
    (history, commands)
}

pub fn get_zsh_history() -> (Vec<String>, Commands) {
    let history = zsh::process_history()
        .split('\n')
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    let commands = Commands::from_history("zsh", &history);
    (history, commands)
}

pub mod zsh {
    use crate::io;
    use regex::Regex;

    pub fn process_history() -> String {
        let history = io::read_as_bytes().unwrap();
        let unmetafied = unmetafy(history);
        remove_timestamps(String::from_utf8(unmetafied).unwrap())
    }

    fn unmetafy(mut bytestring: Vec<u8>) -> Vec<u8> {
        /* Unmetafying zsh history requires looping over the bytestring, removing
         * each encountered Meta character, and XOR-ing the following byte with 32.
         *
         * For instance:
         *
         * Input: ('a', 'b', 'c', Meta, 'd', 'e', 'f')
         * Wanted: ('a', 'b', 'c', 'd' ^ 32, 'e', 'f')
         */
        const ZSH_META: u8 = 0x83;
        for index in (0..bytestring.len()).rev() {
            if bytestring[index] == ZSH_META {
                bytestring.remove(index);
                bytestring[index] ^= 32;
            }
        }
        bytestring
    }

    fn remove_timestamps(history: String) -> String {
        /* The preceding metadata needs to be stripped
         * because zsh history entries look like below:
         *
         * `: 1330648651:0;sudo reboot`
         */
        let r = Regex::new(r"^: \d{10}:\d;").unwrap();
        history.lines().map(|x| r.replace(x, "") + "\n").collect()
    }
}
