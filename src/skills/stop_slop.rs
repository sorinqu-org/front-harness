use regex::Regex;

pub struct StopSlopValidator;

impl StopSlopValidator {
    pub fn contains_unicode_emoji(text: &str) -> bool {
        for c in text.chars() {
            let u = c as u32;
            if (0x1F600..=0x1F64F).contains(&u)
                || (0x1F300..=0x1F5FF).contains(&u)
                || (0x1F680..=0x1F6FF).contains(&u)
                || (0x1F700..=0x1F77F).contains(&u)
                || (0x1F780..=0x1F7FF).contains(&u)
                || (0x1F800..=0x1F8FF).contains(&u)
                || (0x1F900..=0x1F9FF).contains(&u)
                || (0x1FA00..=0x1FA6F).contains(&u)
                || (0x1FA70..=0x1FAFF).contains(&u)
                || (0x2600..=0x26FF).contains(&u)
                || (0x2700..=0x27BF).contains(&u)
            {
                return true;
            }
        }
        false
    }

    pub fn audit_code(code: &str) -> Vec<String> {
        let mut warnings = Vec::new();

        if Self::contains_unicode_emoji(code) {
            warnings.push("Violation: Unicode emojis detected in code. Replace with vector SVG icons.".to_string());
        }

        let purple_pattern = Regex::new(r"(?i)(from-purple|to-purple|from-violet|to-violet|bg-purple|bg-violet)").unwrap();
        if purple_pattern.is_match(code) {
            warnings.push("Violation: Detected generic AI-purple gradient. Use calibrated brand accents.".to_string());
        }

        let fake_metric_pattern = Regex::new(r"(?i)(\+47%|\+85%|trusted by 50,000|10x faster)").unwrap();
        if fake_metric_pattern.is_match(code) {
            warnings.push("Warning: Detected potential fabricated metric. Use verified client data.".to_string());
        }

        warnings
    }

    pub fn clean_slop(text: &str) -> String {
        let mut result = text.to_string();
        // Strip common filler phrases
        let fillers = [
            "In today's fast-paced digital world, ",
            "Without further ado, ",
            "Here is the breakdown: ",
            "It's important to remember that ",
            "Delve into ",
        ];
        for f in fillers {
            result = result.replace(f, "");
        }
        result
    }
}
