/// Crude tool to parse systemd journal files in binary
/// format in order to derive some statistics out of the
/// messages.
///
/// License: MIT
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use structopt::StructOpt;
use systemd::{
    journal::{OpenDirectoryOptions, OpenFilesOptions},
    Journal,
};
use tabled::{Table, Tabled};

#[derive(Debug, StructOpt)]
#[structopt(name = "Journalstat", about = "Command line options")]
struct Opt {
    /// Input journal file or directory.
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,

    /// The number of top talkers to report on.
    #[structopt(short, long)]
    top_talkers: Option<usize>,

    /// The number of large messages to report on.
    #[structopt(short, long)]
    large_messages: Option<usize>,

    /// Filter on a specific unit.
    #[structopt(short, long)]
    unit: Option<String>,
}

#[derive(Eq, PartialEq, Hash, Clone)]
struct Message {
    /// Message contents.
    msg: String,
    /// The process that generated the message.
    process: String,
}

struct JournalStat {
    // Input file/directory for debug purposes.
    input: PathBuf,
    // Filtering on a systemd unit.
    unit: Option<String>,
    // Handle to the journal.
    journal: Journal,
    // Map of messages in the journal to a frequency.
    msg_freq: HashMap<Message, u32>,
    // List of most frequent messages in the journal.
    top_talkers: Vec<(u32, Message)>,
    // The largest messages in the journal.
    largest: Vec<String>,
}

#[derive(Tabled)]
#[allow(non_snake_case)]
struct TopTalkerTableEntry<'a> {
    Rank: usize,
    Frequency: u32,
    Process: &'a str,
    Message: &'a str,
}

#[derive(Tabled)]
#[allow(non_snake_case)]
struct SizeTableEntry<'a> {
    Rank: usize,
    Size: usize,
    Message: &'a str,
}

impl JournalStat {
    /// Create a new JournalStat struct.
    pub fn new(path: &Path) -> Result<Self, systemd::Error> {
        let journal = if path.is_dir() {
            OpenDirectoryOptions::default()
                .open_directory(path.as_os_str().to_str().expect("failed to parse string"))
        } else {
            OpenFilesOptions::default().open_files(path.as_os_str().to_str())
        }?;

        Ok(Self {
            input: path.to_path_buf(),
            journal,
            unit: None,
            msg_freq: HashMap::new(),
            top_talkers: Vec::with_capacity(10),
            largest: Vec::with_capacity(10),
        })
    }

    /// Filter on a particular systemd unit.s
    pub fn set_filter_unit(&mut self, unit: &Option<String>) -> &mut Self {
        self.unit = unit.clone();
        self
    }

    /// Set the number of top talkers to watch for.
    pub fn n_frequent(&mut self, n_freq: usize) -> &mut Self {
        self.top_talkers = Vec::with_capacity(n_freq);
        self
    }

    /// Set the top number of large messages to record.
    pub fn n_largest(&mut self, n_largest: usize) -> &mut Self {
        self.largest = Vec::with_capacity(n_largest);
        self
    }

    /// Read the journal and record any statistics.
    pub fn parse(&mut self) -> &mut Self {
        while let Ok(Some(entry)) = self.journal.next_entry() {
            if let (Some(msg), Some(process_name)) = (entry.get("MESSAGE"), entry.get("_COMM")) {
                if let Some(unit) = &self.unit {
                    if let Some(junit) = entry.get("_SYSTEMD_UNIT") {
                        if !unit.eq(junit) {
                            continue;
                        }
                    }
                }

                let key = Message {
                    msg: msg.clone(),
                    process: process_name.clone(),
                };

                // No way around the to_string() which will hurt performance.
                let count = *self
                    .msg_freq
                    .entry(key.clone())
                    .and_modify(|c| *c += 1)
                    .or_insert(1);

                // Keep track of the top talkers...
                for i in 0..self.top_talkers.capacity() {
                    if let Some((ttcount, _)) = self.top_talkers.get(i) {
                        if count > *ttcount {
                            self.top_talkers[i] = (count, key);
                            break;
                        }
                    } else {
                        self.top_talkers.push((count, key));
                        break;
                    }
                }

                // Keep track of the big messages.
                for i in 0..self.largest.capacity() {
                    if let Some(lmsg) = self.largest.get(i) {
                        if msg.len() > lmsg.len() {
                            self.largest[i] = msg.clone();
                            break;
                        }
                    } else {
                        self.largest.push(msg.clone());
                    }
                }
            }
        }

        self
    }

    /// Generate a report.
    pub fn report(&self) {
        println!("Journal statistics for {}", self.input.display());

        if !self.top_talkers.is_empty() {
            println!("Top {} most frequent messages:", self.top_talkers.len());

            let mut table = Vec::new();

            for (i, (count, msg)) in self.top_talkers.iter().enumerate() {
                table.push(TopTalkerTableEntry {
                    Rank: i + 1,
                    Frequency: *count,
                    Process: &msg.process,
                    Message: &msg.msg,
                });
            }

            println!("{}", Table::new(table));
        }

        if !self.largest.is_empty() {
            println!("Top {} largest messages:", self.largest.len());

            let mut table = Vec::new();

            for (i, msg) in self.largest.iter().enumerate() {
                table.push(SizeTableEntry {
                    Rank: i + 1,
                    Size: msg.len(),
                    Message: msg,
                });
            }

            println!("{}", Table::new(table));
        }
    }
}

fn main() {
    let opt = Opt::from_args();

    JournalStat::new(&opt.input)
        .expect("failed to create new journal stat struct")
        .n_frequent(opt.top_talkers.unwrap_or(0))
        .n_largest(opt.large_messages.unwrap_or(0))
        .set_filter_unit(&opt.unit)
        .parse()
        .report();
}
