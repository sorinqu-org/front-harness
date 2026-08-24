pub const APP_NAME: &str = "FrontHarness";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_CONFIG_DIR: &str = ".config/frontharness";
pub const DEFAULT_DB_FILENAME: &str = "memory.db";
pub const DEFAULT_MODELS_CACHE: &str = "models_cache.json";

pub const DEFAULT_LLM_BASE_URL: &str = "https://agentrouter.org/v1";
pub const DEFAULT_LLM_MODEL: &str = "gpt-5.6-sol";
pub const DEFAULT_REASONING_EFFORT: &str = "high";
pub const DEFAULT_DEV_SERVER_PORT: u16 = 3000;

pub const DEFAULT_MAX_TOKENS: u32 = 16384;
pub const DEFAULT_TEMPERATURE: f32 = 0.7;

pub const SYSTEM_PROMPT_RESEARCH: &str = "You are the Senior Research Agent in FrontHarness. Analyze frontend design trends, target markets, user expectations, and modern interaction architectures. Produce concise research briefs with real data.";

pub const SYSTEM_PROMPT_ART_DIRECTOR: &str = "You are the Art Director Agent in FrontHarness. Create strict design systems, color tokens (OKLCH/Hex), typography scales, macrostructure selections, and anti-slop rules. Strictly forbid Unicode emojis in UI. Specify SVG icons.";

pub const SYSTEM_PROMPT_CODER: &str = "You are the Senior Frontend Implementation Agent in FrontHarness. Write modular, semantic, accessible HTML, Tailwind CSS, and modern JavaScript with GSAP/Motion. Never use Unicode emojis. Use vector SVG icons.";

pub const SYSTEM_PROMPT_QA: &str = "You are the QA and Browser Verification Agent in FrontHarness. Inspect generated web interfaces, test interactive elements, verify responsiveness at 375px and 1920px, and audit console logs.";
