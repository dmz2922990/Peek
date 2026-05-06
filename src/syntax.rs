use std::path::Path;

use ratatui::style::{Color, Modifier, Style};
use syntect::easy::HighlightLines;
use syntect::highlighting::{
    Color as SynColor, FontStyle, StyleModifier, Theme, ThemeItem, ThemeSettings,
};
use syntect::parsing::SyntaxSet;

use crate::ui::theme;

static SYNTAX_SET: std::sync::OnceLock<SyntaxSet> = std::sync::OnceLock::new();
static PEEK_THEME: std::sync::OnceLock<Theme> = std::sync::OnceLock::new();

pub struct SyntaxHighlighter {
    highlighter: Option<HighlightLines<'static>>,
}

impl SyntaxHighlighter {
    pub fn new(file_path: &Path) -> Self {
        let ss = SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines);
        let theme = PEEK_THEME.get_or_init(peek_theme);

        let syntax = ss
            .find_syntax_for_file(file_path)
            .ok()
            .flatten()
            .or_else(|| {
                file_path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .and_then(|ext| ss.find_syntax_by_extension(ext))
            });

        let highlighter = syntax.map(|syntax| {
            HighlightLines::new(syntax, theme)
        });

        Self { highlighter }
    }

    pub fn highlight_line(&mut self, line: &str) -> Vec<(Style, String)> {
        let ss = SYNTAX_SET.get().unwrap();
        if let Some(ref mut hl) = self.highlighter {
            if let Ok(ranges) = hl.highlight_line(line, ss) {
                let mut result: Vec<(Style, String)> = Vec::with_capacity(ranges.len());
                for (style, text) in ranges {
                    let ratatui_style = syntect_style_to_ratatui(style);
                    result.push((ratatui_style, text.to_string()));
                }
                return result;
            }
        }
        vec![(Style::default().fg(theme::TEXT_PRIMARY), line.to_string())]
    }
}

fn syntect_style_to_ratatui(style: syntect::highlighting::Style) -> Style {
    let fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
    let mut s = Style::default().fg(fg);
    if style.font_style.contains(FontStyle::BOLD) {
        s = s.add_modifier(Modifier::BOLD);
    }
    if style.font_style.contains(FontStyle::ITALIC) {
        s = s.add_modifier(Modifier::ITALIC);
    }
    if style.font_style.contains(FontStyle::UNDERLINE) {
        s = s.add_modifier(Modifier::UNDERLINED);
    }
    s
}

fn peek_theme() -> Theme {
    fn rgb(r: u8, g: u8, b: u8) -> SynColor {
        SynColor { r, g, b, a: 0xFF }
    }

    fn scope_item(scope: &str, color: SynColor, font_style: Option<FontStyle>) -> ThemeItem {
        ThemeItem {
            scope: scope.parse().unwrap(),
            style: StyleModifier {
                foreground: Some(color),
                background: None,
                font_style,
            },
        }
    }

    let primary = rgb(210, 210, 220);
    let secondary = rgb(120, 120, 140);
    let green = rgb(80, 200, 120);
    let magenta = rgb(180, 100, 220);
    let blue = rgb(100, 150, 230);
    let cyan = rgb(0, 190, 190);
    let yellow = rgb(220, 200, 80);

    Theme {
        name: Some("Peek".into()),
        author: Some(String::new()),
        settings: ThemeSettings {
            foreground: Some(primary),
            background: Some(rgb(0, 0, 0)),
            caret: Some(primary),
            ..Default::default()
        },
        scopes: vec![
            scope_item("comment", secondary, Some(FontStyle::ITALIC)),
            scope_item("string", green, None),
            scope_item("constant.character.escape", yellow, None),
            scope_item("keyword", magenta, None),
            scope_item("keyword.operator", primary, None),
            scope_item("keyword.control", magenta, Some(FontStyle::BOLD)),
            scope_item("storage", magenta, None),
            scope_item("entity.name.function", blue, None),
            scope_item("entity.name.type", cyan, None),
            scope_item("support.function", blue, None),
            scope_item("support.type", cyan, None),
            scope_item("support.class", cyan, None),
            scope_item("variable", primary, None),
            scope_item("variable.parameter", primary, None),
            scope_item("constant.numeric", yellow, None),
            scope_item("constant.language", yellow, None),
            scope_item("entity.other.attribute-name", cyan, None),
            scope_item("meta.tag", primary, None),
            scope_item("punctuation", primary, None),
            scope_item("punctuation.definition.comment", secondary, None),
        ],
    }
}
