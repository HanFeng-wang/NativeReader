use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::rtf::RtfDoc;
use crate::syntax::{self, Language};
use crate::theme::{Color, Theme};

pub struct RenderedMarkdown {
    pub rtf: String,
    pub has_tables: bool,
    pub links: Vec<(String, String)>,
    pub code_blocks: Vec<String>,
    pub headings: Vec<MarkdownHeading>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkdownHeading {
    pub level: usize,
    pub text: String,
}

pub const HEADING_ANCHOR: char = '\u{E000}';

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TableAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone)]
struct InlineStyle {
    bold: bool,
    italic: bool,
    strike: bool,
    underline: bool,
    code: bool,
    code_size: i32,
    color: Color,
    link: Option<String>,
}

impl InlineStyle {
    fn body(theme: Theme, code_size: i32) -> Self {
        Self {
            bold: false,
            italic: false,
            strike: false,
            underline: false,
            code: false,
            code_size,
            color: theme.fg,
            link: None,
        }
    }
}

struct InlineContext<'a> {
    theme: Theme,
    links: &'a HashMap<String, String>,
    code_size: i32,
    content_width_twips: i32,
    link_targets: RefCell<Vec<(String, String)>>,
    heading_capture: RefCell<Option<String>>,
}

pub fn render(
    source: &str,
    theme: Theme,
    base_pt: f32,
    content_width_twips: i32,
) -> RenderedMarkdown {
    let base_half = (base_pt * 2.0).round().clamp(14.0, 96.0) as i32;
    let code_size = base_half.max(16);
    let mut doc = RtfDoc::new(theme);
    let links = collect_link_definitions(source);
    let ctx = InlineContext {
        theme,
        links: &links,
        code_size,
        content_width_twips,
        link_targets: RefCell::new(Vec::new()),
        heading_capture: RefCell::new(None),
    };
    let lines: Vec<&str> = source.split('\n').collect();
    let mut tables = 0usize;
    let mut code_blocks = Vec::new();
    let mut headings = Vec::new();
    let mut i = 0usize;

    if lines.first().map(|line| line.trim()) == Some("---") {
        if let Some(end) =
            (1..lines.len()).find(|index| matches!(lines[*index].trim(), "---" | "..."))
        {
            render_code_block(
                &mut doc,
                &ctx,
                &lines[1..end],
                Language::Yaml,
                theme,
                code_size,
                &mut code_blocks,
            );
            i = end + 1;
        }
    }

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        if parse_reference_definition(line).is_some() {
            i += 1;
            continue;
        }

        if let Some((level, title)) = parse_atx_heading(line) {
            if let Some(heading) = render_heading(&mut doc, &ctx, level, &title, theme, base_half) {
                headings.push(heading);
            }
            i += 1;
            continue;
        }

        if i + 1 < lines.len() && is_setext_underline(lines[i + 1]) && !trimmed.contains('|') {
            let level = if lines[i + 1].trim_start().starts_with('=') {
                1
            } else {
                2
            };
            if let Some(heading) =
                render_heading(&mut doc, &ctx, level, line.trim(), theme, base_half)
            {
                headings.push(heading);
            }
            i += 2;
            continue;
        }

        if let Some((fence_char, fence_len, info)) = parse_fence(line) {
            let mut code = Vec::new();
            i += 1;
            while i < lines.len() {
                if is_closing_fence(lines[i], fence_char, fence_len) {
                    i += 1;
                    break;
                }
                code.push(strip_fence_indent(
                    lines[i],
                    line.len() - line.trim_start().len(),
                ));
                i += 1;
            }
            let language = Language::from_info(&info);
            render_code_block(
                &mut doc,
                &ctx,
                &code,
                language,
                theme,
                code_size,
                &mut code_blocks,
            );
            continue;
        }

        if is_table_start(&lines, i) {
            let alignments = parse_table_alignments(lines[i + 1]);
            let header = split_table_row(lines[i]);
            let mut rows = Vec::new();
            i += 2;
            while i < lines.len() {
                let row_line = lines[i];
                if row_line.trim().is_empty() || !row_line.contains('|') {
                    break;
                }
                rows.push(split_table_row(row_line));
                i += 1;
            }
            render_table(
                &mut doc,
                &ctx,
                &header,
                &rows,
                &alignments,
                theme,
                content_width_twips,
                base_half,
            );
            tables += 1;
            continue;
        }

        if is_block_quote(line) {
            let mut quote_lines = Vec::new();
            while i < lines.len() && is_block_quote(lines[i]) {
                let mut text = lines[i].trim_start();
                while text.starts_with('>') {
                    text = text[1..].strip_prefix(' ').unwrap_or(&text[1..]);
                }
                quote_lines.push(text.to_string());
                i += 1;
            }
            render_quote(&mut doc, &ctx, &quote_lines, theme, base_half);
            continue;
        }

        if let Some(_item) = parse_list_item(line) {
            render_list_item(&mut doc, &ctx, &lines, &mut i, theme, base_half);
            continue;
        }

        if is_thematic_break(line) {
            let border = doc.color(theme.border);
            doc.raw(&format!(
                "\\pard\\brdrb\\brdrs\\brdrw10\\brdrcf{}\\brsp60\\sb120\\sa180 ",
                border
            ));
            doc.raw("\\par ");
            i += 1;
            continue;
        }

        if is_indented_code(line) {
            let mut code = Vec::new();
            while i < lines.len() {
                if lines[i].trim().is_empty() {
                    code.push(String::new());
                    i += 1;
                    continue;
                }
                if !is_indented_code(lines[i]) {
                    break;
                }
                code.push(strip_indent(lines[i], 4).to_string());
                i += 1;
            }
            while code.last().map(|line| line.is_empty()).unwrap_or(false) {
                code.pop();
            }
            render_code_block(
                &mut doc,
                &ctx,
                &code,
                Language::Plain,
                theme,
                code_size,
                &mut code_blocks,
            );
            continue;
        }

        let mut paragraph = Vec::new();
        while i < lines.len() {
            let current = lines[i];
            if current.trim().is_empty() {
                break;
            }
            if !paragraph.is_empty() && starts_block(&lines, i) {
                break;
            }
            paragraph.push(current.to_string());
            i += 1;
        }
        render_paragraph(&mut doc, &ctx, &paragraph, theme, base_half);
    }

    doc.raw("\\pard ");
    RenderedMarkdown {
        rtf: doc.finish(),
        has_tables: tables > 0,
        links: ctx.link_targets.into_inner(),
        code_blocks,
        headings,
    }
}

