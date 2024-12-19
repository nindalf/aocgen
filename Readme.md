# Advent of Code Generator

`aocgen` is a flexible helper program that makes it easier to work with adventofcode.com.

1. `fetch` the day's problem.
   - Guess the day's test input from the problem text.
   - Fetch the day's full input specific to your account.
   - Store these in local files text files.
   - Create a scaffold according to the language specified - `day2.rs`, `day2.js` etc.
2. `submit` the answer

## One time setup 

Install `aocgen` :

```
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/nindalf/aocgen/releases/latest/download/aocgen-installer.sh | sh
```

Set env variable with the adventofcode.com session cookie. This allows `aocgen` to fetch each day's test input and submit the answer. 

Open adventofcode.com and check the Cookies tab in Developer Tools for the `session` cookie.

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

## Customising `fetch`

The default language chosen by `aocgen fetch` is Rust, but this can be changed with the `--language` option.

If you'd like to change the paths where the input, problem and solution files are stored, change the file in `configs`.

If you'd like to change the language template in `templates` it will need a recompile.
