# hstr-rs

![build status](https://github.com/adder46/hstr-rs/workflows/CI/badge.svg) [![codecov](https://codecov.io/gh/adder46/hstr-rs/branch/master/graph/badge.svg?token=0BZM100XU5)](https://codecov.io/gh/adder46/hstr-rs)

![screenshot](hstr-rs.gif)

**hstr-rs** is a shell history suggest box. Like hstr, but with pages. As opposed to original hstr which was the inspiration for this project, hstr-rs has pages and provides Unicode support out of the box on both bash and zsh.

There is an ongoing effort to support other shells too. Contributors are very welcome.
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
If on bash, run:

```
hstr-rs --show-config bash >> ~/.bashrc
```

...or manually add the lines below to your `.bashrc`:

```sh
# append new history items to .bash_history
shopt -s histappend
# don't put duplicate lines or lines starting with space in the history
HISTCONTROL=ignoreboth
# increase history file size
HISTFILESIZE=1000000
# increase history size
HISTSIZE=${HISTFILESIZE}
# sync entries in memory with .bash_history, and vice-versa
export PROMPT_COMMAND="history -a; history -n; ${PROMPT_COMMAND}"
# bind hstr-rs to CTRL + H
if [[ $- =~ .*i.* ]]; then bind '"\C-h": "hstr-rs \C-j"'; fi
```

For zsh, run:

```
hstr-rs --show-config zsh >> ~/.zshrc
```
...or manually add the lines below to your `.zshrc`:

```zsh
# append new history items to .bash_history
setopt INC_APPEND_HISTORY
# don't put duplicate lines
setopt HIST_IGNORE_ALL_DUPS
# don't put lines starting with space in the history
setopt HIST_IGNORE_SPACE
# increase history file size
HISTFILESIZE=1000000
# increase history size
HISTSIZE=${HISTFILESIZE}
# bind hstr-rs to CTRL + H
bindkey -s '^H' 'hstr-rs^M'
```

## Usage
​
The most convenient option if you're using bash is to make the alias below:

```sh
alias hh=hstr-rs
```

Then invoke the program with `hh`.

## Licensing

Licensed under the [MIT License](https://opensource.org/licenses/MIT). For details, see [LICENSE](https://github.com/adder46/hstr-rs/blob/master/LICENSE).
