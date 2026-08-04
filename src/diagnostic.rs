use colored::Colorize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn point(at: usize) -> Self {
        Self {
            start: at,
            end: at + 1,
        }
    }

    pub fn union(self, other: Span) -> Span {
        Span::new(self.start.min(other.start), self.end.max(other.end))
    }

    pub fn extended_to(self, end: Option<usize>) -> Span {
        Span::new(self.start, end.unwrap_or(self.end))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Lexing,
    Parsing,
    Runtime,
}

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stage::Lexing => write!(f, "Lexing error"),
            Stage::Parsing => write!(f, "Parsing error"),
            Stage::Runtime => write!(f, "Runtime error"),
        }
    }
}

pub trait Diagnostic {
    fn stage(&self) -> Stage;
    fn span(&self) -> Option<Span>;
    fn message(&self) -> String;
}

pub fn line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

pub fn render(diag: &dyn Diagnostic, source: &str, source_name: &str) -> String {
    let stage = diag.stage().to_string().red();
    let message = diag.message();
    match diag.span() {
        None => format!("{}: {}: {}", source_name, stage, message),
        Some(span) => {
            let (line, col) = line_col(source, span.start);
            let (end_line, end_col) = line_col(source, span.end.max(span.start + 1));
            let line_text = source.lines().nth(line.saturating_sub(1)).unwrap_or("");
            let underline_len = if end_line == line {
                end_col.saturating_sub(col).max(1)
            } else {
                line_text.len().saturating_sub(col.saturating_sub(1)).max(1)
            };
            let pointer = format!(
                "{}{}",
                " ".repeat(col.saturating_sub(1)),
                "^".repeat(underline_len)
            );
            format!(
                "{}:{}:{}: {}: {}\n  {}\n  {}",
                source_name,
                line,
                col,
                stage,
                message,
                line_text,
                pointer.red()
            )
        }
    }
}
