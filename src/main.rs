use std::{io::Write, time::Duration};

use arboard::SetExtLinux;
use clap::Parser;
use emoji::lookup_by_glyph::iter_emoji;

mod symbols;

/// Simple program to list all emojis
#[derive(Parser, Debug)]
// #[command(author, version, about, long_about = None)]
struct Args {
    /// What language to use for the descriptions
    #[arg(short, long, default_value_t = {"en".to_string()})]
    lang: String,
    /// Fallback to english descriptions when not found
    #[arg(short, long, default_value_t = false)]
    fallback: bool,
    /// Send dmenu command output to clipoard
    #[arg(short, long, default_value_t = false)]
    copy: bool,
    /// dmenu-like command to wrap for querying
    #[arg(last = true, allow_hyphen_values = true)]
    dmenu_command: Option<Vec<String>>,
}

fn main() {
    let args = Args::parse();
    let out = std::io::stdout().lock();

    match args.dmenu_command {
        Some(ref c) => {
            let out = pick_emoji_dmenu(c, &args);
            if args.copy {
                send_to_clipboard(out.to_string());
            }
        }
        None => {
            write_emojis_to_stdout(&args, out);
        }
    }
}

fn pick_emoji_dmenu(c: &[String], args: &Args) -> char {
    let child_stdout = if args.copy {
        std::process::Stdio::piped()
    } else {
        std::process::Stdio::inherit()
    };
    let mut child = std::process::Command::new(&c[0])
        .args(&c[1..])
        .stdin(std::process::Stdio::piped())
        .stdout(child_stdout)
        .spawn()
        .unwrap();
    let stdin = child.stdin.take().unwrap();
    write_emojis_to_stdout(args, stdin);

    if !args.copy {
        return ' ';
    }

    let output = child.wait_with_output().unwrap();
    let out = String::from_utf8_lossy(&output.stdout)
        .chars()
        .take(1)
        .next()
        .unwrap_or_else(|| std::process::exit(1));

    out
}

fn send_to_clipboard(out: String) {
    if let Ok(fork::Fork::Parent(_)) = fork::fork() {
        std::thread::sleep(Duration::from_millis(100));
    } else {
        eprintln!("Setting clipboard to emoji '{out}'");

        let mut clip = arboard::Clipboard::new().unwrap();

        clip.set().wait().text(out).unwrap();
    }
}

fn write_emojis_to_stdout(args: &Args, mut out: impl Write) {
    for emoji in iter_emoji() {
        let native_description = emoji_description(emoji, &args.lang);
        let fallback_description = emoji_description(emoji, "en");
        let basic_description = emoji.name.to_string();

        let text = match (native_description, fallback_description) {
            (Some(native), Some(fallback)) if args.fallback => {
                format!("{} {}", native, fallback)
            }
            (Some(native), _) => native,
            (_, Some(fallback)) if args.fallback => fallback,
            _ if args.fallback => basic_description,
            _ => continue,
        };

        writeln!(&mut out, "{}: {}", emoji.glyph, text).ok();
    }

    for (c, desc) in symbols::SYMBOLS {
        writeln!(&mut out, "{}: {}", c, desc).ok();
    }
}

fn emoji_description(emoji: &emoji::Emoji, lang: &str) -> Option<String> {
    let annotations = emoji.annotations.iter().find(|a| a.lang == lang)?;

    let keywords = annotations.keywords.join(" ");

    if let Some(annotations) = annotations.tts {
        Some(format!("{} {}", annotations, keywords))
    } else if !keywords.is_empty() {
        Some(keywords)
    } else {
        None
    }
}
