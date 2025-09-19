use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;

use cfb::Stream;
use clap::{Parser, Subcommand};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[clap(author, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Concatenates and prints streams
    Cat { path: Vec<String> },

    /// Changes storage CLSIDs
    Chcls { clsid: Uuid, path: Vec<String> },

    /// Lists storage contents
    Ls {
        #[clap(short, long)]
        /// Lists in long format
        long: bool,

        #[clap(short, long)]
        /// Includes . in output
        all: bool,

        path: Vec<String>,
    },

    /// Creates a new stream with predefined values
    Create {
        /// Path to the compound file, e.g., /path/to/file.pvd
        #[clap(long)]
        file_path: String,
        /// Path to the storage inside the compound file, e.g., Inner/Storage
        #[clap(long)]
        inner_path: String,
        /// Name for the new stream
        #[clap(long)]
        stream_name: String,
    },
}

fn split(path: &str) -> (PathBuf, PathBuf) {
    let mut pieces = path.splitn(2, ':');
    if let Some(piece1) = pieces.next() {
        if let Some(piece2) = pieces.next() {
            (PathBuf::from(piece1), PathBuf::from(piece2))
        } else {
            (PathBuf::from(piece1), PathBuf::new())
        }
    } else {
        (PathBuf::new(), PathBuf::new())
    }
}

fn create_new_stream(
    comp: &mut cfb::CompoundFile<std::fs::File>,
    dir_entry: &cfb::Entry,
    stream_name: &str,
) -> io::Result<cfb::Stream<std::fs::File>> {
    if !dir_entry.is_storage() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "dir_entry is not a storage",
        ));
    }

    let mut full_path = dir_entry.path().to_path_buf();
    println!("Full path: {:?}", full_path);
    full_path.push(stream_name);

    // Create the stream (fails if it already exists)
    let stream = comp.create_stream(full_path)?;

    Ok(stream)
}

/// Writes a string, an i32, an f32, and an f64 into a newly created stream inside the given storage.
/// Binary layout (little endian):
/// [u32 string_length][string bytes][i32][f32][f64]
fn write_values(
    comp: &mut cfb::CompoundFile<std::fs::File>,
    dir_entry: &cfb::Entry,
    stream_name: &str,
    text: &str,
    int_val: i32,
    float_val: f32,
    double_val: f64,
) -> io::Result<()> {
    if !dir_entry.is_storage() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "dir_entry is not a storage",
        ));
    }

    let mut stream: Stream<File> =
        create_new_stream(comp, dir_entry, stream_name)?;

    let s_bytes = text.as_bytes();
    let len = s_bytes.len() as u32;

    stream.write_all(&len.to_le_bytes())?;
    stream.write_all(s_bytes)?;
    stream.write_all(&int_val.to_le_bytes())?;
    stream.write_all(&float_val.to_le_bytes())?;
    stream.write_all(&double_val.to_le_bytes())?;

    Ok(())
}

fn list_directory(
    name: &str,
    entry: &cfb::Entry,
    comp: &cfb::CompoundFile<std::fs::File>,
    long: bool,
    all: bool,
    indent: &str,
) {
    let new_indent = format!("{}  ", indent);
    println!("{}{}", indent, name);
    if entry.is_storage() {
        for subentry in comp.read_storage(entry.path()).unwrap() {
            list_directory(
                subentry.name(),
                &subentry,
                comp,
                long,
                all,
                &new_indent,
            );
        }
    }
}

fn list_entry(name: &str, entry: &cfb::Entry, long: bool) {
    if !long {
        println!("{}", entry.name());
        return;
    }
    let length = if entry.len() >= 10_000_000_000 {
        format!("{} GB", entry.len() / (1 << 30))
    } else if entry.len() >= 100_000_000 {
        format!("{} MB", entry.len() / (1 << 20))
    } else if entry.len() >= 1_000_000 {
        format!("{} kB", entry.len() / (1 << 10))
    } else {
        format!("{} B ", entry.len())
    };
    let last_modified = {
        let timestamp = entry.created().max(entry.modified());
        let datetime = OffsetDateTime::from(timestamp);
        let (year, month, day) = datetime.to_calendar_date();
        format!("{:04}-{:02}-{:02}", year, month as u8, day)
    };
    println!(
        "{}{:08x}   {:>10}   {}   {}",
        if entry.is_storage() { '+' } else { '-' },
        entry.state_bits(),
        length,
        last_modified,
        name
    );
    if entry.is_storage() {
        println!(" {}", entry.clsid().hyphenated());
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Create {
            file_path,
            inner_path,
            stream_name,
        } => {
            // let file = OpenOptions::new()
            //     .read(true)
            //     .write(true)
            //     .open(&file_path)
            //     .unwrap();
            let mut comp = cfb::open_rw(&file_path).unwrap();
            let dir_entry = comp.entry(&inner_path).unwrap();
            // let stream: cfb::Stream<File> = create_new_stream(&mut comp, &dir_entry, &stream_name).unwrap();
            write_values(
                &mut comp,
                &dir_entry,
                &stream_name,
                "Hello",
                123,
                45.67,
                89.1011,
            )
            .unwrap();
            comp.flush().unwrap();
            println!("Successfully created stream '{}' in '{}'", stream_name, file_path);
        }
        Command::Cat { path } => {
            for path in path {
                let (comp_path, inner_path) = split(&path);
                let mut comp = cfb::open(&comp_path).unwrap();
                let mut stream = comp.open_stream(inner_path).unwrap();
                io::copy(&mut stream, &mut io::stdout()).unwrap();
            }
        }
        Command::Chcls { clsid, path } => {
            for path in path {
                let (comp_path, inner_path) = split(&path);
                let mut comp = cfb::open(&comp_path).unwrap();
                comp.set_storage_clsid(inner_path, clsid).unwrap();
                comp.flush().unwrap();
            }
        }
        Command::Ls { long, all, path } => {
            for path in path {
                let (comp_path, inner_path) = split(&path);
                let comp = cfb::open(&comp_path).unwrap();
                let entry = comp.entry(&inner_path).unwrap();
                if entry.is_stream() {
                    list_entry(entry.name(), &entry, long);
                } else {
                    if all {
                        list_directory(
                            entry.name(),
                            &entry,
                            &comp,
                            long,
                            all,
                            "",
                        );
                    } else {
                        for subentry in
                            comp.read_storage(&inner_path).unwrap()
                        {
                            list_entry(subentry.name(), &subentry, long);
                        }
                    }
                }
            }
        }
    }
}
