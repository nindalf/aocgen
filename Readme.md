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
```

Set env variable with the adventofcode.com session cookie. This allows `aocgen` to fetch each day's test input. This can be fetched from dev tools after logging in on the website.

```
export AOC_COOKIE=5361...
```
 
 ## Each day

To generate each day's scaffolding and fetch that day's input

```
aocgen fetch --day 2 # run each day
```

And when you're ready with the answer

```
aocgen submit --day 2 --part 1 --answer 1024
```