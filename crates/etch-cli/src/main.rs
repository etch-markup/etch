mod config;
mod npm;
mod plugin;

use clap::{ArgAction, Parser, Subcommand};
use etch_core::{ParseError, ParseErrorKind, parse, render_html};
use std::{
    ffi::OsStr,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Parser, Debug)]
#[command(
    name = "etch",
    version = "0.1.0",
    about = "The Etch markup language toolkit"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Render an .etch file to HTML or JSON AST
    Render {
        input: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
    },
    /// Validate an .etch file for errors and warnings
    Validate {
        input: PathBuf,
        #[arg(long, action = ArgAction::SetTrue)]
        quiet: bool,
    },
    /// Install a plugin
    Add {
        name: String,
        #[arg(long, action = ArgAction::SetTrue)]
        global: bool,
    },
    /// Remove a project-local plugin
    Remove { name: String },
    /// List installed plugins
    Plugins,
    /// Format an .etch file
    Fmt { input: PathBuf },
    /// Lint an .etch file
    Lint { input: PathBuf },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Render {
            input,
            output,
            json,
        } => render_command(&input, output.as_deref(), json),
        Commands::Validate { input, quiet } => validate_command(&input, quiet),
        Commands::Add { name, global } => plugin_command(plugin::add_plugin(&name, global)),
        Commands::Remove { name } => plugin_command(plugin::remove_plugin(&name)),
        Commands::Plugins => plugin_command(plugin::list_plugins()),
        Commands::Fmt { .. } => {
            eprintln!("fmt is not yet implemented");
            ExitCode::SUCCESS
        }
        Commands::Lint { .. } => {
            eprintln!("lint is not yet implemented");
            ExitCode::SUCCESS
        }
    }
}

fn plugin_command(result: Result<(), Box<dyn std::error::Error + Send + Sync>>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(1)
        }
    }
}

fn render_command(input: &Path, output: Option<&Path>, json: bool) -> ExitCode {
    warn_on_non_etch_extension(input);

    let source = match read_input_file(input) {
        Ok(source) => source,
        Err(()) => return ExitCode::from(1),
    };

    let result = parse(&source);

    for error in result
        .errors
        .iter()
        .filter(|issue| matches!(issue.kind, ParseErrorKind::Error))
    {
        eprintln!("line {}: {}", error.line, error.message);
    }

    let rendered = if json {
        match serde_json::to_string_pretty(&result.document) {
            Ok(json) => json,
            Err(error) => {
                eprintln!("error: failed to serialize AST to JSON: {error}");
                return ExitCode::from(1);
            }
        }
    } else {
        render_html(&result.document)
    };

    if let Some(path) = output {
        if let Err(error) = fs::write(path, rendered) {
            eprintln!("error: failed to write {}: {error}", path.display());
            return ExitCode::from(1);
        }
    } else if let Err(error) = write_stdout(&rendered) {
        eprintln!("error: failed to write output: {error}");
        return ExitCode::from(1);
    }

    ExitCode::SUCCESS
}

fn validate_command(input: &Path, quiet: bool) -> ExitCode {
    if !quiet {
        warn_on_non_etch_extension(input);
    }

    let source = match read_input_file(input) {
        Ok(source) => source,
        Err(()) => return ExitCode::from(1),
    };

    let result = parse(&source);
    let has_errors = result
        .errors
        .iter()
        .any(|issue| matches!(issue.kind, ParseErrorKind::Error));

    if !quiet {
        for issue in &result.errors {
            eprintln!("{}", format_validate_diagnostic(input, issue));
        }
    }

    if has_errors {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn read_input_file(path: &Path) -> Result<String, ()> {
    fs::read_to_string(path).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            eprintln!("error: file not found: {}", path.display());
        } else {
            eprintln!("error: failed to read {}: {error}", path.display());
        }
    })
}

fn warn_on_non_etch_extension(path: &Path) {
    if has_etch_extension(path) {
        return;
    }

    eprintln!(
        "warning: input does not have a .etch extension: {}",
        path.display()
    );
}

fn has_etch_extension(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|ext| ext.eq_ignore_ascii_case("etch"))
}

fn format_validate_diagnostic(path: &Path, issue: &ParseError) -> String {
    let severity = match issue.kind {
        ParseErrorKind::Error => "error",
        ParseErrorKind::Warning => "warning",
    };

    format!(
        "{}:{}: {}: {}",
        path.display(),
        issue.line,
        severity,
        issue.message
    )
}

fn write_stdout(output: &str) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(output.as_bytes())?;
    stdout.write_all(b"\n")
}

#[cfg(test)]
mod tests {
    use super::{format_validate_diagnostic, has_etch_extension};
    use etch_core::{ParseError, ParseErrorKind};
    use std::path::Path;

    #[test]
    fn accepts_case_insensitive_etch_extensions() {
        assert!(has_etch_extension(Path::new("story.etch")));
        assert!(has_etch_extension(Path::new("story.ETCH")));
        assert!(!has_etch_extension(Path::new("story.md")));
        assert!(!has_etch_extension(Path::new("story")));
    }

    #[test]
    fn formats_validate_diagnostics_with_filename_and_severity() {
        let diagnostic = format_validate_diagnostic(
            Path::new("tests/corpus/sample.etch"),
            &ParseError {
                kind: ParseErrorKind::Warning,
                message: "example warning".to_string(),
                line: 7,
                column: Some(1),
            },
        );

        assert_eq!(
            diagnostic,
            "tests/corpus/sample.etch:7: warning: example warning"
        );
    }
}
