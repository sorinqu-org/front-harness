pub const HALLMARK_INSTRUCTIONS: &str = r#"
# Hallmark Skill Directives
1. Structural Variety: Never output the generic centered-hero -> 3-feature -> CTA rhythm. Pick a distinct macrostructure:
   - Bento Grid (varied cell sizes, mixed image/stat cells)
   - Marquee Hero (high typographic impact, lateral rhythm)
   - Workbench (split tool interface, tactile cards)
   - Long Document (editorial layout, generous whitespace, margin notes)
   - Split Screen (asymmetric 50/50 with sticky visual rail)
2. Honest Copy: Never fabricate numbers or metrics ('+47% conversion', '50k+ teams'). Use real content or clean functional labels.
3. Locked Tokens: Use named CSS variables for all colors and fonts. No inline improvised OKLCH or hex colors.
4. Mobile First: Validate at 375px and 1920px. Ensure overflow-x is clipped, touch targets are minimum 44px, and nav collapses seamlessly.
5. Strict Typography: Use clean roman display fonts (Geist, Space Grotesk, Cabinet Grotesk). Never mix random serif words into sans headlines.
"#;

pub fn get_hallmark_rules() -> &'static str {
    HALLMARK_INSTRUCTIONS
}