fn color_index(doc: &mut RtfDoc, color: Color) -> usize {
    doc.color(color)
}

fn render_heading(
    doc: &mut RtfDoc,
    ctx: &InlineContext<'_>,
    level: usize,
    text: &str,
    theme: Theme,
    base_half: i32,
) -> Option<MarkdownHeading> {
    let size = match level {
        1 => base_half + 12,
        2 => base_half + 6,
        3 => base_half + 3,
        4 => base_half + 1,
        5 => base_half,
        _ => base_half - 1,
    }
    .max(22);
    let heading_color = color_index(doc, theme.heading);
    doc.raw(&format!(
        "\\pard\\plain\\f2\\fs{}\\b\\cf{}\\keepn\\sb{}\\sa{}\\sl300\\slmult1\\li0\\ri0 ",
        size,
        heading_color,
        if level == 1 { 220 } else { 180 },
        if level <= 2 { 80 } else { 50 },
    ));
    let mut style = InlineStyle::body(theme, ctx.code_size);
    style.bold = true;
    style.color = theme.heading;
    doc.raw("{\\v \\u-8192?}");
    *ctx.heading_capture.borrow_mut() = Some(String::new());
    render_inline(doc, text, ctx, &style);
    let captured = ctx.heading_capture.borrow_mut().take().unwrap_or_default();
    doc.raw("\\par ");
    let text = captured
        .trim()
        .replace('\r', " ")
        .replace('\n', " ")
        .replace('\t', " ");
    if text.is_empty() {
        None
    } else {
        Some(MarkdownHeading { level, text })
    }
}

fn render_paragraph(
    doc: &mut RtfDoc,
    ctx: &InlineContext<'_>,
    lines: &[String],
    theme: Theme,
    base_half: i32,
) {
    if lines.is_empty() {
        return;
    }
    let mut text = String::new();
    for (index, line) in lines.iter().enumerate() {
        let trailing = line.len() - line.trim_end().len();
        let hard_break = trailing >= 2 || line.trim_end().ends_with('\\');
        let mut part = line.trim_end().to_string();
        if part.ends_with('\\') && hard_break {
            part.pop();
        }
        text.push_str(part.trim_start());
        if index + 1 < lines.len() {
            text.push(if hard_break { '\n' } else { ' ' });
        }
    }
    let fg = color_index(doc, theme.fg);
    doc.raw(&format!(
        "\\pard\\plain\\f2\\fs{}\\cf{}\\sl312\\slmult1\\li0\\ri0\\fi0\\sb60\\sa120 ",
        base_half, fg
    ));
    let style = InlineStyle::body(theme, ctx.code_size);
    render_inline(doc, &text, ctx, &style);
    doc.raw("\\par ");
}

fn render_quote(
    doc: &mut RtfDoc,
    ctx: &InlineContext<'_>,
    lines: &[String],
    theme: Theme,
    base_half: i32,
) {
    let border = color_index(doc, theme.border);
    let muted = color_index(doc, theme.muted);
    doc.raw(&format!(
        "\\pard\\plain\\f2\\fs{}\\cf{}\\i\\sl312\\slmult1\\li360\\ri220\\sb80\\sa130\\brdrl\\brdrs\\brdrw14\\brdrcf{}\\brsp120 ",
        base_half, muted, border
    ));
    let joined = lines.join(" ");
    let mut style = InlineStyle::body(theme, ctx.code_size);
    style.italic = true;
    style.color = theme.muted;
    render_inline(doc, &joined, ctx, &style);
    doc.raw("\\par ");
}

