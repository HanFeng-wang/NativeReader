use std::path::Path;

use crate::rtf::RtfDoc;
use crate::theme::{Color, Theme};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    Plain,
    Rust,
    C,
    Cpp,
    CSharp,
    Java,
    JavaScript,
    TypeScript,
    Python,
    Go,
    Json,
    Toml,
    Yaml,
    Sql,
    Shell,
    PowerShell,
    Html,
    Css,
    Markdown,
    Ini,
    Lua,
    Batch,
}

impl Language {
    pub fn from_path(path: &Path) -> Self {
        let ext = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        Self::from_extension(&ext)
    }

    pub fn from_extension(ext: &str) -> Self {
        match ext.trim_start_matches('.').to_ascii_lowercase().as_str() {
            "rs" => Self::Rust,
            "c" | "h" => Self::C,
            "cc" | "cpp" | "cxx" | "hpp" | "hxx" | "hh" => Self::Cpp,
            "cs" => Self::CSharp,
            "java" | "kt" | "kts" | "scala" => Self::Java,
            "js" | "jsx" | "mjs" | "cjs" => Self::JavaScript,
            "ts" | "tsx" | "mts" | "cts" => Self::TypeScript,
            "py" | "pyw" | "pyi" => Self::Python,
            "go" => Self::Go,
            "json" | "jsonc" => Self::Json,
            "toml" => Self::Toml,
            "yaml" | "yml" => Self::Yaml,
            "sql" => Self::Sql,
            "sh" | "bash" | "zsh" | "fish" | "env" => Self::Shell,
            "ps1" | "psm1" | "psd1" => Self::PowerShell,
            "html" | "htm" | "xhtml" | "xml" | "svg" | "xaml" | "csproj" | "vcxproj" => Self::Html,
            "css" | "scss" | "sass" | "less" => Self::Css,
            "md" | "markdown" | "mdown" | "mkd" => Self::Markdown,
            "ini" | "cfg" | "conf" | "properties" => Self::Ini,
            "lua" => Self::Lua,
            "bat" | "cmd" => Self::Batch,
            _ => Self::Plain,
        }
    }

    pub fn from_info(info: &str) -> Self {
        let name = info
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_start_matches('{')
            .trim_end_matches('}')
            .to_ascii_lowercase();
        match name.as_str() {
            "rust" | "rs" => Self::Rust,
            "c" => Self::C,
            "cpp" | "c++" | "cxx" | "hpp" => Self::Cpp,
            "csharp" | "c#" | "cs" => Self::CSharp,
            "java" | "kotlin" | "kt" => Self::Java,
            "javascript" | "js" | "jsx" | "node" => Self::JavaScript,
            "typescript" | "ts" | "tsx" => Self::TypeScript,
            "python" | "py" | "python3" => Self::Python,
            "go" | "golang" => Self::Go,
            "json" | "jsonc" => Self::Json,
            "toml" => Self::Toml,
            "yaml" | "yml" => Self::Yaml,
            "sql" | "mysql" | "postgresql" | "pgsql" => Self::Sql,
            "sh" | "bash" | "shell" | "zsh" => Self::Shell,
            "powershell" | "ps1" | "pwsh" => Self::PowerShell,
            "html" | "htm" | "xml" | "svg" | "xaml" => Self::Html,
            "css" | "scss" | "sass" | "less" => Self::Css,
            "md" | "markdown" => Self::Markdown,
            "ini" | "cfg" | "conf" => Self::Ini,
            "lua" => Self::Lua,
            "bat" | "batch" | "cmd" => Self::Batch,
            _ => Self::Plain,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Plain => "纯文本",
            Self::Rust => "Rust",
            Self::C => "C",
            Self::Cpp => "C++",
            Self::CSharp => "C#",
            Self::Java => "Java / Kotlin",
            Self::JavaScript => "JavaScript",
            Self::TypeScript => "TypeScript",
            Self::Python => "Python",
            Self::Go => "Go",
            Self::Json => "JSON",
            Self::Toml => "TOML",
            Self::Yaml => "YAML",
            Self::Sql => "SQL",
            Self::Shell => "Shell",
            Self::PowerShell => "PowerShell",
            Self::Html => "HTML / XML",
            Self::Css => "CSS",
            Self::Markdown => "Markdown",
            Self::Ini => "INI",
            Self::Lua => "Lua",
            Self::Batch => "Batch",
        }
    }

