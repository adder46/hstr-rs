# hstr-rs

![build status](https://github.com/adder46/hstr-rs/workflows/CI/badge.svg) [![codecov](https://codecov.io/gh/adder46/hstr-rs/branch/master/graph/badge.svg?token=0BZM100XU5)](https://codecov.io/gh/adder46/hstr-rs)

**hstr-rs** is a shell history suggest box. Like hstr, but with pages. As opposed to original hstr which was the inspiration for this project, hstr-rs does not use history files as a data source, rather, it expects its input to come from stdin. Also, hstr-rs does not require you to tweak your `$PROMPT_COMMAND`. This means that hstr-rs provides Unicode support out of the box, while avoiding potentially disastrous behavior at the same time.

hstr-rs is primarily designed to be used with bash, however, there is an ongoing effort to support other shells too, among which zsh is the priority. Contributors are very welcome.
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

## Screencast

![screenshot](hstr-rs.gif)


## Licensing

Licensed under the [MIT License](https://opensource.org/licenses/MIT). For details, see [LICENSE](https://github.com/adder46/hstr-rs/blob/master/LICENSE).
