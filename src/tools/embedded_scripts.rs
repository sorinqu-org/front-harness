use std::path::PathBuf;

pub const PLAYWRIGHT_CRAWLER_SCRIPT: &str = include_str!("../../helpers/playwright_crawler.py");
pub const TEMPLATE_HTML: &str = include_str!("../../helpers/template.html");

pub fn get_or_extract_crawler_script() -> PathBuf {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let dir = home
        .map(|h| h.join(".config").join("frontharness").join("helpers"))
        .unwrap_or_else(|| std::env::temp_dir().join("frontharness_helpers"));
    
    let _ = std::fs::create_dir_all(&dir);
    let file_path = dir.join("playwright_crawler.py");
    let _ = std::fs::write(&file_path, PLAYWRIGHT_CRAWLER_SCRIPT);
    file_path
}

pub fn get_template_html() -> &'static str {
    TEMPLATE_HTML
}
