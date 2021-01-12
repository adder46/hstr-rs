# hstr-rs

![build status](https://github.com/adder46/hstr-rs/workflows/CI/badge.svg) [![codecov](https://codecov.io/gh/adder46/hstr-rs/branch/master/graph/badge.svg?token=0BZM100XU5)](https://codecov.io/gh/adder46/hstr-rs)

**hstr-rs** is a shell history suggest box. Like hstr, but with pages. As opposed to original hstr which was the inspiration for this project, hstr-rs does not use bash history file as a data source - rather, it expects its input to come from stdin. With zsh, hstr-rs has to use the history file as the source of truth, which implies doing weird stuff such as unmetafying the zsh history file. However, in contrast to the original hstr, this means that hstr-rs provides Unicode support out of the box on both bash and zsh, while avoiding potentially disastrous bash behavior by not reading from the history file and not advertising using `history -n` in `$PROMPT_COMMAND`.

hstr-rs was initially designed to be used with bash, but it works on zsh, too. There is an ongoing effort to support other shells too. Contributors are very welcome.
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
# append new entries from memory to .bash_history
export PROMPT_COMMAND="history -a; ${PROMPT_COMMAND}"
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
