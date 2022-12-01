# Advent of Code Generator

This program generates scaffolding for the day's solutions - `dayN.rs`.

It also fetches that day's input once your `AOC_COOKIE` is set.

## One time setup 

Install `aocgen` and add it to `PATH`

```
git clone https://github.com/nindalf/aocgen.git
cd aocgen
cargo install --path .
```

Set up a new project

```
cargo new advent-2022     # create new project
cd advent-2022/
cargo add anyhow          # improved error handling
```

Set env variable with the adventofcode.com session cookie. This allows `aocgen` to fetch each day's test input. This can be fetched from dev tools after logging in on the website.

```
export AOC_COOKIE=5361...
```
 
 ## Each day

To generate scaffolding and fetch the day's input

```
aocgen --day 1 --year 2022 # run each day
```
