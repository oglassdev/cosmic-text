use cosmic_text::{fontdb, Attrs, Buffer, Family, FontSystem, LayoutGlyph, Metrics, Shaping};

const SMALL: Metrics = Metrics::new(16.0, 20.0);
const LARGE: Metrics = Metrics::new(80.0, 96.0);

fn font_system() -> FontSystem {
    let mut font_system =
        FontSystem::new_with_locale_and_db("en-US".into(), fontdb::Database::new());
    font_system
        .db_mut()
        .load_font_data(std::fs::read("fonts/NotoSans-Regular.ttf").unwrap());
    font_system
}

/// Shapes `spans` as one line of rich text and returns its glyphs and line height.
fn layout(font_system: &mut FontSystem, spans: &[(&str, Metrics)]) -> (Vec<LayoutGlyph>, f32) {
    let attrs = Attrs::new().family(Family::Name("Noto Sans"));
    let mut buffer = Buffer::new(font_system, SMALL);
    buffer.set_size(None, None);
    buffer.set_rich_text(
        spans
            .iter()
            .map(|&(text, metrics)| (text, attrs.clone().metrics(metrics))),
        &attrs,
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(font_system, true);
    let run = buffer.layout_runs().next().unwrap();
    (run.glyphs.to_vec(), run.line_height)
}

#[test]
fn ligature_is_not_formed_across_metrics() {
    let mut font_system = font_system();

    let (glyphs, _) = layout(&mut font_system, &[("fi", SMALL)]);
    assert_eq!(glyphs.len(), 1, "the font should ligate \"fi\"");

    let (glyphs, line_height) = layout(&mut font_system, &[("f", SMALL), ("i", LARGE)]);
    assert_eq!(glyphs.len(), 2);
    assert_eq!(glyphs[0].font_size, SMALL.font_size);
    assert_eq!(glyphs[1].font_size, LARGE.font_size);
    assert_eq!(line_height, LARGE.line_height);
}

#[test]
fn combining_mark_keeps_its_own_metrics() {
    let mut font_system = font_system();

    let (glyphs, _) = layout(&mut font_system, &[("e\u{301}", SMALL)]);
    assert_eq!(glyphs.len(), 1, "the font should compose \"e\" + U+0301");

    let (glyphs, line_height) = layout(&mut font_system, &[("e", SMALL), ("\u{301}", LARGE)]);
    assert_eq!(glyphs.len(), 2);
    assert_eq!((glyphs[1].start, glyphs[1].end), (1, 3));
    assert_eq!(glyphs[0].font_size, SMALL.font_size);
    assert_eq!(glyphs[1].font_size, LARGE.font_size);
    assert_eq!(line_height, LARGE.line_height);
}

#[test]
fn kerning_is_not_applied_across_metrics() {
    let mut font_system = font_system();

    let width = |glyphs: &[LayoutGlyph]| glyphs.iter().map(|g| g.w).sum::<f32>();
    let (a, _) = layout(&mut font_system, &[("A", SMALL)]);
    let (v, _) = layout(&mut font_system, &[("V", LARGE)]);
    let (kerned, _) = layout(&mut font_system, &[("AV", SMALL)]);
    let (split, _) = layout(&mut font_system, &[("A", SMALL), ("V", LARGE)]);

    let a_small = width(&a);
    assert!(
        width(&kerned) < a_small + width(&layout(&mut font_system, &[("V", SMALL)]).0),
        "the font should kern \"AV\""
    );
    assert_eq!(width(&split), a_small + width(&v));
}
