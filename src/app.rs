use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use windows::Win32::Foundation::{HANDLE, HGLOBAL};
use windows::Win32::Globalization::{MultiByteToWideChar, CP_ACP};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};

use crate::syntax::Language;

const CF_UNICODETEXT: u32 = 13;
const MAX_FILE_BYTES: u64 = 128 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Markdown,
    Html,
    Text(Language),
}

impl Kind {
    pub fn is_previewable(self) -> bool {
        matches!(self, Self::Markdown)
    }
}

#[derive(Clone, Debug)]
pub struct Document {
    pub path: Option<PathBuf>,
    pub title: String,
    pub source: String,
    pub kind: Kind,
    pub show_source: bool,
    pub size_label: String,
    pub encoding: String,
    pub modified: Option<SystemTime>,
    pub file_len: u64,
    pub has_tables: bool,
    pub dirty: bool,
}

impl Document {
    pub fn welcome() -> Self {
        let source = WELCOME.to_string();
        Self {
            path: None,
            title: "Native Reader".to_string(),
            size_label: format!("{} 字符", source.chars().count()),
            encoding: "UTF-8".to_string(),
            modified: None,
            file_len: 0,
            has_tables: true,
            kind: Kind::Markdown,
            show_source: false,
            source,
            dirty: false,
        }
    }

    pub fn open(path: &Path) -> Result<Self, String> {
        let metadata = fs::metadata(path).map_err(|error| format!("无法读取文件信息：{error}"))?;
        if metadata.len() > MAX_FILE_BYTES {
            return Err(format!(
                "文件过大（{}），为了保护内存不打开超过 128 MB 的文件。",
                format_size(metadata.len())
            ));
        }
        let bytes = fs::read(path).map_err(|error| format!("无法读取文件：{error}"))?;
        let (source, encoding) = decode_text(&bytes)?;
        let ext = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let kind = if is_markdown_extension(&ext) {
            Kind::Markdown
        } else if is_html_extension(&ext) {
            Kind::Html
        } else {
            Kind::Text(Language::from_path(path))
        };
        let title = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("未命名")
            .to_string();
        Ok(Self {
            path: Some(path.to_path_buf()),
            title,
            size_label: format_size(metadata.len()),
            encoding,
            modified: metadata.modified().ok(),
            file_len: metadata.len(),
            has_tables: false,
            kind,
            show_source: !kind.is_previewable(),
            source,
            dirty: false,
        })
    }

    pub fn from_clipboard(text: String) -> Self {
        let kind = if looks_like_html(&text) {
            Kind::Html
        } else if looks_like_markdown(&text) {
            Kind::Markdown
        } else {
            Kind::Text(detect_language(&text))
        };
        let size_label = format!("{} 字符", text.chars().count());
        Self {
            path: None,
            title: "剪贴板内容".to_string(),
            source: text,
            kind,
            show_source: !kind.is_previewable(),
            size_label,
            encoding: "剪贴板".to_string(),
            modified: None,
            file_len: 0,
            has_tables: false,
            dirty: false,
        }
    }

    pub fn reload(&self) -> Result<Self, String> {
        let Some(path) = &self.path else {
            return Ok(self.clone());
        };
        let mut next = Self::open(path)?;
        next.show_source = self.show_source;
        Ok(next)
    }

    pub fn is_markdown(&self) -> bool {
        self.kind == Kind::Markdown
    }

    pub fn is_html(&self) -> bool {
        self.kind == Kind::Html
    }

    pub fn current_language(&self) -> Language {
        match self.kind {
            Kind::Markdown => Language::Markdown,
            Kind::Html => Language::Html,
            Kind::Text(language) => language,
        }
    }

    pub fn is_rendered_markdown(&self) -> bool {
        self.is_markdown() && !self.show_source
    }

    pub fn is_previewable(&self) -> bool {
        self.kind.is_previewable()
    }

    pub fn is_rendered_preview(&self) -> bool {
        self.is_markdown() && !self.show_source
    }