    fn block_comments(self) -> bool {
        matches!(
            self,
            Self::Rust
                | Self::C
                | Self::Cpp
                | Self::CSharp
                | Self::Java
                | Self::JavaScript
                | Self::TypeScript
                | Self::Go
                | Self::Css
                | Self::Sql
        )
    }

    fn line_comment(self) -> Option<&'static str> {
        match self {
            Self::Rust
            | Self::C
            | Self::Cpp
            | Self::CSharp
            | Self::Java
            | Self::JavaScript
            | Self::TypeScript
            | Self::Go => Some("//"),
            Self::Python | Self::Shell | Self::PowerShell | Self::Yaml | Self::Toml | Self::Ini => {
                Some("#")
            }
            Self::Sql | Self::Lua => Some("--"),
            Self::Batch => Some("REM "),
            _ => None,
        }
    }

    fn keywords(self) -> &'static [&'static str] {
        match self {
            Self::Rust => &[
                "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else",
                "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match",
                "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
                "super", "trait", "true", "type", "unsafe", "use", "where", "while", "yield",
            ],
            Self::C => &[
                "auto", "break", "case", "char", "const", "continue", "default", "do", "double",
                "else", "enum", "extern", "float", "for", "goto", "if", "inline", "int", "long",
                "register", "restrict", "return", "short", "signed", "sizeof", "static", "struct",
                "switch", "typedef", "union", "unsigned", "void", "volatile", "while", "NULL",
            ],
            Self::Cpp => &[
                "alignas",
                "alignof",
                "and",
                "asm",
                "auto",
                "bool",
                "break",
                "case",
                "catch",
                "char",
                "class",
                "const",
                "constexpr",
                "continue",
                "decltype",
                "default",
                "delete",
                "do",
                "double",
                "else",
                "enum",
                "explicit",
                "export",
                "extern",
                "false",
                "float",
                "for",
                "friend",
                "if",
                "inline",
                "int",
                "long",
                "mutable",
                "namespace",
                "new",
                "noexcept",
                "nullptr",
                "operator",
                "private",
                "protected",
                "public",
                "register",
                "return",
                "short",
                "signed",
                "sizeof",
                "static",
                "struct",
                "switch",
                "template",
                "this",
                "throw",
                "true",
                "try",
                "typedef",
                "typename",
                "union",
                "unsigned",
                "using",
                "virtual",
                "void",
                "volatile",
                "while",
            ],
            Self::CSharp => &[
                "abstract",
                "as",
                "async",
                "await",
                "base",
                "bool",
                "break",
                "byte",
                "case",
                "catch",
                "char",
                "class",
                "const",
                "continue",
                "decimal",
                "default",
                "delegate",
                "do",
                "double",
                "else",
                "enum",
                "event",
                "explicit",
                "extern",
                "false",
                "finally",
                "fixed",
                "float",
                "for",
                "foreach",
                "get",
                "goto",
                "if",
                "implicit",
                "in",
                "int",
                "interface",
                "internal",
                "is",
                "lock",
                "long",
                "namespace",
                "new",
                "null",
                "object",
                "operator",
                "out",
                "override",
                "params",
                "partial",
                "private",
                "protected",
                "public",
                "readonly",
                "record",
                "ref",
                "return",
                "sealed",
                "set",
                "short",
                "sizeof",
                "stackalloc",
                "static",
                "string",
                "struct",
                "switch",
                "this",
                "throw",
                "true",
                "try",
                "typeof",
                "uint",
                "ulong",
                "unchecked",
                "unsafe",
                "ushort",
                "using",
                "var",
                "virtual",
                "void",
                "volatile",
                "when",
                "where",
                "while",
                "yield",
            ],
            Self::Java => &[
                "abstract",
                "assert",
                "boolean",
                "break",
                "byte",
                "case",
                "catch",
                "char",
                "class",
                "const",
                "continue",
                "default",
                "do",
                "double",
                "else",
                "enum",
                "extends",
                "false",
                "final",
                "finally",
                "float",
                "for",
                "if",
                "implements",
                "import",
                "instanceof",
                "int",
                "interface",
                "long",
                "native",
                "new",
                "null",
                "package",
                "private",
                "protected",
                "public",
                "record",
                "return",
                "sealed",
                "short",
                "static",
                "strictfp",
                "super",
                "switch",
                "synchronized",
                "this",
                "throw",
                "throws",
                "transient",
                "true",
                "try",
                "var",
                "void",
                "volatile",
                "while",
                "yield",
                "fun",
                "val",
                "when",
                "object",
                "data",
            ],
            Self::JavaScript | Self::TypeScript => &[
                "as",
                "async",
                "await",
                "break",
                "case",
                "catch",
                "class",
                "const",
                "continue",
                "debugger",
                "default",
                "delete",
                "do",
                "else",
                "enum",
                "export",
                "extends",
                "false",
                "finally",
                "for",
                "from",
                "function",
                "get",
                "if",
                "implements",
                "import",
                "in",
                "instanceof",
                "interface",
                "let",
                "new",
                "null",
                "of",
                "private",
                "protected",
                "public",
                "readonly",
                "return",
                "set",
                "static",
                "super",
                "switch",
                "this",
                "throw",
                "true",
                "try",
                "type",
                "typeof",
                "undefined",
                "var",
                "void",
                "while",
                "with",
                "yield",
            ],
            Self::Python => &[
                "and", "as", "assert", "async", "await", "break", "class", "continue", "def",
                "del", "elif", "else", "except", "False", "finally", "for", "from", "global", "if",
                "import", "in", "is", "lambda", "None", "nonlocal", "not", "or", "pass", "raise",
                "return", "True", "try", "while", "with", "yield", "match", "case", "self", "cls",
            ],
            Self::Go => &[
                "break",
                "case",
                "chan",
                "const",
                "continue",
                "default",
                "defer",
                "else",
                "fallthrough",
                "for",
                "func",
                "go",
                "goto",
                "if",
                "import",
                "interface",
                "map",
                "package",
                "range",
                "return",
                "select",
                "struct",
                "switch",
                "type",
                "var",
                "nil",
                "true",
                "false",
                "iota",
            ],
            Self::Json => &["true", "false", "null"],
            Self::Yaml => &["true", "false", "null", "yes", "no", "on", "off"],
            Self::Toml => &["true", "false"],
            Self::Sql => &[
                "add",
                "all",
                "alter",
                "and",
                "any",
                "as",
                "asc",
                "begin",
                "between",
                "by",
                "case",
                "check",
                "column",
                "commit",
                "constraint",
                "create",
                "database",
                "default",
                "delete",
                "desc",
                "distinct",
                "drop",
                "else",
                "end",
                "exists",
                "foreign",
                "from",
                "full",
                "group",
                "having",
                "if",
                "in",
                "index",
                "inner",
                "insert",
                "into",
                "is",
                "join",
                "key",
                "left",
                "like",
                "limit",
                "not",
                "null",
                "offset",
                "on",
                "or",
                "order",
                "outer",
                "primary",
                "references",
                "right",
                "rollback",
                "select",
                "set",
                "table",
                "then",
                "top",
                "union",
                "unique",
                "update",
                "values",
                "view",
                "when",
                "where",
                "with",
            ],
            Self::Shell => &[
                "case", "do", "done", "elif", "else", "esac", "export", "fi", "for", "function",
                "if", "in", "local", "readonly", "return", "select", "then", "until", "while",
                "break", "continue", "echo", "cd", "printf", "test", "true", "false", "sudo",
            ],
            Self::PowerShell => &[
                "begin",
                "break",
                "catch",
                "class",
                "continue",
                "data",
                "do",
                "dynamicparam",
                "else",
                "elseif",
                "end",
                "enum",
                "exit",
                "filter",
                "finally",
                "for",
                "foreach",
                "from",
                "function",
                "if",
                "in",
                "param",
                "process",
                "return",
                "switch",
                "throw",
                "trap",
                "try",
                "until",
                "using",
                "var",
                "while",
                "workflow",
                "True",
                "False",
                "Null",
            ],
            Self::Lua => &[
                "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "goto",
                "if", "in", "local", "nil", "not", "or", "repeat", "return", "then", "true",
                "until", "while",
            ],
            Self::Batch => &[
                "call", "cd", "cls", "copy", "del", "dir", "echo", "else", "endlocal", "exist",
                "exit", "for", "goto", "if", "in", "mkdir", "move", "not", "pause", "pushd",
                "popd", "rem", "ren", "rmdir", "set", "setlocal", "shift", "start", "type",
            ],
            Self::Html | Self::Css | Self::Markdown | Self::Ini | Self::Plain => &[],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TokenKind {
    Plain,
    Comment,
    String,
    Number,
    Keyword,
    Type,
    Function,
    Attribute,
    Heading,
}

#[derive(Clone, Copy, Debug)]
struct Span {
    start: usize,
    end: usize,
    kind: TokenKind,
}

pub struct Highlighter {
    language: Language,
    in_block_comment: bool,
    triple_quote: Option<u8>,
    html_in_tag: bool,
}

impl Highlighter {
    pub fn new(language: Language) -> Self {
        Self {
            language,
            in_block_comment: false,
            triple_quote: None,
            html_in_tag: false,
        }
    }

    pub fn spans(&mut self, line: &str) -> Vec<Span> {
        let mut out = Vec::new();
        match self.language {
            Language::Html => self.scan_html(line, &mut out),
            Language::Markdown => self.scan_markdown(line, &mut out),
            Language::Plain => push_span(&mut out, 0, line.len(), TokenKind::Plain),
            _ => self.scan_code(line, &mut out),
        }
        out
    }

    fn scan_code(&mut self, line: &str, out: &mut Vec<Span>) {
        let bytes = line.as_bytes();
        let mut i = 0;
        let line_comment = self.language.line_comment();

        while i < bytes.len() {
            if self.in_block_comment {
                if let Some(end) = line[i..].find("*/") {
                    let end = i + end + 2;
                    push_span(out, i, end, TokenKind::Comment);
                    self.in_block_comment = false;
                    i = end;
                } else {
                    push_span(out, i, bytes.len(), TokenKind::Comment);
                    return;
                }
                continue;
            }

            if let Some(quote) = self.triple_quote {
                let marker = if quote == b'"' { "\"\"\"" } else { "'''" };
                if let Some(end) = line[i..].find(marker) {
                    let end = i + end + 3;
                    push_span(out, i, end, TokenKind::String);
                    self.triple_quote = None;
                    i = end;
                } else {
                    push_span(out, i, bytes.len(), TokenKind::String);
                    return;
                }
                continue;
            }

            if bytes[i].is_ascii_whitespace() {
                i += 1;
                continue;
            }

            if self.language == Language::Python
                && (line[i..].starts_with("\"\"\"") || line[i..].starts_with("'''"))
            {
                let quote = bytes[i];
                let marker = if quote == b'"' { "\"\"\"" } else { "'''" };
                if let Some(end) = line[i + 3..].find(marker) {
                    let end = i + 3 + end + 3;
                    push_span(out, i, end, TokenKind::String);
                    i = end;
                } else {
                    push_span(out, i, bytes.len(), TokenKind::String);
                    self.triple_quote = Some(quote);
                    return;
                }
                continue;
            }

            if self.language.block_comments() && line[i..].starts_with("/*") {
                if let Some(end) = line[i + 2..].find("*/") {
                    let end = i + 2 + end + 2;
                    push_span(out, i, end, TokenKind::Comment);
                    i = end;
                } else {
                    push_span(out, i, bytes.len(), TokenKind::Comment);
                    self.in_block_comment = true;
                    return;
                }
                continue;
            }

            if let Some(marker) = line_comment {
                if line[i..].starts_with(marker)
                    && (marker != "REM " || i == 0 || bytes[i - 1].is_ascii_whitespace())
                {
                    push_span(out, i, bytes.len(), TokenKind::Comment);
                    return;
                }
            }

            if matches!(
                self.language,
                Language::Python | Language::CSharp | Language::Rust
            ) && bytes[i] == b'#'
                && i + 1 < bytes.len()
                && (bytes[i + 1] == b'[' || bytes[i + 1] == b'!')
            {
                let end = line[i..]
                    .find(']')
                    .map(|value| i + value + 1)
                    .unwrap_or(bytes.len());
                push_span(out, i, end, TokenKind::Attribute);
                i = end;
                continue;
            }

            if (self.language == Language::C && bytes[i] == b'#')
                || (self.language == Language::Cpp && bytes[i] == b'#')
            {
                push_span(out, i, bytes.len(), TokenKind::Attribute);
                return;
            }

            if self.language == Language::Rust && bytes[i] == b'r' {
                if let Some((end, closed)) = scan_rust_raw_string(line, i) {
                    push_span(out, i, end, TokenKind::String);
                    if !closed {
                        return;
                    }
                    i = end;
                    continue;
                }
            }

            if self.language == Language::CSharp
                && (line[i..].starts_with("@\"") || line[i..].starts_with("$\""))
            {
                let start = i;
                i += 1;
                let quote = bytes[i];
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == quote {
                        if bytes[i - 1] == b'\\'
                            || (start + 1 < bytes.len() && bytes[start + 1] == b'@')
                        {
                            i += 1;
                            continue;
                        }
                        i += 1;
                        break;
                    }
                    i += char_len(bytes[i]);
                }
                push_span(out, start, i, TokenKind::String);
                continue;
            }

            if is_string_quote(self.language, bytes[i]) {
                let start = i;
                let quote = bytes[i];
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == b'\\' {
                        i += 1;
                        if i < bytes.len() {
                            i += char_len(bytes[i]);
                        }
                        continue;
                    }
                    if bytes[i] == quote {
                        i += 1;
                        break;
                    }
                    i += char_len(bytes[i]);
                }
                let kind =
                    if self.language == Language::Json && next_non_space(bytes, i) == Some(b':') {
                        TokenKind::Attribute
                    } else {
                        TokenKind::String
                    };
                push_span(out, start, i, kind);
                continue;
            }

            if bytes[i].is_ascii_digit()
                || (bytes[i] == b'.' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit())
            {
                let start = i;
                i = scan_number(bytes, i);
                push_span(out, start, i, TokenKind::Number);
                continue;
            }

            if self.language == Language::Shell && bytes[i] == b'$' {
                let start = i;
                i += 1;
                if i < bytes.len() && bytes[i] == b'{' {
                    if let Some(end) = line[i..].find('}') {
                        i += end + 1;
                    }
                } else {
                    while i < bytes.len() && is_ident_byte(bytes[i]) {
                        i += 1;
                    }
                }
                push_span(out, start, i, TokenKind::Attribute);
                continue;
            }

            if self.language == Language::PowerShell && bytes[i] == b'$' {
                let start = i;
                i += 1;
                while i < bytes.len() && (is_ident_byte(bytes[i]) || bytes[i] == b':') {
                    i += 1;
                }
                push_span(out, start, i, TokenKind::Attribute);
                continue;
            }

            if is_ident_start(bytes[i]) {
                let start = i;
                i += char_len(bytes[i]);
                while i < bytes.len() && is_ident_byte(bytes[i]) {
                    i += char_len(bytes[i]);
                }
                let word = &line[start..i];
                let mut kind = classify_word(self.language, word);

                let next = next_non_space(bytes, i);
                if matches!(
                    self.language,
                    Language::Toml | Language::Yaml | Language::Ini | Language::Css
                ) && matches!(next, Some(b'=') | Some(b':'))
                {
                    kind = TokenKind::Attribute;
                } else if next == Some(b'(') {
                    kind = TokenKind::Function;
                } else if next == Some(b'!') && self.language == Language::Rust {
                    kind = TokenKind::Attribute;
                } else if kind == TokenKind::Plain
                    && word.chars().next().map(char::is_uppercase).unwrap_or(false)
                {
                    kind = TokenKind::Type;
                }
                push_span(out, start, i, kind);
                continue;
            }

            i += char_len(bytes[i]);
        }
    }

    fn scan_markdown(&mut self, line: &str, out: &mut Vec<Span>) {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            push_span(out, 0, line.len(), TokenKind::Heading);
            return;
        }
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            push_span(out, 0, line.len(), TokenKind::Attribute);
            return;
        }

        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'`' {
                let start = i;
                let run = count_run(bytes, i, b'`');
                let marker = "`".repeat(run);
                if let Some(end) = line[i + run..].find(&marker) {
                    i += run + end + run;
                    push_span(out, start, i, TokenKind::String);
                    continue;
                }
            }
            if bytes[i] == b'[' {
                let start = i;
                if let Some(close) = line[i..].find("](") {
                    let close = i + close;
                    if let Some(end) = line[close + 2..].find(')') {
                        i = close + 2 + end + 1;
                        push_span(out, start, i, TokenKind::Attribute);
                        continue;
                    }
                }
            }
            i += char_len(bytes[i]);
        }
    }

    fn scan_html(&mut self, line: &str, out: &mut Vec<Span>) {
        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if !self.html_in_tag {
                if line[i..].starts_with("<!--") {
                    let end = line[i + 4..]
                        .find("-->")
                        .map(|value| i + 4 + value + 3)
                        .unwrap_or(bytes.len());
                    push_span(out, i, end, TokenKind::Comment);
                    i = end;
                    continue;
                }
                if bytes[i] == b'<' {
                    let start = i;
                    i += 1;
                    if i < bytes.len() && bytes[i] == b'/' {
                        i += 1;
                    }
                    while i < bytes.len() && is_tag_char(bytes[i]) {
                        i += 1;
                    }
                    push_span(out, start, i, TokenKind::Keyword);
                    self.html_in_tag = i < bytes.len() && bytes[i] != b'>';
                    continue;
                }
                i += char_len(bytes[i]);
                continue;
            }

            if line[i..].starts_with("/>") || bytes[i] == b'>' {
                let start = i;
                i += if bytes[i] == b'/' { 2 } else { 1 };
                push_span(out, start, i, TokenKind::Keyword);
                self.html_in_tag = false;
                continue;
            }
            if bytes[i].is_ascii_whitespace() {
                i += 1;
                continue;
            }
            if bytes[i] == b'=' {
                i += 1;
                continue;
            }
            if matches!(bytes[i], b'"' | b'\'') {
                let start = i;
                let quote = bytes[i];
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == quote {
                        i += 1;
                        break;
                    }
                    i += char_len(bytes[i]);
                }
                push_span(out, start, i, TokenKind::String);
                continue;
            }
            let start = i;
            while i < bytes.len()
                && !bytes[i].is_ascii_whitespace()
                && !matches!(bytes[i], b'=' | b'>' | b'/')
            {
                i += char_len(bytes[i]);
            }
            if i == start {
                i += char_len(bytes[i]);
            } else {
                push_span(out, start, i, TokenKind::Attribute);
            }
        }
    }
}

