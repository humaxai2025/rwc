use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;

#[derive(Default, Debug)]
struct Counts {
    bytes: usize,
    chars: usize,
    words: usize,
    lines: usize,
}

#[derive(Default)]
struct Config {
    show_bytes: bool,
    show_chars: bool,
    show_words: bool,
    show_lines: bool,
    show_json: bool,
    show_human: bool,
    files: Vec<String>,
}

impl Config {
    fn new() -> Self {
        let mut config = Config::default();
        let args: Vec<String> = env::args().collect();
        
        if args.len() == 1 {
            // Default behavior like wc - show lines, words, bytes
            config.show_lines = true;
            config.show_words = true;
            config.show_bytes = true;
            return config;
        }

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "-l" | "--lines" => config.show_lines = true,
                "-w" | "--words" => config.show_words = true,
                "-c" | "--bytes" => config.show_bytes = true,
                "-m" | "--chars" => config.show_chars = true,
                "--json" => config.show_json = true,
                "-h" | "--human" => config.show_human = true,
                "--help" => {
                    print_help();
                    std::process::exit(0);
                }
                arg if arg.starts_with('-') => {
                    eprintln!("rwc: unknown option '{}'", arg);
                    eprintln!("Try 'rwc --help' for more information.");
                    std::process::exit(1);
                }
                filename => config.files.push(filename.to_string()),
            }
            i += 1;
        }

        // If no specific counts requested, show all
        if !config.show_lines && !config.show_words && !config.show_bytes && !config.show_chars {
            config.show_lines = true;
            config.show_words = true;
            config.show_bytes = true;
        }

        config
    }
}

fn print_help() {
    println!("rwc - A modern word counter");
    println!();
    println!("USAGE:");
    println!("    rwc [OPTIONS] [FILES...]");
    println!();
    println!("OPTIONS:");
    println!("    -l, --lines     Show line count");
    println!("    -w, --words     Show word count");
    println!("    -c, --bytes     Show byte count");
    println!("    -m, --chars     Show character count");
    println!("    --json          Output in JSON format");
    println!("    -h, --human     Human readable numbers (1.2K, 1.5M)");
    println!("    --help          Show this help message");
    println!();
    println!("If no files are specified, reads from stdin.");
    println!("If no count options are specified, shows lines, words, and bytes.");
}

fn count_text(text: &str) -> Counts {
    let mut counts = Counts::default();
    
    counts.bytes = text.len();
    counts.chars = text.chars().count();
    counts.lines = text.lines().count();
    
    // More sophisticated word counting
    counts.words = text
        .split_whitespace()
        .filter(|word| !word.is_empty())
        .count();
    
    counts
}

fn count_reader<R: Read>(mut reader: R) -> io::Result<Counts> {
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer)?;
    Ok(count_text(&buffer))
}

fn format_number(num: usize, human: bool) -> String {
    if !human {
        return num.to_string();
    }
    
    if num >= 1_000_000_000 {
        format!("{:.1}G", num as f64 / 1_000_000_000.0)
    } else if num >= 1_000_000 {
        format!("{:.1}M", num as f64 / 1_000_000.0)
    } else if num >= 1_000 {
        format!("{:.1}K", num as f64 / 1_000.0)
    } else {
        num.to_string()
    }
}

fn print_counts(counts: &Counts, config: &Config, filename: Option<&str>) {
    if config.show_json {
        println!("{{");
        if let Some(name) = filename {
            println!("  \"file\": \"{}\",", name);
        }
        if config.show_lines {
            println!("  \"lines\": {},", counts.lines);
        }
        if config.show_words {
            println!("  \"words\": {},", counts.words);
        }
        if config.show_chars {
            println!("  \"chars\": {},", counts.chars);
        }
        if config.show_bytes {
            println!("  \"bytes\": {}", counts.bytes);
        }
        println!("}}");
        return;
    }

    let mut output = Vec::new();
    
    if config.show_lines {
        output.push(format_number(counts.lines, config.show_human));
    }
    if config.show_words {
        output.push(format_number(counts.words, config.show_human));
    }
    if config.show_chars {
        output.push(format_number(counts.chars, config.show_human));
    }
    if config.show_bytes {
        output.push(format_number(counts.bytes, config.show_human));
    }

    print!("{:>8}", output.join(&format!("{:>8}", "")));
    
    if let Some(name) = filename {
        println!(" {}", name);
    } else {
        println!();
    }
}

fn process_file(filename: &str, config: &Config) -> io::Result<Counts> {
    if filename == "-" {
        count_reader(io::stdin().lock())
    } else {
        let file = File::open(filename)
            .map_err(|e| io::Error::new(e.kind(), format!("rwc: {}: {}", filename, e)))?;
        count_reader(file)
    }
}

fn main() {
    let config = Config::new();
    let mut total_counts = Counts::default();
    let mut file_count = 0;

    if config.files.is_empty() {
        // Read from stdin
        match count_reader(io::stdin().lock()) {
            Ok(counts) => {
                print_counts(&counts, &config, None);
            }
            Err(e) => {
                eprintln!("rwc: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    for filename in &config.files {
        match process_file(filename, &config) {
            Ok(counts) => {
                print_counts(&counts, &config, Some(filename));
                
                // Add to totals
                total_counts.bytes += counts.bytes;
                total_counts.chars += counts.chars;
                total_counts.words += counts.words;
                total_counts.lines += counts.lines;
                file_count += 1;
            }
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }

    // Show totals if multiple files
    if file_count > 1 {
        print_counts(&total_counts, &config, Some("total"));
    }
}