fn render_list_item(
    doc: &mut RtfDoc,
    ctx: &InlineContext<'_>,
    lines: &[&str],
    index: &mut usize,
    theme: Theme,
    base_half: i32,
) {
    let line = lines[*index];
    let Some(item) = parse_list_item(line) else {
        *index += 1;
        return;
    };
    let indent = (item.indent / 2).min(8) as i32;
    let left = 300 + indent * 300;
    let mut content = item.content.to_string();
    *index += 1;

    while *index < lines.len() {
        let next = lines[*index];
        if next.trim().is_empty() || starts_block(lines, *index) {
            break;
        }
        let leading = next.len() - next.trim_start().len();
        if leading <= item.indent {
            break;
        }
        content.push(' ');
        content.push_str(next.trim());
        *index += 1;
    }

    let (prefix, body) = if let Some(rest) = content.strip_prefix("[ ] ") {
        ("☐ ".to_string(), rest.to_string())
    } else if let Some(rest) = content
        .strip_prefix("[x] ")
        .or_else(|| content.strip_prefix("[X] "))
    {
        ("☑ ".to_string(), rest.to_string())
    } else {
        (item.marker.clone(), content)
    };

    let fg = color_index(doc, theme.fg);
    let accent = if prefix == "☐ " || prefix == "☑ " {
        color_index(doc, theme.accent)
    } else {
        fg
    };
    doc.raw(&format!(
        "\\pard\\plain\\f2\\fs{}\\cf{}\\sl312\\slmult1\\li{}\\fi-240\\sb30\\sa55 ",
        base_half, fg, left
    ));
    if !prefix.is_empty() {
        let prefix_color =
            if prefix.ends_with(' ') && (prefix.starts_with('☐') || prefix.starts_with('☑')) {
                accent
            } else {
                fg
            };
        doc.raw(&format!("\\cf{} ", prefix_color));
        doc.text(&prefix);
    }
    let style = InlineStyle::body(theme, ctx.code_size);
    render_inline(doc, &body, ctx, &style);
    doc.raw("\\par ");
}

fn render_code_block(
    doc: &mut RtfDoc,
    ctx: &InlineContext<'_>,
    lines: &[impl AsRef<str>],
    language: Language,
    theme: Theme,
    code_size: i32,
    code_blocks: &mut Vec<String>,
) {
    let bg = color_index(doc, theme.code_bg);
    let header_bg = color_index(doc, theme.code_header_bg);
    let border = color_index(doc, theme.border);
    let fg = color_index(doc, theme.code_fg);
    let header_fg = color_index(doc, theme.muted);
    let mut code = String::new();
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            code.push('\n');
        }
        code.push_str(line.as_ref());
    }
    let block_index = code_blocks.len();
    code_blocks.push(code.clone());

    let header_size = (code_size - 2).max(14);
    let table_width = (ctx.content_width_twips - 420).clamp(3600, 19000);
    let available = (table_width - 360).max(1200);
    let char_width = header_size as f32 * 10.0 * 0.71;
    let max_chars = (available as f32 / char_width).floor() as usize;
    let label_chars = display_width(language.label());
    let padding = max_chars.saturating_sub(label_chars + 3).max(1);

    doc.raw("\\pard\\plain\\li0\\ri0\\fi0\\sa0 ");
    doc.raw(&format!(
        "\\trowd\\trgaph80\\trleft0\\trrh0\\clbrdrt\\brdrs\\brdrw10\\brdrcf{}\\clbrdrl\\brdrs\\brdrw10\\brdrcf{}\\clbrdrb\\brdrs\\brdrw10\\brdrcf{}\\clbrdrr\\brdrs\\brdrw10\\brdrcf{}\\clcbpat{}\\cellx{} ",
        border, border, border, border, header_bg, table_width
    ));
    doc.raw(&format!(
        "\\pard\\plain\\intbl\\f1\\fs{}\\cf{}\\sb70\\sa50\\li180\\ri160\\ql ",
        header_size, header_fg
    ));
    doc.raw("\\b ");
    doc.text(language.label());
    doc.raw("\\b0 ");
    doc.text(&"\u{00A0}".repeat(padding));
    doc.raw(&format!("\\f3\\fs{} ", (header_size + 4).max(18)));
    let mut action_style = InlineStyle::body(theme, ctx.code_size);
    action_style.color = theme.muted;
    action_style.link = Some(format!("native-reader://copy-code/{block_index}"));
    emit_text(doc, ctx, "\u{E8C8}", &action_style);
    doc.raw("\\f2\\cell\\row ");

    doc.raw(&format!(
        "\\trowd\\trgaph80\\trleft0\\trrh0\\clbrdrl\\brdrs\\brdrw10\\brdrcf{}\\clbrdrb\\brdrs\\brdrw10\\brdrcf{}\\clbrdrr\\brdrs\\brdrw10\\brdrcf{}\\clcbpat{}\\cellx{} ",
        border, border, border, bg, table_width
    ));
    doc.raw(&format!(
        "\\pard\\plain\\intbl\\f1\\fs{}\\cf{}\\sb55\\sa55\\li180\\ri160\\sl264\\slmult1 ",
        code_size, fg
    ));
    syntax::append_highlighted_runs(doc, &code, language, theme, code_size as f32 / 2.0);
    doc.raw("\\cell\\row ");
    doc.raw("\\pard\\plain\\sa170 ");
}