fn classify_word(language: Language, word: &str) -> TokenKind {
    if language == Language::Sql {
        let lower = word.to_ascii_lowercase();
        if language.keywords().iter().any(|keyword| *keyword == lower) {
            return TokenKind::Keyword;
        }
    } else if language.keywords().iter().any(|keyword| *keyword == word) {
        return TokenKind::Keyword;
    }
    TokenKind::Plain
}

fn scan_number(bytes: &[u8], mut i: usize) -> usize {
    if i + 1 < bytes.len()
        && bytes[i] == b'0'
        && matches!(bytes[i + 1], b'x' | b'X' | b'b' | b'B' | b'o' | b'O')
    {
        i += 2;
        while i < bytes.len() && (bytes[i].is_ascii_hexdigit() || bytes[i] == b'_') {
            i += 1;
        }
        return i;
    }
    while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'_') {
        i += 1;
    }
    if i < bytes.len() && bytes[i] == b'.' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit() {
        i += 1;
        while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'_') {
            i += 1;
        }
    }
    if i < bytes.len() && matches!(bytes[i], b'e' | b'E') {
        i += 1;
        if i < bytes.len() && matches!(bytes[i], b'+' | b'-') {
            i += 1;
        }
        while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'_') {
            i += 1;
        }
    }
    while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
        i += 1;
    }
    i
}