    pub fn kind_label(&self) -> &'static str {
        match self.kind {
            Kind::Markdown => "Markdown",
            Kind::Html => "HTML",
            Kind::Text(language) => language.label(),
        }
    }

    pub fn mode_label(&self) -> &'static str {
        match self.kind {
            Kind::Markdown if !self.show_source => "只读预览",
            Kind::Markdown => "Markdown 源码",
            Kind::Html => "HTML 源码 · 浏览器预览",
            Kind::Text(_) if self.show_source => "编辑",
            Kind::Text(_) => "只读查看",
        }
    }

    pub fn display_title(&self) -> String {
        let title = if self.path.is_none() && self.title == "Native Reader" {
            self.title.clone()
        } else {
            format!("{} — Native Reader", self.title)
        };
        if self.dirty {
            format!("*{title}")
        } else {
            title
        }
    }

    pub fn stamp(&self) -> Option<(u64, Option<SystemTime>)> {
        let path = self.path.as_ref()?;
        let metadata = fs::metadata(path).ok()?;
        Some((metadata.len(), metadata.modified().ok()))
    }

    pub fn save(&mut self, text: String) -> Result<(), String> {
        let Some(path) = self.path.clone() else {
            return Err("当前内容没有文件路径，请使用“另存为”。".to_string());
        };
        self.save_as(&path, text)
    }

    pub fn save_as(&mut self, path: &Path, text: String) -> Result<(), String> {
        fs::write(path, text.as_bytes()).map_err(|error| format!("无法保存文件：{error}"))?;
        let metadata =
            fs::metadata(path).map_err(|error| format!("已写入，但无法读取文件信息：{error}"))?;
        let ext = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        self.kind = if is_markdown_extension(&ext) {
            Kind::Markdown
        } else if is_html_extension(&ext) {
            Kind::Html
        } else {
            Kind::Text(Language::from_path(path))
        };
        self.path = Some(path.to_path_buf());
        self.title = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("未命名")
            .to_string();
        self.source = text;
        self.dirty = false;
        self.encoding = "UTF-8".to_string();
        self.file_len = metadata.len();
        self.size_label = format_size(metadata.len());
        self.modified = metadata.modified().ok();
        Ok(())
    }
}

pub fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / 1024.0 / 1024.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

fn is_markdown_extension(ext: &str) -> bool {
    matches!(ext, "md" | "markdown" | "mdown" | "mkd")
}

fn is_html_extension(ext: &str) -> bool {
    matches!(ext, "html" | "htm" | "xhtml")
}

pub fn looks_like_html(text: &str) -> bool {
    let trimmed = text.trim_start();
    let lower = trimmed
        .chars()
        .take(512)
        .collect::<String>()
        .to_ascii_lowercase();
    lower.starts_with("<!doctype html")
        || lower.starts_with("<html")
        || lower.starts_with("<?xml")
        || lower.contains("<body")
        || (lower.contains("<p") && lower.contains("</p>"))
}

pub fn looks_like_markdown(text: &str) -> bool {
    let mut score = 0;
    for line in text.lines().take(120) {
        let trimmed = line.trim_start();
        if trimmed.starts_with("# ")
            || trimmed.starts_with("## ")
            || trimmed.starts_with("### ")
            || trimmed.starts_with("```")
            || trimmed.starts_with("- [ ] ")
            || trimmed.starts_with("- [x] ")
            || trimmed.starts_with("> ")
        {
            score += 1;
        }
        if trimmed.contains("|---") || trimmed.contains("| ---") {
            score += 2;
        }
    }
    score >= 2
}

fn detect_language(text: &str) -> Language {
    let trimmed = text.trim_start();
    if trimmed.starts_with("<?xml")
        || trimmed.starts_with("<!DOCTYPE html")
        || trimmed.starts_with("<html")
    {
        return Language::Html;
    }
    if (trimmed.starts_with('{') || trimmed.starts_with('['))
        && trimmed.contains(':')
        && (trimmed.contains("\":") || trimmed.contains(": \""))
    {
        return Language::Json;
    }
    let first = text.lines().take(12).collect::<Vec<_>>().join("\n");
    if first.contains("#!/usr/bin/env python") || first.contains("def __init__") {
        Language::Python
    } else if first.contains("use std::") || first.contains("fn main(") {
        Language::Rust
    } else if first.contains("using System;")
        || first.contains("namespace ") && first.contains("class ")
    {
        Language::CSharp
    } else if first.contains("function ") || first.contains("const ") && first.contains("=>") {
        Language::JavaScript
    } else {
        Language::Plain
    }
}