fn render_table(
    doc: &mut RtfDoc,
    ctx: &InlineContext<'_>,
    header: &[String],
    rows: &[Vec<String>],
    alignments: &[TableAlign],
    theme: Theme,
    content_width_twips: i32,
    base_half: i32,
) {
    let columns = header.len().max(1);
    let target_width = content_width_twips.clamp(4200, 18000);
    let mut weights = vec![6usize; columns];
    for (index, cell) in header.iter().enumerate() {
        weights[index] = weights[index].max(display_width(cell).min(40));
    }
    for row in rows {
        for index in 0..columns {
            let width = row.get(index).map(|cell| display_width(cell)).unwrap_or(0);
            weights[index] = weights[index].max(width.min(40));
        }
    }
    let min_width = 850i32;
    let total_weight: usize = weights.iter().sum::<usize>().max(1);
    let remaining = (target_width - min_width * columns as i32).max(0);
    let mut widths = Vec::with_capacity(columns);
    for weight in &weights {
        widths.push(min_width + remaining * *weight as i32 / total_weight as i32);
    }

    let border = color_index(doc, theme.border);
    let header_bg = color_index(doc, theme.table_header_bg);
    let alternate_bg = color_index(doc, theme.table_alt_bg);
    let fg = color_index(doc, theme.fg);
    let fs = (base_half - 1).max(16);

    render_table_row(
        doc, ctx, header, &widths, alignments, true, header_bg, border, fg, fs, theme,
    );
    for (row_index, row) in rows.iter().enumerate() {
        let background = if row_index % 2 == 1 { alternate_bg } else { 0 };
        render_table_row(
            doc, ctx, row, &widths, alignments, false, background, border, fg, fs, theme,
        );
    }
    doc.raw("\\pard\\plain\\sa160 ");
}

#[allow(clippy::too_many_arguments)]
fn render_table_row(
    doc: &mut RtfDoc,
    ctx: &InlineContext<'_>,
    cells: &[String],
    widths: &[i32],
    alignments: &[TableAlign],
    header: bool,
    background: usize,
    border: usize,
    fg: usize,
    font_size: i32,
    theme: Theme,
) {
    doc.raw("\\trowd\\trgaph120\\trleft0\\trrh0 ");
    let mut cursor = 0i32;
    for width in widths {
        cursor += *width;
        doc.raw(&format!(
            "\\clbrdrt\\brdrs\\brdrw6\\brdrcf{}\\clbrdrl\\brdrs\\brdrw6\\brdrcf{}\\clbrdrb\\brdrs\\brdrw6\\brdrcf{}\\clbrdrr\\brdrs\\brdrw6\\brdrcf{}",
            border, border, border, border
        ));
        if background != 0 {
            doc.raw(&format!("\\clcbpat{}", background));
        }
        doc.raw(&format!("\\cellx{} ", cursor));
    }
    for (index, width) in widths.iter().enumerate() {
        let _ = width;
        let align = match alignments.get(index).copied().unwrap_or(TableAlign::Left) {
            TableAlign::Left => "\\ql ",
            TableAlign::Center => "\\qc ",
            TableAlign::Right => "\\qr ",
        };
        doc.raw(&format!(
            "\\pard\\plain\\intbl\\f2\\fs{}\\cf{}\\sb55\\sa55\\li0\\ri90\\fi0{}{}",
            font_size,
            fg,
            if header { "\\b " } else { "\\b0 " },
            align
        ));
        let style = InlineStyle::body(theme, ctx.code_size);
        if let Some(cell) = cells.get(index) {
            render_inline(doc, cell, ctx, &style);
        }
        doc.raw("\\cell ");
    }
    doc.raw("\\row ");
}

