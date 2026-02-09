use crate::cli::{Args, OptLevel, WarnLevel};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
pub struct RustcCommand {
    pub executable: String,
    pub args: Vec<String>,
    pub env_vars: Vec<(String, String)>,
    pub input_files: Vec<PathBuf>,
    pub output: Option<PathBuf>,
}

impl RustcCommand {
    pub fn new() -> Self {
        Self {
            executable: "rustc".to_string(),
            args: Vec::new(),
            env_vars: Vec::new(),
            input_files: Vec::new(),
            output: None,
        }
    }
    
    pub fn display(&self) -> String {
        let mut parts = vec![self.executable.clone()];
        parts.extend(self.args.clone());
        for file in &self.input_files {
            parts.push(file.display().to_string());
        }
        if let Some(out) = &self.output {
            parts.push("-o".to_string());
            parts.push(out.display().to_string());
        }
        parts.join(" ")
    }
}

pub fn translate(args: &Args) -> Result<RustcCommand> {
    let mut cmd = RustcCommand::new();
    
    // 1. 优化级别
    translate_optimization(&mut cmd, args)?;
    
    // 2. 架构目标
    translate_architecture(&mut cmd, args)?;
    
    // 3. 编译模式
    if args.compile_only {
        cmd.args.push("--emit=obj".to_string());
    }
    
    // 4. 输出文件
    translate_output(&mut cmd, args)?;
    
    // 5. 预处理器定义
    translate_defines(&mut cmd, args)?;
    
    // 6. 警告级别
    translate_warnings(&mut cmd, args)?;
    
    // 7. 链接参数
    translate_linking(&mut cmd, args)?;
    
    // 8. Rust 特定
    translate_rust_specific(&mut cmd, args)?;
    
    // 9. 输入文件
    for file in &args.files {
        if file.extension().map_or(false, |e| e == "rs") {
            cmd.input_files.push(file.clone());
        } else {
            // 可能是库或其他输入
            cmd.args.push(file.display().to_string());
        }
    }
    
    if cmd.input_files.is_empty() && !args.version && !args.help {
        anyhow::bail!("No input files specified");
    }
    
    // 10. 透传原始参数
    cmd.args.extend(args.raw_args.clone());
    
    Ok(cmd)
}

fn translate_optimization(cmd: &mut RustcCommand, args: &Args) -> Result<()> {
    let level = match (&args.opt_level, &args.msvc_opt) {
        (Some(l), _) => match l {
            OptLevel::O0 => "0",
            OptLevel::O1 => "1",
            OptLevel::O2 => "2",
            OptLevel::O3 => "3",
            OptLevel::Ox => "3", // Ox = O3 for Rust
        },
        (None, Some(s)) => match s.as_str() {
            "0" | "d" => "0",
            "1" => "1",
            "2" => "2",
            "3" | "x" => "3",
            _ => "2",
        },
        (None, None) if args.release => "3",
        _ => "2", // default
    };
    
    cmd.args.push(format!("-Copt-level={}", level));
    
    // LTO for high optimization
    if level == "3" {
        cmd.args.push("-Clto=fat".to_string());
    }
    
    Ok(())
}

fn translate_architecture(cmd: &mut RustcCommand, args: &Args) -> Result<()> {
    if args.xhost {
        // 检测主机架构
        let target = detect_host_target()?;
        cmd.args.push(format!("--target={}", target));
        
        // 添加目标特性
        let features = detect_host_features()?;
        if !features.is_empty() {
            cmd.args.push(format!("-Ctarget-feature={}", features.join(",")));
        }
        return Ok(());
    }
    
    if let Some(arch) = &args.arch {
        let features = match arch.as_str() {
            "AVX" => vec!["+avx"],
            "AVX2" => vec!["+avx2"],
            "AVX512" | "CORE-AVX512" => vec!["+avx512f", "+avx512vl", "+avx512bw"],
            "SSE4.2" | "CORE-AVX" => vec!["+sse4.2"],
            "SSE2" => vec!["+sse2"],
            _ => {
                eprintln!("[icx-rustc] warning: unknown arch '{}', using default", arch);
                vec![]
            }
        };
        
        if !features.is_empty() {
            cmd.args.push(format!("-Ctarget-feature={}", features.join(",")));
        }
    }
    
    Ok(())
}

