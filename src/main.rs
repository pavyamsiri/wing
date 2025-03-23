mod command;
mod duration;
mod webhook;

use colored::Colorize;
use jiff::Timestamp;
use jiff::tz::TimeZone;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;

use crate::command::CommandDisplay;
use crate::command::TIMESTAMP_FORMAT;
use crate::duration::ColoredDuration;
use crate::webhook::WebhookInfo;

fn capture_and_forward(reader: impl Read, mut writer: impl Write) -> String {
    let mut buf_reader = BufReader::new(reader);
    let mut output = Vec::new();
    let mut buffer = [0u8; 1024];

    while let Ok(bytes_read) = buf_reader.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }
        let valid_slice = &buffer[..bytes_read];

        writer.write_all(valid_slice).unwrap();
        writer.flush().unwrap();
        output.extend_from_slice(&strip_ansi_escapes::strip(valid_slice));
    }
    String::from_utf8_lossy(&output).into_owned()
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install().expect("Only called once.");

    let webhook_info = WebhookInfo::from_env()?;

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        assert_eq!(args.len(), 1);
        eprintln!(
            "{}: {} {} {}",
            "Usage".bright_green(),
            args[0].bright_cyan(),
            "<command>".cyan(),
            "[args...]".cyan()
        );
        eprintln!(
            "\n{}: {} {}",
            "Example".bright_green(),
            args[0].bright_cyan(),
            "ls -la".cyan()
        );
        std::process::exit(1);
    }

    let program = &args[1];
    let cmd_args = if args.len() > 2 { &args[2..] } else { &[] };

    let mut command = Command::new(program);
    command
        .args(cmd_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    println!(
        "{}: {} - {}",
        "WING".bright_yellow(),
        "Running".bright_green(),
        CommandDisplay(&command).to_string().cyan()
    );
    let start_timestamp = Timestamp::now();
    let start_time = std::time::Instant::now();
    let mut child = command.spawn().expect("Failed to spawn child process");

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    let thread_out = std::thread::spawn(move || capture_and_forward(stdout, std::io::stdout()));
    let thread_err = std::thread::spawn(move || capture_and_forward(stderr, std::io::stderr()));

    let status = child.wait().unwrap();
    let elapsed = start_time.elapsed();
    let end_timestamp = Timestamp::now();
    let stdout = thread_out.join().unwrap();
    let stderr = thread_err.join().unwrap();

    webhook_info
        .report(
            command::CommandReport {
                command,
                elapsed,
                start: start_timestamp,
                end: end_timestamp,
            },
            &stdout,
            &stderr,
        )
        .unwrap();

    let code = match status.code() {
        Some(code) => {
            eprintln!(
                "{}: Process exited with code {}",
                "WING".bright_yellow(),
                code.to_string().bright_red()
            );
            code
        }
        None => {
            eprintln!("{}: Process terminated by signal!", "WING".bright_yellow());
            1
        }
    };

    println!(
        "{}: Ran for:  {}.",
        "WING".bright_yellow(),
        ColoredDuration(elapsed)
    );
    println!(
        "{}: Started:  {}",
        "WING".bright_yellow(),
        start_timestamp
            .to_zoned(TimeZone::system())
            .strftime(TIMESTAMP_FORMAT)
            .to_string()
            .cyan(),
    );
    println!(
        "{}: Finished: {}",
        "WING".bright_yellow(),
        end_timestamp
            .to_zoned(TimeZone::system())
            .strftime(TIMESTAMP_FORMAT)
            .to_string()
            .bright_cyan(),
    );

    std::process::exit(code);
}