fn render_inline(doc: &mut RtfDoc, text: &str, ctx: &InlineContext<'_>, style: &InlineStyle) {
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            if i + 1 < bytes.len() && is_ascii_punctuation(bytes[i + 1]) {
                let next_len = utf8_len(bytes[i + 1]);
                emit_text(doc, ctx, &text[i + 1..i + 1 + next_len], style);
                i += 1 + next_len;
            } else {
                emit_text(doc, ctx, "\\", style);
                i += 1;
            }
            continue;
        }

        if bytes[i] == b'\n' {
            if let Some(capture) = ctx.heading_capture.borrow_mut().as_mut() {
                capture.push(' ');
            }
            doc.raw("\\line ");
            i += 1;
            continue;
        }

        if bytes[i] == b'`' {
            let run = count_byte_run(bytes, i, b'`');
            let marker = "`".repeat(run);
            if let Some(offset) = text[i + run..].find(&marker) {
                let close = i + run + offset;
                let mut code = &text[i + run..close];
                if code.starts_with(' ') && code.ends_with(' ') && code.len() > 1 {
                    code = &code[1..code.len() - 1];
                }
                let mut code_style = style.clone();
                code_style.code = true;
                code_style.code_size = ctx.code_size;
                code_style.color = ctx.theme.code_fg;
                code_style.bold = false;
                code_style.italic = false;
                code_style.strike = false;
                code_style.underline = false;
                code_style.link = None;
                emit_text(doc, ctx, &code.replace('\n', " "), &code_style);
                i = close + run;
                continue;
            }
        }

        if bytes[i] == b'!' && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            if let Some(link) = parse_link(text, i + 1, ctx.links) {
                let label = if link.label.trim().is_empty() {
                    "图片".to_string()
                } else {
                    format!("图片：{}", link.label.trim())
                };
                let mut image_style = style.clone();
                image_style.color = ctx.theme.link;
                image_style.link = Some(link.url.clone());
                image_style.underline = true;
                emit_text(doc, ctx, &label, &image_style);
                i = link.end;
                continue;
            }
        }

        if bytes[i] == b'[' {
            if let Some(link) = parse_link(text, i, ctx.links) {
                let mut link_style = style.clone();
                link_style.color = ctx.theme.link;
                link_style.underline = true;
                link_style.link = Some(link.url);
                emit_text(doc, ctx, &link.label, &link_style);
                i = link.end;
                continue;
            }
        }

        if bytes[i] == b'<' {
            if let Some(close) = text[i + 1..].find('>') {
                let close = i + 1 + close;
                let inner = &text[i + 1..close];
                if is_auto_link(inner) {
                    let mut link_style = style.clone();
                    link_style.color = ctx.theme.link;
                    link_style.underline = true;
                    link_style.link = Some(inner.to_string());
                    emit_text(doc, ctx, inner, &link_style);
                    i = close + 1;
                    continue;
                }
                if inner.starts_with('/')
                    || inner.starts_with('!')
                    || inner.starts_with('?')
                    || inner
                        .chars()
                        .next()
                        .map(|ch| ch.is_ascii_alphabetic())
                        .unwrap_or(false)
                {
                    i = close + 1;
                    continue;
                }
            }
        }

        if bytes[i] == b'~' && i + 1 < bytes.len() && bytes[i + 1] == b'~' {
            if let Some(offset) = text[i + 2..].find("~~") {
                let close = i + 2 + offset;
                if close > i + 2 {
                    let mut strike = style.clone();
                    strike.strike = true;
                    render_inline(doc, &text[i + 2..close], ctx, &strike);
                    i = close + 2;
                    continue;
                }
            }
        }

        if (bytes[i] == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'*')
            || (bytes[i] == b'_' && i + 1 < bytes.len() && bytes[i + 1] == b'_')
        {
            let marker = &text[i..i + 2];
            if let Some(offset) = text[i + 2..].find(marker) {
                let close = i + 2 + offset;
                if close > i + 2 {
                    let mut bold = style.clone();
                    bold.bold = true;
                    render_inline(doc, &text[i + 2..close], ctx, &bold);
                    i = close + 2;
                    continue;
                }
            }
        }

        if bytes[i] == b'*' || bytes[i] == b'_' {
            if i + 1 < bytes.len() && !bytes[i + 1].is_ascii_whitespace() {
                let marker = &text[i..i + 1];
                if let Some(offset) = text[i + 1..].find(marker) {
                    let close = i + 1 + offset;
                    if close > i + 1 && !text[..close].ends_with(' ') {
                        let mut italic = style.clone();
                        italic.italic = true;
                        render_inline(doc, &text[i + 1..close], ctx, &italic);
                        i = close + 1;
                        continue;
                    }
                }
            }
        }

        if style.link.is_none()
            && (text[i..].starts_with("http://") || text[i..].starts_with("https://"))
        {
            let mut end = i;
            while end < bytes.len()
                && !bytes[end].is_ascii_whitespace()
                && !matches!(bytes[end], b'<' | b'>' | b'"' | b'\'')
            {
                end += utf8_len(bytes[end]);
            }
            while end > i && matches!(bytes[end - 1], b'.' | b',' | b';' | b':' | b')' | b'!') {
                end -= 1;
            }
            let url = &text[i..end];
            let mut link_style = style.clone();
            link_style.color = ctx.theme.link;
            link_style.underline = true;
            link_style.link = Some(url.to_string());
            emit_text(doc, ctx, url, &link_style);
            i = end;
            continue;
        }

        let start = i;
        while i < bytes.len()
            && !matches!(
                bytes[i],
                b'\\' | b'`' | b'*' | b'_' | b'~' | b'[' | b'!' | b'<' | b'\n'
            )
        {
            i += utf8_len(bytes[i]);
        }
        if i == start {
            i += utf8_len(bytes[i]);
        }
        emit_text(doc, ctx, &text[start..i], style);
    }
}

