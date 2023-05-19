# Journalstat.

Performance is not a goal!

Take as input a systemd journal file in binary format, or a directory containing
many journal files and produces tablular statistics on the journal contents.
Supported statistics:

  * Most frequently occurring messages.
  * Largest messages.

Filter by:

  * Systemd unit.
  * Regex.

## Build

cargo build --release

## Run

```
peter@p15v:~/git/journalstat$ ./target/release/journalstat --help
Journalstat 0.1.0
Command line options

USAGE:
    journalstat [OPTIONS] --input <input>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --input <input>                      Input journal file or directory
    -l, --large-messages <large-messages>    The number of large messages to report on
    -p, --pattern <pattern>                  Filter messages based on this regex pattern
    -t, --top-talkers <top-talkers>          The number of top talkers to report on
    -u, --unit <unit>                        Filter on a specific unit
peter@p15v:~/git/journalstat$
```

### Examples

On a directory containing many journals:

```
./target/release/journalstat --top-talkers 100 --input ~/toptalkers/exampleserver/journal/
```

On a single journal file

```
./target/release/journalstat --top-talkers 100 --input ./system@ad2cfc43460948acab23eb00bf503884-00000000002086ea-0005f75194ab51cb.journal
```
