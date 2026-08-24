use crate::skills::hallmark::get_hallmark_rules;
use crate::skills::motion::get_motion_rules;
use crate::skills::taste::get_taste_rules;

pub struct SkillRegistry;

impl SkillRegistry {
    pub fn get_combined_system_prompt(role_prompt: &str) -> String {
        format!(
            "{}\n\n{}\n\n{}\n\n{}",
            role_prompt,
            get_hallmark_rules(),
            get_taste_rules(),
            get_motion_rules()
        )
    }
}
