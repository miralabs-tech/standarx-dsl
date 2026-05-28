//! `standarx-grammar-gen` — write the editor grammar files to disk.
//!
//! Usage:
//! ```text
//! standarx-grammar-gen --out-dir <DIR>      # default: ./dist
//! standarx-grammar-gen --stdout textmate    # print to stdout
//! standarx-grammar-gen --stdout config
//! ```

use std::path::PathBuf;
use std::process::ExitCode;
use std::{env, fs, io};

use standarx_dsl_grammar::{lang_config, textmate};

const USAGE: &str = "\
standarx-grammar-gen — emit editor grammar files.

USAGE:
    standarx-grammar-gen [--out-dir <DIR>]
    standarx-grammar-gen --stdout {textmate|config}

OPTIONS:
    --out-dir <DIR>    Write standarx.tmLanguage.json and
                       standarx.language-configuration.json into DIR.
                       Default: ./dist
    --stdout <KIND>    Print one document to stdout. KIND is
                       'textmate' or 'config'.
    -h, --help         Show this message.
";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("standarx-grammar-gen: {e}");
            ExitCode::FAILURE
        }
    }
}

enum Mode {
    OutDir(PathBuf),
    Stdout(StdoutKind),
}

enum StdoutKind {
    Textmate,
    Config,
}

fn run() -> Result<(), String> {
    let mode = parse_args(env::args().skip(1))?;
    let tm = serde_json::to_string_pretty(&textmate::grammar_json())
        .map_err(|e| format!("encode tmLanguage: {e}"))?;
    let cfg = serde_json::to_string_pretty(&lang_config::config_json())
        .map_err(|e| format!("encode language-configuration: {e}"))?;

    match mode {
        Mode::Stdout(StdoutKind::Textmate) => {
            println!("{tm}");
        }
        Mode::Stdout(StdoutKind::Config) => {
            println!("{cfg}");
        }
        Mode::OutDir(dir) => {
            fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
            write_file(&dir.join("standarx.tmLanguage.json"), &tm)?;
            write_file(&dir.join("standarx.language-configuration.json"), &cfg)?;
            eprintln!(
                "wrote standarx.tmLanguage.json + standarx.language-configuration.json to {}",
                dir.display()
            );
        }
    }
    Ok(())
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Mode, String> {
    let mut out_dir: Option<PathBuf> = None;
    let mut stdout: Option<StdoutKind> = None;
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                std::process::exit(0);
            }
            "--out-dir" => {
                let value = args.next().ok_or("--out-dir needs a path")?;
                out_dir = Some(PathBuf::from(value));
            }
            "--stdout" => {
                let value = args.next().ok_or("--stdout needs 'textmate' or 'config'")?;
                stdout = Some(match value.as_str() {
                    "textmate" => StdoutKind::Textmate,
                    "config" => StdoutKind::Config,
                    other => return Err(format!("unknown --stdout kind: {other}")),
                });
            }
            other => return Err(format!("unknown argument: {other}\n\n{USAGE}")),
        }
    }
    match (out_dir, stdout) {
        (Some(_), Some(_)) => Err("--out-dir and --stdout are mutually exclusive".into()),
        (None, Some(kind)) => Ok(Mode::Stdout(kind)),
        (Some(dir), None) => Ok(Mode::OutDir(dir)),
        (None, None) => Ok(Mode::OutDir(PathBuf::from("dist"))),
    }
}

fn write_file(path: &std::path::Path, contents: &str) -> Result<(), String> {
    fs::write(path, contents).map_err(|e: io::Error| format!("write {}: {e}", path.display()))
}
