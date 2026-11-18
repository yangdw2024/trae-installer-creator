use anyhow::{bail, Result};
use clap::Parser;
use std::path::PathBuf;
use std::time::Instant;
use trae_core::{InstallerBuilder, IssParser};

#[derive(Parser, Debug)]
#[command(
    name = "traecc",
    about = "Trae Installer Creator Native Command-Line Compiler v3.0",
    disable_version_flag = true
)]
struct Cli {
    /// Path to the .iss script file
    script: PathBuf,

    /// Output directory
    #[arg(short = 'O', long = "output-dir")]
    output_dir: Option<PathBuf>,

    /// Output filename
    #[arg(short = 'F', long = "output-name")]
    output_name: Option<String>,

    /// Explicit path to trae-stub.exe
    #[arg(long = "stub")]
    stub: Option<PathBuf>,

    /// Quiet mode
    #[arg(short = 'Q', long = "quiet")]
    quiet: bool,

    /// Verbose mode
    #[arg(short = 'V', long = "verbose")]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli) {
        eprintln!("Error: {:#}", err);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    if !cli.script.is_file() {
        bail!("Script file not found: {}", cli.script.display());
    }

    if !cli.quiet {
        println!("Trae Installer Creator Native Compiler v3.0 (Rust)");
        println!("Compiling script: {}", cli.script.display());
    }

    let start = Instant::now();
    let cwd = std::env::current_dir()?;
    let abs_script = if cli.script.is_absolute() {
        cli.script.clone()
    } else {
        cwd.join(&cli.script)
    };
    let base_dir = abs_script
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| cwd.clone());

    let mut script = IssParser::parse_file(&abs_script)?;

    if let Some(ref out_name) = cli.output_name {
        script.setup.output_base_filename = out_name.clone();
    }
    if let Some(ref out_dir) = cli.output_dir {
        let abs_out_dir = if out_dir.is_absolute() {
            out_dir.clone()
        } else {
            cwd.join(out_dir)
        };
        script.setup.output_dir = abs_out_dir.to_string_lossy().into_owned();
    }

    let report = InstallerBuilder::build(
        &script,
        &base_dir,
        None,
        cli.stub.as_deref(),
    )?;

    let elapsed = start.elapsed();

    if !cli.quiet {
        println!("Successfully compiled single-file installer:");
        println!("  Output EXE       : {}", report.output_path.display());
        if cli.verbose {
            println!("  Stub Base        : {}", report.stub_path.display());
        }
        println!("  Files Packed     : {}", report.file_count);
        println!(
            "  Uncompressed Size: {} bytes ({:.2} MB)",
            report.uncompressed_size,
            report.uncompressed_size as f64 / (1024.0 * 1024.0)
        );
        println!(
            "  Archive Size     : {} bytes ({:.2} MB)",
            report.compressed_size,
            report.compressed_size as f64 / (1024.0 * 1024.0)
        );
        println!(
            "  Final EXE Size   : {} bytes ({:.2} MB)",
            report.total_exe_size,
            report.total_exe_size as f64 / (1024.0 * 1024.0)
        );
        println!("  Time Elapsed     : {:.3}s", elapsed.as_secs_f64());
    }

    Ok(())
}
