# hstr-rs

**hstr** is bash history suggest box. Like hstr, but with pages.
​
## Install hstr
​
Make sure you have ncurses packages installed.
​
On Ubuntu, run:
​
```bash
sudo apt install libncurses5 libncurses5-dev libncursesw5 libncursesw5-dev
```
​
Then run:
​
```bash
git clone https://github.com/adder46/hstr-rs.git
cd hstr-rs
cargo install --path .
```
​
## Usage
​
You can make an alias in your `.bashrc`:

```bash
alias hh=hstr-rs
```
​
![screenshot](hstr-rs.gif)
​
### FIXME:

- config script