fn decode_text(bytes: &[u8]) -> Result<(String, String), String> {
    if bytes.is_empty() {
        return Ok((String::new(), "UTF-8".to_string()));
    }
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        let text = String::from_utf8_lossy(&bytes[3..]).into_owned();
        return Ok((text, "UTF-8 BOM".to_string()));
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return Ok((decode_utf16(&bytes[2..], true), "UTF-16 LE".to_string()));
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return Ok((decode_utf16(&bytes[2..], false), "UTF-16 BE".to_string()));
    }
    if looks_like_utf16_le(bytes) {
        return Ok((decode_utf16(bytes, true), "UTF-16 LE".to_string()));
    }
    if let Ok(text) = std::str::from_utf8(bytes) {
        return Ok((text.to_string(), "UTF-8".to_string()));
    }
    if bytes.iter().take(4096).filter(|byte| **byte == 0).count() > 8 {
        return Err("该文件看起来是二进制文件，不是纯文本。".to_string());
    }
    let wide_len = unsafe { MultiByteToWideChar(CP_ACP, Default::default(), bytes, None) };
    if wide_len <= 0 {
        return Ok((
            String::from_utf8_lossy(bytes).into_owned(),
            "ANSI".to_string(),
        ));
    }
    let mut wide = vec![0u16; wide_len as usize];
    let written =
        unsafe { MultiByteToWideChar(CP_ACP, Default::default(), bytes, Some(&mut wide)) };
    if written <= 0 {
        return Ok((
            String::from_utf8_lossy(bytes).into_owned(),
            "ANSI".to_string(),
        ));
    }
    Ok((
        String::from_utf16_lossy(&wide[..written as usize]),
        "ANSI / GBK".to_string(),
    ))
}

fn decode_utf16(bytes: &[u8], little_endian: bool) -> String {
    let mut words = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks_exact(2) {
        words.push(if little_endian {
            u16::from_le_bytes([pair[0], pair[1]])
        } else {
            u16::from_be_bytes([pair[0], pair[1]])
        });
    }
    String::from_utf16_lossy(&words)
}

fn looks_like_utf16_le(bytes: &[u8]) -> bool {
    let sample = bytes.len().min(4096);
    if sample < 16 {
        return false;
    }
    let mut even_zero = 0usize;
    let mut odd_zero = 0usize;
    let mut pairs = 0usize;
    for pair in bytes[..sample].chunks_exact(2) {
        pairs += 1;
        if pair[0] == 0 {
            even_zero += 1;
        }
        if pair[1] == 0 {
            odd_zero += 1;
        }
    }
    pairs > 0 && odd_zero * 3 > pairs
}

pub fn read_clipboard_text() -> Result<String, String> {
    unsafe {
        OpenClipboard(None).map_err(|_| "无法打开剪贴板。".to_string())?;
    }
    let result = (|| unsafe {
        let handle =
            GetClipboardData(CF_UNICODETEXT).map_err(|_| "剪贴板中没有文本。".to_string())?;
        if handle.is_invalid() {
            return Err("剪贴板中没有文本。".to_string());
        }
        let global = HGLOBAL(handle.0);
        let pointer = GlobalLock(global) as *const u16;
        if pointer.is_null() {
            return Err("无法读取剪贴板文本。".to_string());
        }
        let mut len = 0usize;
        while *pointer.add(len) != 0 && len < 32 * 1024 * 1024 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(pointer, len);
        let text = String::from_utf16_lossy(slice);
        let _ = GlobalUnlock(global);
        Ok(text)
    })();
    unsafe {
        let _ = CloseClipboard();
    }
    result
}

