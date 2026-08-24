use frontharness::skills::{IconEngine, StopSlopValidator};

#[test]
fn test_unicode_emoji_detection() {
    let slop_code = "<button>Click Me 🚀</button>";
    assert!(StopSlopValidator::contains_unicode_emoji(slop_code));

    let clean_code = "<button>Click Me</button>";
    assert!(!StopSlopValidator::contains_unicode_emoji(clean_code));
}

#[test]
fn test_stop_slop_code_audit() {
    let code_with_violations = r#"
        <div class="bg-gradient-to-r from-purple-500 to-indigo-600">
            <h1>Trusted by 50,000+ teams 🚀</h1>
        </div>
    "#;
    let warnings = StopSlopValidator::audit_code(code_with_violations);
    assert!(!warnings.is_empty());
    assert!(warnings.iter().any(|w| w.contains("Unicode emojis")));
    assert!(warnings.iter().any(|w| w.contains("AI-purple")));
}

#[test]
fn test_icon_engine_svg_generation() {
    let svg = IconEngine::get_svg_icon("phone", "w-5 h-5 text-amber-500");
    assert!(svg.contains("<svg"));
    assert!(svg.contains("viewBox=\"0 0 24 24\""));
    assert!(svg.contains("class=\"w-5 h-5 text-amber-500\""));
    assert!(!StopSlopValidator::contains_unicode_emoji(&svg));
}