fn emit_text(doc: &mut RtfDoc, ctx: &InlineContext<'_>, text: &str, style: &InlineStyle) {
    if text.is_empty() {
        return;
    }
    let decoded = decode_entities(text);
    if let Some(capture) = ctx.heading_capture.borrow_mut().as_mut() {
        capture.push_str(decoded.as_ref());
    }
    if let Some(url) = &style.link {
        let label = decoded.to_string();
        ctx.link_targets.borrow_mut().push((label, url.clone()));
        if url.starts_with("native-reader://") {
            let mut inner = style.clone();
            inner.link = None;
            inner.underline = false;
            inner.color = ctx.theme.link;
            emit_text_group(doc, text, &inner);
            return;
        }
        doc.raw("{\\field{\\*\\fldinst{HYPERLINK \"");
        doc.text(&sanitize_url(url));
        doc.raw("\"}}{\\fldrslt{");
        let mut inner = style.clone();
        inner.link = None;
        inner.underline = true;
        inner.color = ctx.theme.link;
        emit_text_group(doc, text, &inner);
        doc.raw("}}}");
        return;
    }
    emit_text_group(doc, text, style);
}

fn emit_text_group(doc: &mut RtfDoc, text: &str, style: &InlineStyle) {
    doc.raw("{");
    if style.bold {
        doc.raw("\\b ");
    }
    if style.italic {
        doc.raw("\\i ");
    }
    if style.strike {
        doc.raw("\\strike ");
    }
    if style.underline {
        doc.raw("\\ul ");
    }
    if style.code {
        doc.raw(&format!(
            "\\b0\\i0\\strike0\\ul0\\f1\\fs{} ",
            style.code_size
        ));
    }
    let color = color_index(doc, style.color);
    doc.raw(&format!("\\cf{} ", color));
    let decoded = decode_entities(text);
    doc.text(decoded.as_ref());
    doc.raw("}");
}

struct ParsedLink {
    label: String,
    url: String,
    end: usize,
}

fn parse_link(text: &str, start: usize, links: &HashMap<String, String>) -> Option<ParsedLink> {
    if text.as_bytes().get(start) != Some(&b'[') {
        return None;
    }
    let close = find_matching(text, start, b'[', b']')?;
    let label = &text[start + 1..close];
    let after = close + 1;
    if text.as_bytes().get(after) == Some(&b'(') {
        let end_paren = find_matching(text, after, b'(', b')')?;
        let target = text[after + 1..end_paren].trim();
        let url = parse_link_target(target);
        if url.is_empty() {
            return None;
        }
        return Some(ParsedLink {
            label: label.to_string(),
            url,
            end: end_paren + 1,
        });
    }
    if text.as_bytes().get(after) == Some(&b'[') {
        let ref_close = find_matching(text, after, b'[', b']')?;
        let id = text[after + 1..ref_close].trim().to_ascii_lowercase();
        let lookup = if id.is_empty() {
            label.trim().to_ascii_lowercase()
        } else {
            id
        };
        if let Some(url) = links.get(&lookup) {
            return Some(ParsedLink {
                label: label.to_string(),
                url: url.clone(),
                end: ref_close + 1,
            });
        }
    }
    let lookup = label.trim().to_ascii_lowercase();
    links.get(&lookup).map(|url| ParsedLink {
        label: label.to_string(),
        url: url.clone(),
        end: after,
    })
}

fn find_matching(text: &str, start: usize, open: u8, close: u8) -> Option<usize> {
    let bytes = text.as_bytes();
    if bytes.get(start).copied() != Some(open) {
        return None;
    }
    let mut depth = 1usize;
    let mut i = start + 1;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 1 + bytes.get(i + 1).map(|byte| utf8_len(*byte)).unwrap_or(0);
            continue;
        }
        if bytes[i] == open {
            depth += 1;
        } else if bytes[i] == close {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += utf8_len(bytes[i]);
    }
    None
}

fn parse_link_target(target: &str) -> String {
    let target = target.trim();
    if target.starts_with('<') {
        if let Some(end) = target.find('>') {
            return target[1..end].to_string();
        }
    }
    target
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_matches('"')
        .to_string()
}

fn is_auto_link(text: &str) -> bool {
    text.starts_with("http://")
        || text.starts_with("https://")
        || text.starts_with("mailto:")
        || (text.contains('@') && !text.contains(' '))
}

fn sanitize_url(url: &str) -> String {
    url.chars()
        .filter(|ch| !matches!(ch, '\r' | '\n' | '"' | '{' | '}'))
        .collect()
}

fn decode_entities(text: &str) -> Cow<'_, str> {
    if !text.contains('&') {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(index) = rest.find('&') {
        out.push_str(&rest[..index]);
        rest = &rest[index..];
        if let Some(end) = rest.find(';') {
            if end <= 12 {
                let entity = &rest[1..end];
                let decoded = match entity {
                    "amp" => Some('&'),
                    "lt" => Some('<'),
                    "gt" => Some('>'),
                    "quot" => Some('"'),
                    "apos" => Some('\''),
                    "nbsp" => Some('\u{00A0}'),
                    _ if entity.starts_with("#x") || entity.starts_with("#X") => {
                        u32::from_str_radix(&entity[2..], 16)
                            .ok()
                            .and_then(char::from_u32)
                    }
                    _ if entity.starts_with('#') => {
                        entity[1..].parse::<u32>().ok().and_then(char::from_u32)
                    }
                    _ => None,
                };
                if let Some(ch) = decoded {
                    out.push(ch);
                    rest = &rest[end + 1..];
                    continue;
                }
            }
        }
        out.push('&');
        rest = &rest[1..];
    }
    out.push_str(rest);
    Cow::Owned(out)
}

