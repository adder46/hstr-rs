# hstr-rs

![build status](https://github.com/adder46/hstr-rs/workflows/CI/badge.svg) [![codecov](https://codecov.io/gh/adder46/hstr-rs/branch/master/graph/badge.svg?token=0BZM100XU5)](https://codecov.io/gh/adder46/hstr-rs)

**hstr-rs** is a shell history suggest box. Like hstr, but with pages. As opposed to original hstr which was the inspiration for this project, hstr-rs does not use history files as a data source, rather, it expects its input to come from stdin. This means that hstr-rs doesn't need to do weird stuff such as unmetafying zsh history files to provide proper Unicode support, which original hstr has failed to do up to this point. Also, hstr-rs does not require you to edit your `$PROMPT_COMMAND`. All this combined avoids potentially disastrous behavior.

hstr-rs is primarily designed to be used with bash, however, its crucial features work with other shells, too, such as zsh (bear in mind [issue #14](https://github.com/adder46/hstr-rs/issues/14)).
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
```

## Usage
​
The most convenient option if you're using bash is to put this alias in your `~/.bash_aliases`:

```sh
alias hh="hstr-rs < <(history)"
```

Then invoke the program with `hh`.

When it comes to zsh, its `history` command significantly differs from the one that can be found on bash. This means that all major features of hstr-rs will work on zsh too, however, you might experience some unexpected behavior when deleting commands from history. In any case, if you want to try it out with zsh, add the same alias to `~/.zshrc`.
​
## Screencast

![screenshot](hstr-rs.gif)
