use font::OpenTypeFont;
use genpdf::{
    elements::{self, MathElement},
    fonts::FontFamily,
    math::MathBlock,
};
use rex::{
    font::FontContext,
    layout::{engine, Layout, LayoutSettings},
    parser, Renderer,
};

const MATH_EXAMPLE: &'static str = r"\frac{1}{\left(\sqrt{\phi\sqrt5} - \phi\right) e^{\frac{2}{5}\pi}} = 1 + \frac{e^{-2\pi}}{1 + \frac{e^{-4\pi}}{1 + \frac{e^{-6\pi}}{1 + \frac{e^{-8\pi}}{1 + \unicodecdots}}}}";

fn main() {
    let xits_font = include_bytes!("./fonts/rex-xits.ttf");

    let pdf_font = genpdf::fonts::FontData::new(xits_font.to_vec(), None).unwrap();
    let pdf_font_family = FontFamily {
        regular: pdf_font.clone(),
        bold: pdf_font.clone(),
        italic: pdf_font.clone(),
        bold_italic: pdf_font,
    };

    let mut pdf_doc = genpdf::Document::new(pdf_font_family);
    pdf_doc.set_title("genpdf+rex Demo Document");
    pdf_doc.set_minimal_conformance();
    pdf_doc.set_line_spacing(1.25);
    pdf_doc.push(elements::Text::new("Math with Rex and genpdf"));

    let rex_font = font::parse(xits_font)
        .unwrap()
        .downcast_box::<OpenTypeFont>();

    if let Ok(rex_font) = rex_font {
        let rex_font_ctx = FontContext::new(&rex_font);
        let rex_layout_settings =
            LayoutSettings::new(&rex_font_ctx, 10.0, rex::layout::Style::Display);
        let rex_ast = parser::parse(MATH_EXAMPLE).unwrap();
        let rex_layout = engine::layout(&rex_ast, rex_layout_settings).unwrap();

        let rex_renderer = Renderer::new();

        let mut math_block = MathBlock::new();
        rex_renderer.render(&rex_layout, &mut math_block);

        pdf_doc.push(MathElement::new(math_block));

        pdf_doc
            .render_to_file("./math.pdf")
            .expect("Failed to write output file");
    }
}