fn scan_rust_raw_string(line: &str, start: usize) -> Option<(usize, bool)> {
    let bytes = line.as_bytes();
    if start + 1 >= bytes.len() || bytes[start] != b'r' {
        return None;
    }
    let mut i = start + 1;
    let mut hashes = 0;
    while i < bytes.len() && bytes[i] == b'#' {
        hashes += 1;
        i += 1;
    }
    if i >= bytes.len() || bytes[i] != b'"' {
        return None;
    }
    i += 1;
    let closing = format!("\"{}", "#".repeat(hashes));
    if let Some(end) = line[i..].find(&closing) {
        Some((i + end + closing.len(), true))
    } else {
        Some((bytes.len(), false))
    }
}

fn is_string_quote(language: Language, byte: u8) -> bool {
    match language {
        Language::Lua
        | Language::Sql
        | Language::Python
        | Language::Shell
        | Language::PowerShell => {
            matches!(byte, b'"' | b'\'')
        }
        Language::JavaScript | Language::TypeScript | Language::Go => {
            matches!(byte, b'"' | b'\'' | b'`')
        }
        Language::Json => byte == b'"',
        Language::Toml | Language::Yaml | Language::Ini | Language::Css => {
            matches!(byte, b'"' | b'\'')
        }
        _ => matches!(byte, b'"' | b'\''),
    }
}