fn translate_output(cmd: &mut RustcCommand, args: &Args) -> Result<()> {
    // 优先级：-o > /Fe > /Fo
    let output = args.output.clone()
        .or_else(|| args.msvc_exe.clone())
        .or_else(|| args.msvc_obj.clone());
    
    if let Some(out) = output {
        cmd.output = Some(out);
    } else if args.compile_only && args.files.len() == 1 {
    // 单文件编译模式
        let input = &args.files[0];
        let stem = input.file_stem()
            .context("Invalid input filename")?;
        let obj_name = format!("{}.o", stem.to_string_lossy());
        cmd.output = Some(PathBuf::from(obj_name));
    }
    
    Ok(())
}

fn translate_defines(cmd: &mut RustcCommand, args: &Args) -> Result<()> {
    for def in &args.defines {
        if let Some((name, value)) = def.split_once('=') {
            // --cfg feature=\"value\"
            cmd.args.push(format!("--cfg={}={}", name, value));
        } else {
            // --cfg feature
            cmd.args.push(format!("--cfg={}", def));
        }
    }
    
    for undef in &args.undefines {
        eprintln!("[icx-rustc] warning: /U{} not fully supported in Rust", undef);
    }
    
    Ok(())
}

fn translate_warnings(cmd: &mut RustcCommand, args: &Args) -> Result<()> {
    if args.wx {
        cmd.args.push("-Dwarnings".to_string());
    }
    
    match &args.warn_level {
        Some(l) => match l.as_str() {
            "0" => cmd.args.push("-Awarnings".to_string()),
            "1" => {
                cmd.args.push("-Wwarnings".to_string());
                cmd.args.push("-Adead_code".to_string());
            }
            "3" | "all" => cmd.args.push("-Wwarnings".to_string()),
            _ => {}
        },
        None => {}
    }
    
    Ok(())
}

fn translate_linking(cmd: &mut RustcCommand, args: &Args) -> Result<()> {
    if !args.link_args.is_empty() {
        let joined = args.link_args.join(" ");
        cmd.args.push(format!("-Clink-args={}", shlex::quote(&joined)));
    }
    
    Ok(())
}

fn translate_rust_specific(cmd: &mut RustcCommand, args: &Args) -> Result<()> {
    if let Some(edition) = &args.edition {
        cmd.args.push(format!("--edition={}", edition));
    }
    
    if let Some(crate_type) = &args.crate_type {
        cmd.args.push(format!("--crate-type={}", crate_type));
    }
    
    if let Some(target) = &args.target {
        cmd.args.push(format!("--target={}", target));
    }
    
    cmd.args.push("-Ccodegen-units=1".to_string()); // 类似 IPO
    cmd.args.push("-Cpanic=abort".to_string());     // 类似 MSVC
    
    Ok(())
}

fn detect_host_target() -> Result<String> {
    // 简化实现，实际应使用 rustc --print target-list
    #[cfg(target_os = "windows")]
    return Ok("x86_64-pc-windows-msvc".to_string());
    
    #[cfg(target_os = "linux")]
    return Ok("x86_64-unknown-linux-gnu".to_string());
    
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    anyhow::bail!("Unsupported host platform");
}

fn detect_host_features() -> Result<Vec<String>> {
    use std::arch::is_x86_feature_detected;
    
    let mut features = vec!["+crt-static".to_string()];
    
    if is_x86_feature_detected!("avx512f") {
        features.push("+avx512f".to_string());
        features.push("+avx512vl".to_string());
    } else if is_x86_feature_detected!("avx2") {
        features.push("+avx2".to_string());
    } else if is_x86_feature_detected!("sse4.2") {
        features.push("+sse4.2".to_string());
    }
    
    Ok(features)
}
