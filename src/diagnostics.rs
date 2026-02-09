use colored::Colorize;
use regex::Regex;
use std::sync::OnceLock;

pub struct DiagnosticReporter {
    error_regex: Regex,
    warning_regex: Regex,
    note_regex: Regex,
    help_regex: Regex,
    location_regex: Regex,
}

impl DiagnosticReporter {
    pub fn new() -> Self {
        static ERROR_RE: OnceLock<Regex> = OnceLock::new();
        static WARNING_RE: OnceLock<Regex> = OnceLock::new();
        static NOTE_RE: OnceLock<Regex> = OnceLock::new();
        static HELP_RE: OnceLock<Regex> = OnceLock::new();
        static LOCATION_RE: OnceLock<Regex> = OnceLock::new();

        Self {
            error_regex: ERROR_RE
                .get_or_init(|| Regex::new(r"^error(\[E\d+\])?:").unwrap())
                .clone(),
            warning_regex: WARNING_RE
                .get_or_init(|| Regex::new(r"^warning:").unwrap())
                .clone(),
            note_regex: NOTE_RE
                .get_or_init(|| Regex::new(r"^\s*= note:").unwrap())
                .clone(),
            help_regex: HELP_RE
                .get_or_init(|| Regex::new(r"^\s*= help:").unwrap())
                .clone(),
            location_regex: LOCATION_RE
                .get_or_init(|| Regex::new(r"^\s*--> (.+):(\d+):(\d+)").unwrap())
                .clone(),
        }
    }

    /// Reports a diagnostic line and returns (warnings, errors) count delta
    pub fn report(&self, line: &str) -> (u32, u32) {
        let mut warnings = 0;
        let mut errors = 0;

        // Check for location line first (comes before error/warning)
        if let Some(caps) = self.location_regex.captures(line) {
            let file = &caps[1];
            let row = &caps[2];
            let col = &caps[3];
            println!(
                "     {} {}:{}:{}",
                "-->".bright_blue(),
                file.bright_cyan(),
                row.bright_yellow(),
                col.bright_yellow()
            );
            return (0, 0);
        }

        // Error detection
        if self.error_regex.is_match(line) {
            errors = 1;
            let formatted = self.format_error(line);
            println!("{}", formatted);
        }
        // Warning detection  
        else if self.warning_regex.is_match(line) {
            warnings = 1;
            let formatted = self.format_warning(line);
            println!("{}", formatted);
        }
        // Note
        else if self.note_regex.is_match(line) {
            let formatted = self.format_note(line);
            println!("{}", formatted);
        }
        // Help
        else if self.help_regex.is_match(line) {
            let formatted = self.format_help(line);
            println!("{}", formatted);
        }
        // Code context (lines with | )
        else if line.trim_start().starts_with('|') {
            println!("{}", self.format_code_line(line));
        }
        // Generic continuation
        else {
            println!("     {}", line.bright_black());
        }

        (warnings, errors)
    }

    fn format_error(&self, line: &str) -> String {
        let msg = self.error_regex.replace(line, "");
        format!(
            "{} {} {}",
            "error".bright_red().bold(),
            "[ICX]".bright_cyan(),
            msg.bright_white()
        )
    }

    fn format_warning(&self, line: &str) -> String {
        let msg = self.warning_regex.replace(line, "");
        format!(
            "{} {} {}",
            "warning".bright_yellow().bold(),
            "[ICX]".bright_cyan(),
            msg.bright_white()
        )
    }

    fn format_note(&self, line: &str) -> String {
        let msg = line.replace("= note:", "");
        format!(
            "     {} {}",
            "note:".bright_blue(),
            msg.trim().bright_white()
        )
    }

    fn format_help(&self, line: &str) -> String {
        let msg = line.replace("= help:", "");
        format!(
            "     {} {}",
            "help:".bright_green(),
            msg.trim().bright_white()
        )
    }

    fn format_code_line(&self, line: &str) -> String {
        // Format code with syntax highlighting hints
        if line.contains('^') {
            // This is the pointer line
            format!("     {}", line.bright_green().bold())
        } else {
            // Regular code line
            format!("     {}", line.bright_black())
        }
    }
}