pub fn copy_text_to_clipboard(text: &str) -> Result<(), String> {
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        OpenClipboard(None).map_err(|_| "无法打开剪贴板。".to_string())?;
    }
    let result = (|| unsafe {
        EmptyClipboard().map_err(|_| "无法清空剪贴板。".to_string())?;
        let size = wide.len() * std::mem::size_of::<u16>();
        let global = GlobalAlloc(GMEM_MOVEABLE, size).map_err(|_| "内存不足。".to_string())?;
        let pointer = GlobalLock(global) as *mut u16;
        if pointer.is_null() {
            return Err("无法写入剪贴板。".to_string());
        }
        std::ptr::copy_nonoverlapping(wide.as_ptr(), pointer, wide.len());
        let _ = GlobalUnlock(global);
        SetClipboardData(CF_UNICODETEXT, Some(HANDLE(global.0)))
            .map_err(|_| "无法设置剪贴板文本。".to_string())?;
        Ok(())
    })();
    unsafe {
        let _ = CloseClipboard();
    }
    result
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub dark: bool,
    pub topmost: bool,
    pub wrap_markdown: bool,
    pub wrap_code: bool,
    pub zoom: i32,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: i32,
    pub height: i32,
    pub defaults_prompted: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            dark: true,
            topmost: true,
            wrap_markdown: true,
            wrap_code: false,
            zoom: 100,
            x: None,
            y: None,
            width: 1120,
            height: 760,
            defaults_prompted: false,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let Some(path) = settings_path() else {
            return Self::default();
        };
        let Ok(text) = fs::read_to_string(path) else {
            return Self::default();
        };
        let mut settings = Self::default();
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();
            match key.trim() {
                "dark" => settings.dark = parse_bool(value, settings.dark),
                "topmost" => settings.topmost = parse_bool(value, settings.topmost),
                "wrap_markdown" => {
                    settings.wrap_markdown = parse_bool(value, settings.wrap_markdown)
                }
                "wrap_code" => settings.wrap_code = parse_bool(value, settings.wrap_code),
                "zoom" => {
                    settings.zoom = value.parse::<i32>().unwrap_or(settings.zoom).clamp(70, 200)
                }
                "x" => settings.x = value.parse().ok(),
                "y" => settings.y = value.parse().ok(),
                "width" => settings.width = value.parse::<i32>().unwrap_or(settings.width).max(480),
                "height" => {
                    settings.height = value.parse::<i32>().unwrap_or(settings.height).max(360)
                }
                "defaults_prompted" => {
                    settings.defaults_prompted = parse_bool(value, settings.defaults_prompted)
                }
                _ => {}
            }
        }
        settings
    }

    pub fn save(&self) {
        let Some(path) = settings_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let x = self.x.map(|value| value.to_string()).unwrap_or_default();
        let y = self.y.map(|value| value.to_string()).unwrap_or_default();
        let text = format!(
            "dark={}\ntopmost={}\nwrap_markdown={}\nwrap_code={}\nzoom={}\nx={}\ny={}\nwidth={}\nheight={}\ndefaults_prompted={}\n",
            self.dark as u8,
            self.topmost as u8,
            self.wrap_markdown as u8,
            self.wrap_code as u8,
            self.zoom,
            x,
            y,
            self.width,
            self.height,
            self.defaults_prompted as u8
        );
        let _ = fs::write(path, text);
    }
}

fn parse_bool(value: &str, fallback: bool) -> bool {
    match value.trim() {
        "1" | "true" | "yes" | "on" => true,
        "0" | "false" | "no" | "off" => false,
        _ => fallback,
    }
}

fn settings_path() -> Option<PathBuf> {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .map(|path| path.join("NativeReader").join("settings.ini"))
}

const WELCOME: &str = r#"# Native Reader

一个原生 Win32 的 Markdown / 代码 / 纯文本查看器。没有 Electron、没有 WebView，也不依赖解释器运行时。

## 快速开始

- `Ctrl+O` 打开 Markdown、代码或任意文本文件。
- `Ctrl+Shift+V` 直接把剪贴板内容渲染到窗口，适合临时查看 AI 回答、代码或表格。
- 鼠标选中文本后按 `Ctrl+C` 复制；`Ctrl+Shift+C` 复制当前文件的完整原文。
- `Ctrl+T` 切换窗口置顶，`Ctrl+F` 搜索，`F3` 查找下一个。
- 代码模式支持行号、语法着色、`Ctrl+G` 跳转行和关闭自动换行。
- 拖拽文件到窗口即可打开；文件被外部修改后会自动重新加载。

## 格式支持

| 功能 | 支持情况 | 说明 |
|---|---|---|
| Markdown 标题、列表、引用 | 是 | 原生 RichEdit 排版 |
| 表格 | 是 | 对齐、表头底色、边框 |
| 代码块 | 是 | Rust、C/C++、C#、Python、JS/TS、Go、JSON 等 |
| 链接 | 是 | 点击后用系统浏览器打开 |
| 中文与 Unicode | 是 | UTF-8、UTF-16、ANSI / GBK 自动识别 |

> 代码视图默认不换行，接近 VS Code：长行使用水平滚动条，左侧显示行号。

## 示例代码

```rust
fn main() {
    let message = "原生、轻量、快速";
    println!("{message}");
}
```

按 `Ctrl+1` 回到 Markdown 预览，按 `Ctrl+2` 查看 Markdown 源码。主题、字号和窗口位置会自动记住。
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_auto_detection() {
        assert!(looks_like_markdown(
            "# title\n\n| a | b |\n|---|---|\n| 1 | 2 |\n"
        ));
        assert!(!looks_like_markdown("plain text\nsecond line\n"));
    }

    #[test]
    fn decodes_utf16() {
        let bytes = [0xFF, 0xFE, b'A', 0, b'B', 0];
        let (text, encoding) = decode_text(&bytes).unwrap();
        assert_eq!(text, "AB");
        assert_eq!(encoding, "UTF-16 LE");
    }
}
