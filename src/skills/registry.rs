use crate::skills::hallmark::get_hallmark_rules;
use crate::skills::motion::get_motion_rules;
use crate::skills::taste::get_taste_rules;

#[derive(Debug, Clone)]
pub struct SkillItem {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub enabled: bool,
}

pub struct SkillRegistry;

impl SkillRegistry {
    pub fn default_skills() -> Vec<SkillItem> {
        vec![
            SkillItem {
                id: "hallmark",
                name: "Hallmark Macrostructures",
                description: "Structural variety: Bento Grid, Workbench, Split Screen, Anti-template",
                enabled: true,
            },
            SkillItem {
                id: "taste",
                name: "Taste & Contrast Dials",
                description: "Variance/Motion/Density dials, single accent palette, strict contrast",
                enabled: true,
            },
            SkillItem {
                id: "stop_slop",
                name: "Stop-Slop Linter & Emoji Shield",
                description: "Blocks generic AI gradients, fake metrics, strictly zero Unicode emojis",
                enabled: true,
            },
            SkillItem {
                id: "motion",
                name: "Motion & Physics Engine",
                description: "GSAP 3.12 ScrollTrigger, Lenis smooth scroll, spring hover states",
                enabled: true,
            },
            SkillItem {
                id: "icons",
                name: "Lucide Vector SVG Engine",
                description: "Embedded vector SVG icons, zero raster/unicode iconography",
                enabled: true,
            },
            SkillItem {
                id: "modern_web",
                name: "Modern Web Guidance",
                description: "CSS Grid/Flex, container queries, clamp() scaling, responsive layout",
                enabled: true,
            },
            SkillItem {
                id: "security",
                name: "Cybersecurity Hardening",
                description: "XSS prevention, CSP directives, secure form attributes",
                enabled: true,
            },
        ]
    }

    pub fn get_combined_system_prompt(role_prompt: &str) -> String {
        format!(
            "{}\n\n{}\n\n{}\n\n{}",
            role_prompt,
            get_hallmark_rules(),
            get_taste_rules(),
            get_motion_rules()
        )
    }

    pub fn build_custom_system_prompt(
        role_prompt: &str,
        enabled_skills: &[String],
        design_style: &str,
        references: &[String],
    ) -> String {
        let mut prompt = role_prompt.to_string();

        if !design_style.is_empty() {
            prompt.push_str(&format!(
                "\n\n### EXPLICIT USER DESIGN STYLE DIRECTIVES:\nYou MUST strictly adhere to the following user-defined aesthetic and styling guidelines:\n{}\n",
                design_style
            ));
        }

        if !references.is_empty() {
            prompt.push_str("\n\n### ATTACHED DESIGN REFERENCES & INSPIRATION:\nIncorporate structural and visual inspiration from the following references:\n");
            for r in references {
                prompt.push_str(&format!("- Reference: {}\n", r));
            }
        }

        prompt.push_str("\n\n### ACTIVE SKILLS MATRIX & ENFORCED DIRECTIVES:\n");

        for skill_id in enabled_skills {
            match skill_id.as_str() {
                "hallmark" => {
                    prompt.push_str("\n--- SKILL: HALLMARK MACROSTRUCTURES ---\n");
                    prompt.push_str(get_hallmark_rules());
                }
                "taste" => {
                    prompt.push_str("\n--- SKILL: TASTE & CONTRAST DIALS ---\n");
                    prompt.push_str(get_taste_rules());
                }
                "stop_slop" => {
                    prompt.push_str("\n--- SKILL: STOP-SLOP CODE AUDIT ---\n");
                    prompt.push_str("- STRICT ZERO UNICODE EMOJIS: Never use emojis in UI markup.\n- No generic AI-purple gradients (#8A2BE2 -> #FF1493).\n- No fake stats/counters without real data backing.\n");
                }
                "motion" => {
                    prompt.push_str("\n--- SKILL: MOTION & GSAP PHYSICS ---\n");
                    prompt.push_str(get_motion_rules());
                }
                "icons" => {
                    prompt.push_str("\n--- SKILL: LUCIDE VECTOR ICONS ---\n");
                    prompt.push_str("- Always inline clean vector <svg> elements for icons (viewBox='0 0 24 24', stroke='currentColor', stroke-width='2').\n");
                }
                "modern_web" => {
                    prompt.push_str("\n--- SKILL: MODERN WEB GUIDANCE ---\n");
                    prompt.push_str("- Use modern CSS features: flexbox, CSS grid, clamp() for responsive typography, backdrop-filter, smooth scrolling.\n");
                }
                "security" => {
                    prompt.push_str("\n--- SKILL: CYBERSECURITY HARDENING ---\n");
                    prompt.push_str("- Sanitize all form inputs, use autocomplete attributes, protect against XSS injection.\n");
                }
                _ => {}
            }
        }

        prompt
    }
}
