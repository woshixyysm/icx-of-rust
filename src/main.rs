use anyhow::Result;
use colored::Colorize;

mod cli;
mod diagnostics;
mod executor;
mod translator;

fn main() {
    if let Err(e) = run() {
        eprintln!("{}: {}", "icx-rustc error".bright_red().bold(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = cli::parse_args();
    
    // 显示版本信息
    if args.version {
        print_version();
        return Ok(());
    }
    
    // 显示
    if args.help {
        print_help();
        return Ok(());
    }
    
    // 翻译参数
    let rustc_cmd = translator::translate(&args)?;
    
    // 显示命令（verbose 模式）
    if args.verbose || args.dry_run {
        eprintln!(
            "{} {}",
            "[icx-rustc]".bright_blue().bold(),
            rustc_cmd.display().dimmed()
        );
    }
    
    if args.dry_run {
        return Ok(());
    }
    
    // 执行
    let exit_code = executor::run(&rustc_cmd)?;
    
    // 后处理诊断信息
    if args.optimize_diagnostics {
        diagnostics::post_process(&rustc_cmd)?;
    }
    
    std::process::exit(exit_code);
}

fn print_version() {
    println!("Intel(R) oneAPI Rust Compiler (icx-rustc)");
    println!("Version 2025.0.0 (Rust Edition)");
    println!("Target: x86_64-pc-windows-msvc / x86_64-unknown-linux-gnu");
    println!("Rustc wrapper with Intel-style command interface");
}

fn print_help() {
    println!("{}", "Intel(R) oneAPI Rust Compiler".bright_blue().bold());
    println!("Usage: icx-rustc [options] <input files>");
    println!();
    println!("{}", "Optimization Options:".yellow().bold());
    println!("  /O0, -O0          Disable optimization");
    println!("  /O1, -O1          Optimize for size");
    println!("  /O2, -O2          Optimize for speed (default)");
    println!("  /O3, -O3          Aggressive optimization");
    println!("  /Ox               Maximum optimization");
    println!("  -xHost            Optimize for host architecture");
    println!("  /arch:<feature>   Target specific architecture (AVX2, AVX512, etc.)");
    println!();
    println!("{}", "Code Generation:".yellow().bold());
    println!("  /c                Compile only, do not link");
    println!("  /o <file>         Specify output file name");
    println!("  -o <file>         Same as /o");
    println!("  /Fo<file>         Specify object file name (MSVC style)");
    println!("  /Fe<file>         Specify executable name (MSVC style)");
    println!();
    println!("{}", "Preprocessor:".yellow().bold());
    println!("  /D<name>          Define macro");
    println!("  /D<name>=<value>  Define macro with value");
    println!("  /U<name>          Undefine macro");
    println!("  /I<dir>           Add include directory");
    println!();
    println!("{}", "Linking:".yellow().bold());
    println!("  /link <options>   Pass options to linker");
    println!("  -C link-args=...  Raw linker arguments");
    println!();
    println!("{}", "Diagnostics:".yellow().bold());
    println!("  /W0, -w           Disable warnings");
    println!("  /W1, -W1          Basic warnings");
    println!("  /W3, -W           Default warnings");
    println!("  /Wall             All warnings");
    println!("  /WX               Warnings as errors");
    println!("  -v                Verbose mode");
    println!("  --###             Show commands without executing");
    println!();
    println!("{}", "Rust-specific:".yellow().bold());
    println!("  --edition <year>  Rust edition (2015/2018/2021/2024)");
    println!("  --crate-type      bin/lib/rlib/dylib/cdylib/staticlib");
    println!("  --target <triple> Cross-compilation target");
    println!();
    println!("Examples:");
    println!("  icx-rustc main.rs");
    println!("  icx-rustc /O3 /arch:AVX2 program.rs -o program.exe");
    println!("  icx-rustc /c /Fooutput.o lib.rs");
}
