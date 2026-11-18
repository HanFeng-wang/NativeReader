#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub dark: bool,
    pub bg: Color,
    pub panel: Color,
    pub hover: Color,
    pub accent: Color,
    pub fg: Color,
    pub muted: Color,
    pub heading: Color,
    pub link: Color,
    pub code_bg: Color,
    pub code_header_bg: Color,
    pub code_fg: Color,
    pub border: Color,
    pub table_header_bg: Color,
    pub table_alt_bg: Color,
    pub keyword: Color,
    pub string: Color,
    pub comment: Color,
    pub number: Color,
    pub type_name: Color,
    pub function: Color,
    pub attribute: Color,
}

pub const LIGHT: Theme = Theme {
    dark: false,
    bg: Color::rgb(255, 255, 255),
    panel: Color::rgb(247, 248, 250),
    hover: Color::rgb(235, 238, 242),
    accent: Color::rgb(9, 105, 218),
    fg: Color::rgb(31, 35, 40),
    muted: Color::rgb(96, 103, 112),
    heading: Color::rgb(28, 33, 40),
    link: Color::rgb(9, 105, 218),
    code_bg: Color::rgb(238, 242, 246),
    code_header_bg: Color::rgb(226, 232, 240),
    code_fg: Color::rgb(36, 41, 47),
    border: Color::rgb(201, 209, 217),
    table_header_bg: Color::rgb(246, 248, 250),
    table_alt_bg: Color::rgb(252, 252, 252),
    keyword: Color::rgb(0, 0, 255),
    string: Color::rgb(163, 21, 21),
    comment: Color::rgb(0, 128, 0),
    number: Color::rgb(9, 134, 88),
    type_name: Color::rgb(38, 127, 153),
    function: Color::rgb(121, 94, 38),
    attribute: Color::rgb(129, 80, 0),
};

pub const DARK: Theme = Theme {
    dark: true,
    bg: Color::rgb(24, 25, 27),
    panel: Color::rgb(31, 32, 35),
    hover: Color::rgb(45, 47, 51),
    accent: Color::rgb(0, 122, 204),
    fg: Color::rgb(228, 232, 237),
    muted: Color::rgb(157, 163, 172),
    heading: Color::rgb(240, 243, 247),
    link: Color::rgb(92, 175, 255),
    code_bg: Color::rgb(31, 34, 39),
    code_header_bg: Color::rgb(43, 47, 54),
    code_fg: Color::rgb(218, 222, 227),
    border: Color::rgb(62, 69, 80),
    table_header_bg: Color::rgb(38, 40, 45),
    table_alt_bg: Color::rgb(28, 30, 33),
    keyword: Color::rgb(197, 134, 192),
    string: Color::rgb(206, 145, 120),
    comment: Color::rgb(106, 153, 85),
    number: Color::rgb(181, 206, 168),
    type_name: Color::rgb(78, 201, 176),
    function: Color::rgb(220, 220, 170),
    attribute: Color::rgb(156, 220, 254),
};

pub fn for_mode(dark: bool) -> Theme {
    if dark {
        DARK
    } else {
        LIGHT
    }
}