fn is_ident_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_' || byte >= 0x80
}

fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$' || byte >= 0x80
}

fn is_tag_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'-' | b'_')
}

fn next_non_space(bytes: &[u8], mut i: usize) -> Option<u8> {
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    bytes.get(i).copied()
}

fn count_run(bytes: &[u8], mut i: usize, byte: u8) -> usize {
    let start = i;
    while i < bytes.len() && bytes[i] == byte {
        i += 1;
    }
    i - start
}

fn char_len(first: u8) -> usize {
    if first < 0x80 {
        1
    } else if first < 0xE0 {
        2
    } else if first < 0xF0 {
        3
    } else {
        4
    }
}

fn push_span(out: &mut Vec<Span>, start: usize, end: usize, kind: TokenKind) {
    if start >= end {
        return;
    }
    if let Some(last) = out.last_mut() {
        if last.end == start && last.kind == kind {
            last.end = end;
            return;
        }
    }
    out.push(Span { start, end, kind });
}

fn token_color(theme: &Theme, kind: TokenKind) -> Color {
    match kind {
        TokenKind::Plain => theme.fg,
        TokenKind::Comment => theme.comment,
        TokenKind::String => theme.string,
        TokenKind::Number => theme.number,
        TokenKind::Keyword => theme.keyword,
        TokenKind::Type => theme.type_name,
        TokenKind::Function => theme.function,
        TokenKind::Attribute => theme.attribute,
        TokenKind::Heading => theme.heading,
    }
}

