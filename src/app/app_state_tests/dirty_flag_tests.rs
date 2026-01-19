use crate::test_utils::test_helpers::test_app;

#[test]
fn test_initial_state_needs_render_is_true() {
    let app = test_app(r#"{"test": true}"#);
    assert!(app.needs_render, "New app should need initial render");
}

#[test]
fn test_mark_dirty_sets_flag() {
    let mut app = test_app(r#"{"test": true}"#);
    app.needs_render = false;

    app.mark_dirty();

    assert!(
        app.needs_render,
        "mark_dirty should set needs_render to true"
    );
}

#[test]
fn test_clear_dirty_clears_flag() {
    let mut app = test_app(r#"{"test": true}"#);
    app.needs_render = true;

    app.clear_dirty();

    assert!(
        !app.needs_render,
        "clear_dirty should set needs_render to false"
    );
}

#[test]
fn test_should_render_when_dirty() {
    let mut app = test_app(r#"{"test": true}"#);
    app.needs_render = true;

    assert!(
        app.should_render(),
        "should_render should return true when dirty"
    );
}

#[test]
fn test_should_render_false_when_clean_no_animation() {
    let mut app = test_app(r#"{"test": true}"#);
    app.clear_dirty();

    app.query.as_mut().unwrap().cancel_in_flight();
    app.ai.loading = false;
    app.file_loader = None;
    app.notification.dismiss();

    assert!(
        !app.should_render(),
        "should_render should return false when clean and no animations"
    );
}

#[test]
fn test_needs_animation_with_pending_query() {
    let mut app = test_app(r#"{"test": true}"#);
    app.clear_dirty();

    if let Some(query) = app.query.as_mut() {
        query.execute_async(".");
        assert!(
            app.should_render(),
            "should_render should return true when query is pending (animation)"
        );
    }
}

#[test]
fn test_needs_animation_with_ai_loading() {
    let mut app = test_app(r#"{"test": true}"#);
    app.clear_dirty();

    app.ai.loading = true;

    assert!(
        app.should_render(),
        "should_render should return true when AI is loading (animation)"
    );
}

#[test]
fn test_needs_animation_with_file_loading() {
    let config = crate::config::Config::default();
    let loader = crate::input::FileLoader::spawn_load(std::path::PathBuf::from("/nonexistent"));
    let mut app = crate::app::app_state::App::new_with_loader(loader, &config);
    app.clear_dirty();

    assert!(
        app.should_render(),
        "should_render should return true when file is loading (animation)"
    );
}

#[test]
fn test_needs_animation_with_notification() {
    let mut app = test_app(r#"{"test": true}"#);
    app.clear_dirty();

    app.notification.show("Test notification");

    assert!(
        app.should_render(),
        "should_render should return true when notification is active (timer expiry check)"
    );
}

#[test]
fn test_needs_animation_false_when_idle() {
    let mut app = test_app(r#"{"test": true}"#);
    app.clear_dirty();

    if let Some(query) = app.query.as_mut() {
        query.cancel_in_flight();
    }
    app.ai.loading = false;
    app.file_loader = None;
    app.notification.dismiss();

    assert!(
        !app.should_render(),
        "should_render should return false when idle"
    );
}

#[test]
fn test_mark_dirty_idempotent() {
    let mut app = test_app(r#"{"test": true}"#);
    app.clear_dirty();

    app.mark_dirty();
    app.mark_dirty();
    app.mark_dirty();

    assert!(
        app.needs_render,
        "Multiple mark_dirty calls should still result in needs_render=true"
    );
}

#[test]
fn test_should_render_with_dirty() {
    let mut app = test_app(r#"{"test": true}"#);
    app.needs_render = true;
    app.ai.loading = false;
    app.query.as_mut().unwrap().cancel_in_flight();
    app.file_loader = None;
    app.notification.dismiss();

    assert!(
        app.should_render(),
        "should_render should return true when dirty"
    );
}

#[test]
fn test_should_render_with_ai_loading() {
    let mut app = test_app(r#"{"test": true}"#);
    app.needs_render = false;
    app.ai.loading = true;
    app.query.as_mut().unwrap().cancel_in_flight();
    app.file_loader = None;
    app.notification.dismiss();

    assert!(
        app.should_render(),
        "should_render should return true when ai_loading"
    );
}

#[test]
fn test_clear_dirty_with_ai_loading() {
    let mut app = test_app(r#"{"test": true}"#);
    app.mark_dirty();
    app.clear_dirty();
    app.ai.loading = true;
    app.query.as_mut().unwrap().cancel_in_flight();
    app.file_loader = None;
    app.notification.dismiss();

    assert!(
        app.should_render(),
        "After clear_dirty with ai_loading, should_render should return true"
    );
}

#[test]
fn test_clear_dirty_without_ai_loading() {
    let mut app = test_app(r#"{"test": true}"#);
    app.mark_dirty();
    app.clear_dirty();
    app.ai.loading = false;
    app.query.as_mut().unwrap().cancel_in_flight();
    app.file_loader = None;
    app.notification.dismiss();

    assert!(
        !app.should_render(),
        "After clear_dirty without ai_loading, should_render should return false"
    );
}
