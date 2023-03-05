use genpdf::{
    elements,
    fonts::{FontData, FontFamily},
};

const MATH_EXAMPLE: &'static str = r"
\mathop{\overbrace{1+2+3+\unicodecdots+n}}\limits^{\mathrm{Arithmatic}} = \frac{n(n+1)}{2}  \mid
\color{red} \frac{1}{\left(\sqrt{\phi\sqrt5} - \phi\right) e^{\frac{2}{5}\pi}} = 1 + \frac{e^{-2\pi}}{1 + \frac{e^{-4\pi}}{1 + \frac{e^{-6\pi}}{1 + \frac{e^{-8\pi}}{1 + \unicodecdots}}}}  \mid
\left\vert\sum_k a_k b_k\right\vert \leq \left(\sum_k a_k^2\right)^{\frac12}\left(\sum_k b_k^2\right)^{\frac12}
\mathop{\mathrm{lim\,sup}}\limits_{x\rightarrow\infty}\ \mathop{\mathrm{sin}}(x)\mathrel{\mathop{=}\limits^?}1
f^{(n)}(z) = \frac{n!}{2\pi i} \oint \frac{f(\xi)}{(\xi - z)^{n+1}}\,\mathrm{d}\xi
";

fn make_font_family(data: &[u8]) -> FontFamily<FontData> {
    let font = genpdf::fonts::FontData::new(data.to_vec(), None).unwrap();
    FontFamily {
        regular: font.clone(),
        bold: font.clone(),
        italic: font.clone(),
        bold_italic: font,
    }
}

fn main() {
    let text_font_data = include_bytes!("./fonts/open-sans.ttf");
    let math_font_data = include_bytes!("./fonts/rex-xits.ttf");

    let text_font_family = make_font_family(text_font_data);
    let math_font_family = make_font_family(math_font_data);

    let mut pdf_doc = genpdf::Document::new(text_font_family);
    pdf_doc.set_title("genpdf+rex Demo Document");
    pdf_doc.set_minimal_conformance();
    pdf_doc.set_line_spacing(1.25);
    pdf_doc.set_font_size(30);

    let math_font_family = pdf_doc.add_font_family(math_font_family);
    pdf_doc.enable_math(math_font_data, math_font_family);

    pdf_doc.push(elements::Text::new("Math with Rex and genpdf"));
    pdf_doc.push(elements::Math::new(MATH_EXAMPLE));
    pdf_doc.push(elements::Text::new("End of test!"));

    pdf_doc
        .render_to_file("./math.pdf")
        .expect("Failed to write output file");
}
