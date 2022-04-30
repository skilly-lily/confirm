# Confirm

A simple tool for scripted confirmation.  Returns zero if confirmed, non-zero otherwise.

```shell
$ confirm "Echo something?" -a 3 && echo something
Echo something? [y/n]: 
Echo something? [y/n]: 
Echo something? [y/n]: y
something
```

## Installation

Currently, must be installed using `cargo install confirm-rs`.  Platform support is determined by the `getch` crate.

You can also download the code and build from source.

## User guide

The up-to-date docs are maintained in the `--help` output of the tool, copied below.

`confirm --help` output:

```text
confirm-rs 1.1.0
A simple tool to obtain interactive confirmation in scripts.

USAGE:
    confirm [OPTIONS] [PROMPT]

ARGS:
    <PROMPT>
            The prompt to display
            
            Prompt of "Continue?" will become "Continue? [y/n]: ".  Options are added and
            highlighted based on given settings.  Original message will NEVER be modified.
            
            [default: Continue?]

OPTIONS:
    -a, --ask-count <ASK_COUNT>
            Number of times to ask
            
            Number of total times a question should be asked.  Use 0 for infinite retries.
            
            [default: 3]

    -d, --default <DEFAULT>
            Choose a default answer
            
            If no default is chosen, and the user supplies an empty answer, then a retry is
            triggered.  Otherwise, the default is used on an empty answer. If the retry count has
            been hit, then the process assumes a negative response and exits 1. Using the keyword
            "retry" is identical to omitting the option.
            
            [default: retry]

    -f, --full-words
            Require explicit "yes" or "no", not single letters.
            
            Cannot be used with --no-enter.

    -h, --help
            Print help information

        --no
            Don't ask any question, fail immediately.
            
            Turns the tool into no-op failure.  Useful when testing shell scripts built around this
            tool.

        --no-enter
            Don't require newlines
            
            Read the character on the terminal as it's typed, without waiting for the user to hit
            enter/return.

    -V, --version
            Print version information

        --yes
            Don't ask any question, return successfully.
            
            Turns the entire tool into a no-op, useful when building shell scripts around the tool.

```
