use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OptLevel {
    O0, O1, O2, O3, Ox,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum WarnLevel {
    W0, W1, W2, W3, Wall, WX,
}

#[derive(Debug, Parser)]
#[command(name = "icx-rustc")]
#[command(about = "Intel-style Rust compiler wrapper")]
#[command(trailing_var_arg = true)]
pub struct Args {
    /// Input files
    pub files: Vec<PathBuf>,
    
    /// Optimization level
    #[arg(long = "O", value_enum)]
    pub opt_level: Option<OptLevel>,
    
    /// MSVC-style optimization
    #[arg(short = 'O', value_name = "level")]
    pub msvc_opt: Option<String>,
    
    /// Compile only
    #[arg(short = 'c', long = "c")]
    pub compile_only: bool,
    
    /// Output file
    #[arg(short = 'o', long = "o")]
    pub output: Option<PathBuf>,
    
    /// MSVC-style object output
    #[arg(long = "Fo")]
    pub msvc_obj: Option<PathBuf>,
    
    /// MSVC-style executable output
    #[arg(long = "Fe")]
    pub msvc_exe: Option<PathBuf>,
    
    /// Target architecture
    #[arg(long = "arch")]
    pub arch: Option<String>,
    
    /// Optimize for host
    #[arg(long = "xHost")]
    pub xhost: bool,
    
    /// Define macro
    #[arg(short = 'D', long = "D")]
    pub defines: Vec<String>,
    
    /// Undefine macro
    #[arg(short = 'U', long = "U")]
    pub undefines: Vec<String>,
    
    /// Include directory
    #[arg(short = 'I', long = "I")]
    pub includes: Vec<PathBuf>,
    
    /// Warning level
    #[arg(short = 'W', long = "W")]
    pub warn_level: Option<String>,
    
    /// Warnings as errors
    #[arg(long = "WX")]
    pub wx: bool,
    
    /// Linker arguments (MSVC style)
    #[arg(long = "link")]
    pub link_args: Vec<String>,
    
    /// Verbose
    #[arg(short = 'v', long = "v")]
    pub verbose: bool,
    
    /// Dry run
    #[arg(long = "###")]
    pub dry_run: bool,
    
    /// Show version
    #[arg(long = "version")]
    pub version: bool,
    
    /// Show help
    #[arg(long = "help")]
    pub help: bool,
    
    /// Rust edition
    #[arg(long = "edition")]
    pub edition: Option<String>,
    
    /// Crate type
    #[arg(long = "crate-type")]
    pub crate_type: Option<String>,
    
    /// Target triple
    #[arg(long = "target")]
    pub target: Option<String>,
    
    /// Release mode
    #[arg(long = "release")]
    pub release: bool,
    
    /// Optimize diagnostics output
    #[arg(long = "optimize-diagnostics", default_value = "true")]
    pub optimize_diagnostics: bool,
    
    /// Raw rustc flags (pass-through)
    #[arg(last = true)]
    pub raw_args: Vec<String>,
}

pub fn parse_args() -> Args {
    let args: Vec<String> = std::env::args()
        .map(|arg| {
            if arg.starts_with('/') && !arg.starts_with("//") {
                let without_slash = &arg[1..];
                if without_slash.contains(':') {
                    format!("--{}", without_slash.replace(':', "="))
                } else {
                    format!("-{}", without_slash)
                }
            } else {
                arg
            }
        })
        .collect();
    
    Args::parse_from(args)
}