fn collect_link_definitions(source: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let mut in_fence: Option<(u8, usize)> = None;
    for line in source.lines() {
        if let Some((fence, len)) = in_fence {
            if is_closing_fence(line, fence, len) {
                in_fence = None;
            }
            continue;
        }
        if let Some((fence, len, _)) = parse_fence(line) {
            in_fence = Some((fence, len));
            continue;
        }
        if let Some((id, url)) = parse_reference_definition(line) {
            result.insert(id, url);
        }
    }
    result
}

fn parse_reference_definition(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('[') {
        return None;
    }
    let close = trimmed.find("]:")?;
    if close <= 1 {
        return None;
    }
    let id = trimmed[1..close].trim().to_ascii_lowercase();
    let url = parse_link_target(trimmed[close + 2..].trim());
    if id.is_empty() || url.is_empty() {
        None
    } else {
        Some((id, url))
    }
}

fn parse_atx_heading(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim_start();
    let level = trimmed.chars().take_while(|ch| *ch == '#').count();
    if !(1..=6).contains(&level) {
        return None;
    }
    let rest = &trimmed[level..];
    if !rest.is_empty() && !rest.starts_with(' ') && !rest.starts_with('\t') {
        return None;
    }
    let mut title = rest.trim().to_string();
    if title.ends_with('#') {
        let trailing = title.chars().rev().take_while(|ch| *ch == '#').count();
        let before = title[..title.len() - trailing].to_string();
        if before.is_empty() || before.ends_with(' ') || before.ends_with('\t') {
            title = before.trim_end().to_string();
        }
    }
    Some((level, title.trim().to_string()))
}

fn is_setext_underline(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.len() < 1 || trimmed.contains(' ') || trimmed.contains('|') {
        return false;
    }
    (trimmed.chars().all(|ch| ch == '=') || trimmed.chars().all(|ch| ch == '-'))
}

fn parse_fence(line: &str) -> Option<(u8, usize, String)> {
    let trimmed = line.trim_start();
    if line.len() - trimmed.len() > 3 {
        return None;
    }
    let first = *trimmed.as_bytes().first()?;
    if first != b'`' && first != b'~' {
        return None;
    }
    let len = count_byte_run(trimmed.as_bytes(), 0, first);
    if len < 3 {
        return None;
    }
    Some((first, len, trimmed[len..].trim().to_string()))
}

fn is_closing_fence(line: &str, fence: u8, len: usize) -> bool {
    let trimmed = line.trim();
    trimmed.len() >= len
        && trimmed
            .as_bytes()
            .iter()
            .take(len)
            .all(|byte| *byte == fence)
}

fn strip_fence_indent(line: &str, indent: usize) -> String {
    let mut removed = 0;
    let mut index = 0;
    for ch in line.chars() {
        if removed >= indent || ch != ' ' {
            break;
        }
        removed += 1;
        index += ch.len_utf8();
    }
    line[index..].to_string()
}

fn is_table_start(lines: &[&str], index: usize) -> bool {
    if index + 1 >= lines.len() || !lines[index].contains('|') {
        return false;
    }
    let separator = lines[index + 1].trim();
    if !separator.contains('|') || !separator.contains('-') {
        return false;
    }
    let cells = split_table_row(separator);
    !cells.is_empty()
        && cells.iter().all(|cell| {
            let cell = cell.trim();
            let cell = cell.strip_prefix(':').unwrap_or(cell);
            let cell = cell.strip_suffix(':').unwrap_or(cell);
            cell.len() >= 3 && cell.chars().all(|ch| ch == '-')
        })
}

fn split_table_row(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut chars = trimmed.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next) = chars.next() {
                current.push(next);
            }
            continue;
        }
        if ch == '|' {
            cells.push(current.trim().to_string());
            current.clear();
        } else {
            current.push(ch);
        }
    }
    cells.push(current.trim().to_string());
    if trimmed.starts_with('|') && cells.first().map(|cell| cell.is_empty()).unwrap_or(false) {
        cells.remove(0);
    }
    if trimmed.ends_with('|') && cells.last().map(|cell| cell.is_empty()).unwrap_or(false) {
        cells.pop();
    }
    cells
}

fn parse_table_alignments(line: &str) -> Vec<TableAlign> {
    split_table_row(line)
        .into_iter()
        .map(|cell| {
            let cell = cell.trim();
            match (cell.starts_with(':'), cell.ends_with(':')) {
                (true, true) => TableAlign::Center,
                (false, true) => TableAlign::Right,
                _ => TableAlign::Left,
            }
        })
        .collect()
}

