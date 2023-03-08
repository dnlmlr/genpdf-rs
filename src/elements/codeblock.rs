use crate::{
    fonts,
    style::{Style, StyledStr},
    Element, Mm, Position, RenderResult, Size,
};

/// A Block of Code that is usually rendered using a monospace font and utilizes syntax highlighting
pub struct CodeBlock {
    code: String,
    base_style: Style,

    #[cfg(feature = "code-syntax-highlighting")]
    only_regular_font: bool,
    #[cfg(feature = "code-syntax-highlighting")]
    language: String,
    #[cfg(feature = "code-syntax-highlighting")]
    theme: Option<String>,
}

impl CodeBlock {
    /// Create a new Codeblock that renders the given Code
    #[cfg(not(feature = "code-syntax-highlighting"))]
    pub fn new(code: &str, base_style: Style) -> Self {
        let code = code.to_string();

        Self { code, base_style }
    }

    /// Create a new Codeblock that renders the given Code with the provided Theme, assuming the
    /// provided language
    #[cfg(feature = "code-syntax-highlighting")]
    pub fn new(code: &str, language: &str, theme: Option<&str>, base_style: Style) -> Self {
        let code = code.to_string();
        let language = language.to_string();
        let theme = theme.map(String::from);

        Self {
            code,
            base_style,
            only_regular_font: false,
            language,
            theme,
        }
    }

    fn dummy_highlighting(&self, style: Style) -> Vec<Vec<StyledStr<'_>>> {
        self.code
            .lines()
            .map(|line| vec![StyledStr::new(line, style)])
            .collect()
    }
}

impl Element for CodeBlock {
    fn render(
        &mut self,
        context: &crate::Context,
        mut area: crate::render::Area<'_>,
        _style: crate::style::Style,
    ) -> Result<crate::RenderResult, crate::error::Error> {
        let mut result = RenderResult::default();

        if self.code.is_empty() {
            return Ok(result);
        }

        let highlighted_lines;
        #[cfg(feature = "code-syntax-highlighting")]
        {
            if let Some(theme) = self.theme.as_ref() {
                highlighted_lines = context
                    .syntax_highlighter
                    .as_ref()
                    .expect("Trying to use Codeblocks without syntax highlighter")
                    .highlight(
                        &self.code,
                        &self.language,
                        theme,
                        self.base_style,
                        self.only_regular_font,
                    )
                    .unwrap_or_else(|| self.dummy_highlighting(self.base_style));
            } else {
                highlighted_lines = self.dummy_highlighting(self.base_style);
            }
        }
        #[cfg(not(feature = "code-syntax-highlighting"))]
        {
            highlighted_lines = self.dummy_highlighting(self.base_style);
        }

        let mut rendered_chars = 0;

        for line in highlighted_lines {
            let width: Mm = line.iter().map(|s| s.width(&context.font_cache)).sum();
            // Calculate the maximum line height
            let metrics = line
                .iter()
                .map(|s| s.style.metrics(&context.font_cache))
                .fold(fonts::Metrics::default(), |max, m| max.max(&m));

            if let Some(mut section) =
                area.text_section(&context.font_cache, Position::new(0, 0), metrics)
            {
                for s in line {
                    // Trim to remove end line character
                    section.print_str_xoff_trim(
                        &s.s.trim_end_matches('\n'),
                        s.style,
                        Mm(0.0),
                        false,
                    )?;
                    rendered_chars += s.s.chars().count();
                }
            } else {
                result.has_more = true;
                break;
            }
            result.size = result
                .size
                .stack_vertical(Size::new(width, metrics.line_height));
            area.add_offset(Position::new(0, metrics.line_height));
        }

        self.code.drain(..rendered_chars);

        Ok(result)
    }
}
