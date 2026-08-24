pub const MOTION_INSTRUCTIONS: &str = r#"
# Motion & Micro-interactions Directives
1. Motivated Motion: Every animation must communicate hierarchy, narrative flow, or tactile feedback.
2. GSAP & ScrollTrigger:
   - For pinned elements: start: "top top", pin: true.
   - For horizontal pans: scrub: 1, calculate distance from track width.
3. Spring Physics:
   - Hover and active states use spring physics (stiffness 120, damping 14).
4. Accessibility:
   - Wrap interactive transitions in @media (prefers-reduced-motion: no-preference).
   - Degrade to instant state swaps when reduced motion is requested.
"#;

pub fn get_motion_rules() -> &'static str {
    MOTION_INSTRUCTIONS
}
