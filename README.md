# hstr-rs

![build status](https://github.com/adder46/hstr-rs/workflows/CI/badge.svg) [![codecov](https://codecov.io/gh/adder46/hstr-rs/branch/master/graph/badge.svg?token=0BZM100XU5)](https://codecov.io/gh/adder46/hstr-rs)

**hstr** is shell history suggest box. Like hstr, but with pages.

It is primarily designed to be used with bash, however, it can be used with other shells, too, such as zsh (bear in mind a small issue mentioned in **Usage**).
hstr-rs has not been tested with other shells, such as fish, ksh, and tcsh.
​
## Installation
​
Make sure you have ncurses and readline packages installed.

If on Ubuntu:
​
```
sudo apt install libncurses5 libncurses5-dev libncursesw5 libncursesw5-dev libreadline5 libreadline-dev
```
​
Then run:
​
```
cargo install --git https://github.com/adder46/hstr-rs.git
```
​
If on bash, add this to .bashrc:

```bash
# append new history items to .bash_history
shopt -s histappend 
# don't put duplicate lines or lines starting with space in the history
HISTCONTROL=ignoreboth
# increase history file size
HISTFILESIZE=1000000
# increase history size
HISTSIZE=${HISTFILESIZE}
# append new entries from memory to .bash_history, and vice-versa
export PROMPT_COMMAND="history -a; history -n; ${PROMPT_COMMAND}"
```

## Usage
​
The most convenient option if you're using bash is to put the function below in your `~/.bashrc`:

```sh
hh() {
    history | hstr-rs
    history -r
}
```

Then invoke the program with `hh`.

When it comes to zsh, its `history` command significantly differs from the one that can be found on bash. zsh also lacks `$PROMPT_COMMAND`, which means that all major features of hstr-rs will work on zsh too, however, you might experience some unexpected behavior when deleting commands from history. In any case, if you want to use it with zsh, add this to `~/.zshenv`:

```sh
hh() {
    history 0 | hstr-rs
    fc -R
}
```
​
## Screencast

![screenshot](hstr-rs.gif)