fn is_thematic_break(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.len() < 3 {
        return false;
    }
    let mut marker = None;
    let mut count = 0;
    for ch in trimmed.chars() {
        if ch == ' ' || ch == '\t' {
            continue;
        }
        if !matches!(ch, '-' | '*' | '_') {
            return false;
        }
        if marker.map(|value| value != ch).unwrap_or(false) {
            return false;
        }
        marker = Some(ch);
        count += 1;
    }
    count >= 3
}

struct ListItem<'a> {
    indent: usize,
    marker: String,
    content: &'a str,
}

fn parse_list_item(line: &str) -> Option<ListItem<'_>> {
    let leading = line.len() - line.trim_start().len();
    let trimmed = &line[leading..];
    let bytes = trimmed.as_bytes();
    if bytes.len() >= 2 && matches!(bytes[0], b'-' | b'*' | b'+') && bytes[1].is_ascii_whitespace()
    {
        return Some(ListItem {
            indent: leading,
            marker: if matches!(bytes[0], b'-' | b'*') {
                "• ".to_string()
            } else {
                "◦ ".to_string()
            },
            content: trimmed[2..].trim_start(),
        });
    }
    let digits = bytes
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    if digits > 0 && digits < 10 && digits < bytes.len() {
        let marker_end = if bytes[digits] == b'.' || bytes[digits] == b')' {
            digits + 1
        } else {
            return None;
        };
        if marker_end < bytes.len() && !bytes[marker_end].is_ascii_whitespace() {
            return None;
        }
        return Some(ListItem {
            indent: leading,
            marker: format!("{}. ", &trimmed[..digits]),
            content: trimmed[marker_end..].trim_start(),
        });
    }
    None
}

fn is_block_quote(line: &str) -> bool {
    line.trim_start().starts_with('>')
}

fn is_indented_code(line: &str) -> bool {
    line.starts_with("    ") || line.starts_with('\t')
}

fn strip_indent(line: &str, spaces: usize) -> &str {
    let mut count = 0;
    let mut index = 0;
    for ch in line.chars() {
        if count >= spaces {
            break;
        }
        if ch == ' ' {
            count += 1;
            index += 1;
        } else if ch == '\t' {
            count = spaces;
            index += 1;
            break;
        } else {
            break;
        }
    }
    &line[index..]
}

fn starts_block(lines: &[&str], index: usize) -> bool {
    let line = lines[index];
    if line.trim().is_empty()
        || parse_atx_heading(line).is_some()
        || parse_fence(line).is_some()
        || is_block_quote(line)
        || parse_list_item(line).is_some()
        || is_thematic_break(line)
        || is_indented_code(line)
        || parse_reference_definition(line).is_some()
        || is_table_start(lines, index)
    {
        return true;
    }
    index + 1 < lines.len() && is_setext_underline(lines[index + 1]) && !line.contains('|')
}

fn count_byte_run(bytes: &[u8], mut index: usize, byte: u8) -> usize {
    let start = index;
    while index < bytes.len() && bytes[index] == byte {
        index += 1;
    }
    index - start
}

fn display_width(text: &str) -> usize {
    text.chars()
        .map(|ch| if ch.is_ascii() { 1 } else { 2 })
        .sum()
}

fn is_ascii_punctuation(byte: u8) -> bool {
    byte.is_ascii_punctuation()
}

fn utf8_len(first: u8) -> usize {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_tables_and_unicode() {
        let source =
            "# 标题\n\n| 名称 | 值 |\n|:---|---:|\n| 内存 | 很小 |\n\n```rust\nfn main() {}\n```\n";
        let rendered = render(source, crate::theme::DARK, 10.5, 9000);
        assert!(rendered.has_tables);
        assert!(rendered.rtf.contains("\\trowd"));
        assert!(rendered.rtf.contains("\\u"));
    }

    #[test]
    fn parses_link_definitions() {
        let defs = collect_link_definitions("[OpenAI]: https://openai.com\n");
        assert_eq!(defs.get("openai").unwrap(), "https://openai.com");
    }

    #[test]
    fn detects_markdown_table_start() {
        let lines = vec!["| a | b |", "|---|---|", "| 1 | 2 |"];
        assert!(is_table_start(&lines, 0));
    }

    #[test]
    fn collects_visible_heading_text() {
        let source =
            "# 入门\n\n## **安装** `native`\n\n## [官网](https://example.com) 标题\n\nSetext 标题\n-------------\n";
        let rendered = render(source, crate::theme::DARK, 10.5, 9000);
        assert_eq!(
            rendered.headings,
            vec![
                MarkdownHeading {
                    level: 1,
                    text: "入门".to_string(),
                },
                MarkdownHeading {
                    level: 2,
                    text: "安装 native".to_string(),
                },
                MarkdownHeading {
                    level: 2,
                    text: "官网 标题".to_string(),
                },
                MarkdownHeading {
                    level: 2,
                    text: "Setext 标题".to_string(),
                },
            ]
        );
        assert!(rendered.rtf.contains("\\v \\u-8192?"));
    }
}
