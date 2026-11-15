use calculator::{Theme, layout};
#[test]
fn themes_are_distinct_and_persistable() {
    let themes = [Theme::Original, Theme::Light, Theme::Dark];
    assert_ne!(themes[0].background(), themes[1].background());
    assert_ne!(themes[0].background(), themes[2].background());
    for theme in themes {
        let text = serde_json::to_string(&theme).unwrap();
        assert_eq!(serde_json::from_str::<Theme>(&text).unwrap(), theme);
    }
}
#[test]
fn layout_fits_compact_and_landscape() {
    for (w, h) in [
        (320.0, 568.0),
        (400.0, 760.0),
        (568.0, 320.0),
        (844.0, 390.0),
    ] {
        let l = layout(w, h);
        assert!(l.key_height >= 28.0);
        assert!(l.tape_height >= 30.0);
        assert!(l.key_height * 5.0 + 24.0 <= h);
        assert!(l.content_width <= w);
    }
}
