//! Provides syntax highlighting support for code blocks

use syntect::{
    easy::HighlightLines,
    highlighting::{FontStyle, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

use crate::style::{Color, Style, StyledStr};

/// The SyntaxHighlighter is used to create styled string segments from the given input text
#[derive(Debug)]
pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl SyntaxHighlighter {
    /// Load the default language support and themes.
    pub fn load_defaults() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();

        Self {
            syntax_set,
            theme_set,
        }
    }

    /// Highlight the provided code with syntax for the specified language and using the provided
    /// theme. The `base_style` is used to provide the font size and font family.
    pub fn highlight<'a>(
        &self,
        code: &'a str,
        language: &str,
        theme: &str,
        base_style: Style,
        only_regular_font: bool,
    ) -> Option<Vec<Vec<StyledStr<'a>>>> {
        let syntax = self.syntax_set.find_syntax_by_token(&language)?;

        let mut h = HighlightLines::new(syntax, self.theme_set.themes.get(theme)?);

        let lines = LinesWithEndings::from(&code)
            .map(|line| {
                h.highlight_line(line, &self.syntax_set)
                    .unwrap()
                    .into_iter()
                    .map(|(syntax_style, code_segment)| {
                        let color = Color::Rgb(
                            syntax_style.foreground.r,
                            syntax_style.foreground.g,
                            syntax_style.foreground.b,
                        );

                        let mut style = base_style.clone().with_color(color);

                        if !only_regular_font {
                            let bold = syntax_style.font_style.contains(FontStyle::BOLD);
                            let italic = syntax_style.font_style.contains(FontStyle::ITALIC);
                            if bold {
                                style.set_bold();
                            }
                            if italic {
                                style.set_italic();
                            }
                        }

                        StyledStr::new(code_segment, style)
                    })
                    .collect()
            })
            .collect();

        Some(lines)
    }
}