pub fn append_highlighted(
    doc: &mut RtfDoc,
    source: &str,
    language: Language,
    theme: Theme,
    base_pt: f32,
) {
    let font_size = (base_pt * 2.0).round().clamp(12.0, 96.0) as i32;
    let fg = doc.color(theme.fg);
    doc.raw(&format!(
        "\\pard\\f1\\fs{}\\cf{}\\sl240\\slmult1\\li0\\ri0\\sa0\\sb0 ",
        font_size, fg
    ));
    append_highlighted_runs(doc, source, language, theme, base_pt);
    doc.raw("\\par ");
}

pub fn append_highlighted_runs(
    doc: &mut RtfDoc,
    source: &str,
    language: Language,
    theme: Theme,
    base_pt: f32,
) {
    let font_size = (base_pt * 2.0).round().clamp(12.0, 96.0) as i32;
    let fg = doc.color(theme.fg);
    doc.raw(&format!("\\plain\\f1\\fs{}\\cf{} ", font_size, fg));

    let mut highlighter = Highlighter::new(language);
    let lines: Vec<&str> = source.split('\n').collect();
    for (line_index, raw_line) in lines.iter().enumerate() {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let spans = highlighter.spans(line);
        append_line_spans(doc, line, &spans, &theme);
        if line_index + 1 < lines.len() {
            doc.raw("\\line ");
        }
    }
}

