pub const TASTE_INSTRUCTIONS: &str = r#"
# Design Taste & Anti-Slop Directives
1. Dials Configuration:
   - DESIGN_VARIANCE: 8 (Asymmetric whitespace, non-standard layouts)
   - MOTION_INTENSITY: 6 (Motivated scroll-reveals, spring hover feedback)
   - VISUAL_DENSITY: 4 (Airy, clean editorial hierarchy)
2. Color Calibration:
   - Maximum 1 saturated accent color.
   - Forbid AI-purple/neon gradients on black backgrounds.
   - Lock color consistency across all sections.
3. Layout & Hero Discipline:
   - Hero must fit in viewport (min-h-[100dvh] or max-h-screen).
   - Headline maximum 2 lines on desktop.
   - Single line navigation on desktop (height 64-72px).
   - Eyebrow restraint: maximum 1 uppercase tracking label per 3 sections.
4. Interactive States:
   - Support default, hover, active (-translate-y-[1px]), focus-visible, disabled, loading.
   - Button text must never wrap on desktop.
   - No duplicate CTA intent across the page.
"#;

pub fn get_taste_rules() -> &'static str {
    TASTE_INSTRUCTIONS
}
