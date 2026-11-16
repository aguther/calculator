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

#[test]
fn portrait_tablets_use_the_available_width() {
    // iPad mini portrait, after the app's 10-point outer margins.
    let mini = layout(724.0, 1089.0);
    assert!(!mini.landscape);
    assert_eq!(mini.content_width, 720.0);

    let narrow_tablet = layout(600.0, 940.0);
    assert!(!narrow_tablet.landscape);
    assert_eq!(narrow_tablet.content_width, 600.0);

    let phone = layout(410.0, 880.0);
    assert_eq!(phone.content_width, 410.0);
    let desktop = layout(1200.0, 800.0);
    assert!(desktop.landscape);
    assert_eq!(desktop.content_width, 1100.0);
}
