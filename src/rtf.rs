use crate::theme::{Color, Theme};

/// Small RTF writer. RichEdit is the only renderer, so the subset here is
/// deliberately limited to what the native control understands.
pub struct RtfDoc {
    body: String,
    colors: Vec<Color>,
    pub theme: Theme,
}

impl RtfDoc {
    pub fn new(theme: Theme) -> Self {
        Self {
            body: String::new(),
            colors: Vec::new(),
            theme,
        }
    }

    pub fn color(&mut self, color: Color) -> usize {
        if let Some(index) = self.colors.iter().position(|c| *c == color) {
            return index + 1;
        }
        self.colors.push(color);
        self.colors.len()
    }

    pub fn raw(&mut self, value: &str) {
        self.body.push_str(value);
    }

    pub fn text(&mut self, value: &str) {
        escape_into(&mut self.body, value);
    }

    pub fn text_with_color(&mut self, value: &str, color: Color) {
        let index = self.color(color);
        self.body.push_str("\\cf");
        self.body.push_str(&index.to_string());
        self.body.push(' ');
        escape_into(&mut self.body, value);
    }

    pub fn finish(self) -> String {
        let mut out = String::with_capacity(self.body.len() + 512);
        out.push_str("{\\rtf1\\ansi\\ansicpg1252\\deff0\\uc1");
        out.push_str("{\\fonttbl{\\f0\\fnil\\fcharset0 Segoe UI;}{\\f1\\fmodern\\fcharset0 Consolas;}{\\f2\\fnil\\fcharset0 Microsoft YaHei UI;}{\\f3\\fnil\\fcharset0 Segoe MDL2 Assets;}}");
        out.push_str("{\\colortbl;");
        for color in &self.colors {
            out.push_str("\\red");
            out.push_str(&color.r.to_string());
            out.push_str("\\green");
            out.push_str(&color.g.to_string());
            out.push_str("\\blue");
            out.push_str(&color.b.to_string());
            out.push(';');
        }
        out.push('}');
        out.push_str("\\viewkind4\\uc1\\pard\\f2\\fs21\\cf1 ");
        out.push_str(&self.body);
        out.push('}');
        out
    }
}

pub fn escape_into(out: &mut String, value: &str) {
    for unit in value.encode_utf16() {
        match unit {
            0x09 => out.push_str("\\tab "),
            0x0A => out.push_str("\\line "),
            0x0D => {}
            0x5C => out.push_str("\\\\"),
            0x7B => out.push_str("\\{"),
            0x7D => out.push_str("\\}"),
            0x20..=0x7E => out.push(unit as u8 as char),
            _ => {
                out.push_str("\\u");
                out.push_str(&(unit as i16).to_string());
                out.push('?');
            }
        }
    }
}

pub fn escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    escape_into(&mut out, value);
    out
}
