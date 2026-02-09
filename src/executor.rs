use crate::diagnostics::{format_diagnostic, print_summary};
use crate::translator::RustcCommand;
use anyhow::{Context, Result};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::time::Instant;

pub fn run(cmd: &RustcCommand) -> Result<i32> {
    let start = Instant::now();
    
    let mut command = Command::new(&cmd.executable);
    command.args(&cmd.args);
    
    for file in &cmd.input_files {
        command.arg(file);
    }
    
    if let Some(out) = &cmd.output {
        command.arg("-o").arg(out);
    }
    
    // 设置环境变量
    for (key, val) in &cmd.env_vars {
        command.env(key, val);
    }
    
    // 捕获输出以便处理
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    
    let mut child = command.spawn()
        .with_context(|| format!("Failed to spawn {}", cmd.executable))?;
    
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    
    let stdout_handle = std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                println!("{}", line);
            }
        }
    });
    
    let mut errors = 0;
    let mut warnings = 0;
    
    let stderr_handle = std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(line) = line {
                let formatted = format_diagnostic(&line);
                eprintln!("{}", formatted);
                
                if line.contains("error[") || line.contains("error:") {
                    // 统计错误
                } else if line.contains("warning:") {
                    // 统计警告
                }
            }
        }
    });
    
    let status = child.wait()
        .context("Failed to wait for rustc")?;
    
    stdout_handle.join().ok();
    stderr_handle.join().ok();
    
    let elapsed = start.elapsed().as_millis() as u64;
    
    print_summary(0, 0, elapsed);
    
    Ok(status.code().unwrap_or(1))
}