fn append_line_spans(doc: &mut RtfDoc, line: &str, spans: &[Span], theme: &Theme) {
    let mut cursor = 0usize;
    for span in spans {
        let start = span.start.min(line.len());
        let end = span.end.min(line.len());
        if start < cursor || start >= end {
            continue;
        }
        if start > cursor {
            append_styled_text(doc, &line[cursor..start], TokenKind::Plain, theme);
        }
        append_styled_text(doc, &line[start..end], span.kind, theme);
        cursor = end;
    }
    if cursor < line.len() {
        append_styled_text(doc, &line[cursor..], TokenKind::Plain, theme);
    }
}

fn append_styled_text(doc: &mut RtfDoc, text: &str, kind: TokenKind, theme: &Theme) {
    if text.is_empty() {
        return;
    }
    let color_index = doc.color(token_color(theme, kind));
    doc.raw(&format!("\\cf{} ", color_index));
    doc.text(text);
}

pub fn render_code(source: &str, language: Language, theme: Theme, base_pt: f32) -> String {
    let mut doc = RtfDoc::new(theme);
    append_highlighted(&mut doc, source, language, theme, base_pt);
    doc.finish()
}

/// Renders syntax-coloured text with one real RichEdit paragraph per source
/// line.  Unlike `render_code`, this form is safe to use in the editable
/// source view: line numbers, selections, search ranges and saving all operate
/// on the same line structure as the original text.
pub fn render_code_editable(
    source: &str,
    language: Language,
    theme: Theme,
    base_pt: f32,
) -> String {
    let font_size = (base_pt * 2.0).round().clamp(12.0, 96.0) as i32;
    let mut doc = RtfDoc::new(theme);
    let fg = doc.color(theme.fg);
    doc.raw(&format!(
        "\\pard\\f1\\fs{font_size}\\cf{fg}\\sl240\\slmult1\\li0\\ri0\\sa0\\sb0 "
    ));

    let mut highlighter = Highlighter::new(language);
    let lines: Vec<&str> = source.split('\n').collect();
    for (line_index, raw_line) in lines.iter().enumerate() {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let spans = highlighter.spans(line);
        append_line_spans(&mut doc, line, &spans, &theme);
        if line_index + 1 < lines.len() {
            doc.raw("\\par ");
        }
    }
    doc.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rtf_text(rtf: &str) -> String {
        let bytes = rtf.as_bytes();
        let mut out = String::new();
        let mut i = 0usize;
        while i < bytes.len() {
            if bytes[i] == b'\\' {
                i += 1;
                if i >= bytes.len() {
                    break;
                }
                match bytes[i] {
                    b'\\' | b'{' | b'}' => {
                        out.push(bytes[i] as char);
                        i += 1;
                    }
                    b'\'' => i = (i + 3).min(bytes.len()),
                    b'\n' | b'\r' => i += 1,
                    b'*' => i += 1,
                    _ if bytes[i].is_ascii_alphabetic() => {
                        let start = i;
                        while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
                            i += 1;
                        }
                        let name = &rtf[start..i];
                        let number_start = i;
                        if i < bytes.len() && (bytes[i] == b'-' || bytes[i].is_ascii_digit()) {
                            i += 1;
                            while i < bytes.len() && bytes[i].is_ascii_digit() {
                                i += 1;
                            }
                        }
                        match name {
                            "u" => {
                                if let Ok(value) = rtf[number_start..i].parse::<i16>() {
                                    if let Some(ch) = char::from_u32(value as u16 as u32) {
                                        out.push(ch);
                                    }
                                }
                                if i < bytes.len() && bytes[i] == b'?' {
                                    i += 1;
                                }
                            }
                            "par" | "line" => out.push('\n'),
                            "tab" => out.push('\t'),
                            _ => {}
                        }
                        if i < bytes.len() && bytes[i] == b' ' {
                            i += 1;
                        }
                    }
                    _ => i += 1,
                }
            } else if bytes[i] == b'{' || bytes[i] == b'}' {
                i += 1;
            } else {
                out.push(bytes[i] as char);
                i += 1;
            }
        }
        out
    }

    #[test]
    fn detects_common_languages() {
        assert_eq!(Language::from_path(Path::new("main.rs")), Language::Rust);
        assert_eq!(Language::from_info("python"), Language::Python);
        assert_eq!(Language::from_extension("json"), Language::Json);
    }

    #[test]
    fn highlight_spans_cover_input() {
        let mut highlighter = Highlighter::new(Language::Rust);
        let line = "fn main() { let value = \"hello\"; } // comment";
        let spans = highlighter.spans(line);
        assert!(spans.iter().any(|span| span.kind == TokenKind::Keyword));
        assert!(spans.iter().any(|span| span.kind == TokenKind::String));
        assert!(spans.iter().any(|span| span.kind == TokenKind::Comment));
        for span in spans {
            assert!(line.get(span.start..span.end).is_some());
        }
    }

    #[test]
    fn editable_rtf_preserves_plain_text_and_indentation() {
        let source = "# 标题\n\n    let value = 1;\n普通结尾";
        let rtf = render_code_editable(source, Language::Markdown, crate::theme::LIGHT, 10.5);
        let text = rtf_text(&rtf);
        assert!(text.contains("标题"));
        assert!(text.contains("    let value = 1;"));
        assert!(text.contains("普通结尾"));
        assert!(text.matches('\n').count() >= 3);
    }

    #[test]
    fn highlighted_runs_keep_unstyled_text() {
        let rtf = render_code(
            "fn main() {\n    let value = \"hello\";\n}",
            Language::Rust,
            crate::theme::LIGHT,
            10.5,
        );
        let text = rtf_text(&rtf);
        assert!(text.contains("    let value = "));
        assert!(text.contains("hello"));
    }
}
