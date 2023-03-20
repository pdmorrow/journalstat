# Journalstat.

Take as input a systemd journal file in binary format, or a directory containing
many journal files and produces tablular statistics on the journal contents. Supported
statistics:

  * Most frequently occurring messages.
  * Largest messages.

## Build

cargo build --release

## Run

```./target/debug/journalstat --help```

### Examples

On a directory containing many journals:

```
./target/release/journalstat --top-talkers 100 --input ~/toptalkers/exampleserver/journal/
```

On a single journal file

```
./target/release/journalstat --top-talkers 100 --input ./system@ad2cfc43460948acab23eb00bf503884-00000000002086ea-0005f75194ab51cb.journal
```
