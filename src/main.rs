use clap::Parser;
use regex::Regex;
use std::{
    env,
    error::Error,
    fs::metadata,
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct BulkCp {
    /// Move instead of copy. Also moving if first argument contains mv
    #[arg(long)]
    mv: bool,

    /// Print out the intended copies without modifying the filesystem
    #[arg(long)]
    dry_run: bool,

    /// Don't print to standard out
    #[arg(short, long, conflicts_with("dry_run"))]
    silent: bool,

    /// Don't require pattern to patch the whole filename (non-anchored)
    #[arg(short)]
    floating: bool,

    /// Recurse when encountering a directory
    #[arg(short, long)]
    recursive: bool,

    /// Regex to match against. Only files in the current directory are tested,
    /// and are matched only on their filename, without the preceding `./`.
    /// There are implicit anchors surrounding the pattern if the `-f`
    /// flag is not included.
    pattern: String,

    /// The destination to copy the files to. This is a pattern which will
    /// substitute `%[0-9]` with that numbered capture group. Use `%%` to
    /// insert a single percent symbol.
    #[arg(value_parser = parse_destination)]
    destination: &'static DestinationPattern,
}

type DestinationPattern = [DestinationPatternPart];
enum DestinationPatternPart {
    String(String),
    Substitution(usize),
}

fn parse_destination<'a>(s: &str) -> Result<&'a DestinationPattern, Box<dyn Error + Send + Sync>> {
    let mut destination = vec![DestinationPatternPart::String(String::new())];
    let mut parse_state = 0;
    for c in s.chars() {
        match (c, parse_state) {
            ('%', 1) => {
                let last = destination.last_mut().unwrap();
                if let DestinationPatternPart::String(last) = last {
                    last.push('%');
                }
                parse_state = 0;
            }
            ('0'..='9', 1) => {
                destination.push(DestinationPatternPart::Substitution(
                    c.to_digit(10).unwrap() as usize,
                ));
                destination.push(DestinationPatternPart::String(String::new()));
                parse_state = 0;
            }
            (c, 1) => {
                let last = destination.last_mut().unwrap();
                if let DestinationPatternPart::String(last) = last {
                    last.push('%');
                    last.push(c);
                    parse_state = 0;
                }
            }
            ('%', 0) => parse_state = 1,
            (c, 0) => {
                let last = destination.last_mut().unwrap();
                if let DestinationPatternPart::String(last) = last {
                    last.push(c);
                }
            }
            _ => unreachable!(),
        }
    }

    // This is probably bad code, but this doesn't contain any
    // important drop behavior so it's hopefully fine
    Ok(destination.leak())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = BulkCp::parse();

    let pattern = if args.floating {
        args.pattern
    } else {
        format!("^{}$", args.pattern)
    };
    let regex = Regex::new(&pattern)?;

    let mut changes = get_changes_r(
        args.recursive,
        &regex,
        &args.destination,
        PathBuf::from("."),
    )?;

    let old_len = changes.len();
    changes.dedup_by(|a, b| a.1 == b.1);
    if changes.len() != old_len {
        return Err("Can't copy multiple files to same destination".into());
    }
    if changes.len() == 0 {
        return Err("Nothing to do...".into());
    }

    let moving = args.mv
        || Path::new(&env::args_os().next().unwrap())
            .file_name()
            .is_some_and(|x| x.to_str().is_some_and(|x| x.contains("mv")));

    if !args.silent {
        if moving {
            println!("Moving:");
        } else {
            println!("Copying:");
        }
    }

    for change in changes.into_iter() {
        if !args.silent {
            println!("{} -> {}", &change.0.to_string_lossy()[2..], change.1);
        }
        if !args.dry_run {
            if moving {
                std::fs::rename(change.0, change.1)?;
            } else {
                std::fs::copy(change.0, change.1)?;
            }
        }
    }

    Ok(())
}

fn get_changes_r(
    should_recurse: bool,
    regex: &Regex,
    destination: &DestinationPattern,
    path: PathBuf,
) -> Result<Vec<(PathBuf, String)>, Box<dyn Error>> {
    let mut changes = Vec::new();
    let mut entries = path.read_dir()?;
    while let Some(entry) = entries.next() {
        let entry = entry?; // crash
        let file_type = entry.file_type()?;

        let mut path = path.clone();
        path.push(entry.file_name());

        if file_type.is_dir() {
            if should_recurse {
                changes.extend(get_changes_r(true, regex, destination, path)?);
            }

            continue;
        }

        let match_str = &path
            .to_str()
            .ok_or("filename is not valid utf-8, for some reason")?[2..]; // strip the dots off??
        let captures = regex.captures(match_str);
        if let Some(capture) = captures {
            let mut s = String::new();

            for part in destination.iter() {
                match part {
                    DestinationPatternPart::String(string) => s.push_str(&string),
                    DestinationPatternPart::Substitution(n) => {
                        s.push_str(capture.get(*n).unwrap().as_str())
                    }
                }
            }

            if metadata(&s).is_ok_and(|metadata| metadata.is_dir()) {
                if !s.ends_with('/') {
                    s.push('/');
                }
                s.push_str(match_str);
            }

            changes.push((path, s));
        }
    }

    Ok(changes)
}
