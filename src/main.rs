use clap::{Parser, Subcommand};
use frontharness::agents::PipelineOrchestrator;
use frontharness::config::Settings;
use frontharness::core::EventBus;
use frontharness::llm::discover_models;
use frontharness::tui::TuiApp;
use frontharness::utils::logger::init_logger;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "frontharness", author, version, about = "Event-driven CLI/TUI tool for automated frontend generation and deep redesign")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Run without TUI interface (Headless CLI mode)
    #[arg(long, global = true)]
    headless: bool,

    /// Custom workspace output directory
    #[arg(long, global = true)]
    workspace_dir: Option<PathBuf>,

    /// Log file path
    #[arg(long, global = true)]
    log_file: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Perform deep redesign & audit of a target website or local folder
    Audit {
        /// Target website URL to audit and redesign
        #[arg(short, long)]
        url: Option<String>,

        /// Path to a local site folder (contains index.html / assets)
        #[arg(long)]
        local_dir: Option<PathBuf>,

        /// Business objective and design requirements
        #[arg(short, long, default_value = "Elevate aesthetics and readability, implement smooth modern animations and interactive components, preserving structure while optimizing conversion.")]
        goal: String,

        /// Explicit design style directives (palette, typography, layout)
        #[arg(long)]
        style: Option<String>,

        /// Comma-separated inspiration links or local image reference paths
        #[arg(long)]
        references: Option<String>,

        /// Comma-separated list of mandatory skills (e.g. hallmark,taste,stop_slop,motion,icons)
        #[arg(long)]
        skills: Option<String>,
    },
    /// Generate a brand new web application from scratch
    Greenfield {
        /// Project specification and brief
        #[arg(short, long)]
        goal: String,

        /// Explicit design style directives
        #[arg(long)]
        style: Option<String>,

        /// References and inspiration URLs or file paths
        #[arg(long)]
        references: Option<String>,
    },
    /// Discover available models from the configured LLM endpoint
    Models,
    /// View or update FrontHarness configuration
    Config {
        /// Show active configuration
        #[arg(long)]
        show: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut settings = Settings::load()?;

    if let Some(ws) = cli.workspace_dir {
        settings.workspace_dir = ws;
    }

    let log_path = cli.log_file.unwrap_or_else(|| {
        settings
            .workspace_dir
            .join("frontharness.log")
    });
    let _ = init_logger(Some(&log_path));

    let event_bus = EventBus::default();

    match cli.command {
        Some(Commands::Audit { url, local_dir, goal, style, references, skills }) => {
            let target_source = if let Some(u) = url {
                u
            } else if let Some(ld) = local_dir {
                ld.to_string_lossy().to_string()
            } else {
                "https://as-chelyabinsk.ru/".to_string()
            };

            if let Some(st) = style {
                settings.design.style_prompt = st;
            }
            if let Some(refs) = references {
                settings.design.references = refs.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            }
            if let Some(sk) = skills {
                settings.design.selected_skills = sk.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            }

            println!("[FrontHarness] Starting redesign audit for: {}", target_source);
            println!("[FrontHarness] Design style: {}", settings.design.style_prompt);
            println!("[FrontHarness] Active skills: {:?}", settings.design.selected_skills);
            println!("[FrontHarness] Output workspace: {:?}", settings.workspace_dir);

            if cli.headless {
                println!("[FrontHarness] Running in headless mode...");
                let mut orchestrator = PipelineOrchestrator::new(settings, event_bus);
                let output = orchestrator.run_redesign_pipeline(&target_source, &goal).await?;
                println!("[FrontHarness] Success! Generated frontend saved to: {:?}", output);
            } else {
                let target = target_source.clone();
                let target_goal = goal.clone();
                let orchestrator_bus = event_bus.clone();
                let orch_settings = settings.clone();

                tokio::spawn(async move {
                    let mut orch = PipelineOrchestrator::new(orch_settings, orchestrator_bus);
                    let _ = orch.run_redesign_pipeline(&target, &target_goal).await;
                });

                let mut app = TuiApp::new(&settings.llm.model, &settings.llm.reasoning_effort, event_bus);
                app.run().await?;
            }
        }
        Some(Commands::Greenfield { goal, style, references }) => {
            if let Some(st) = style {
                settings.design.style_prompt = st;
            }
            if let Some(refs) = references {
                settings.design.references = refs.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            }

            println!("[FrontHarness] Starting greenfield generation for: {}", goal);
            let mut orchestrator = PipelineOrchestrator::new(settings.clone(), event_bus.clone());
            let output = orchestrator.run_redesign_pipeline("https://example.com", &goal).await?;
            println!("[FrontHarness] Greenfield generated at: {:?}", output);
        }
        Some(Commands::Models) => {
            println!("[FrontHarness] Fetching models from {}...", settings.llm.base_url);
            let models = discover_models(&settings.llm.base_url, &settings.llm.api_key).await?;
            if models.is_empty() {
                println!("No models discovered or endpoint returned empty list.");
            } else {
                println!("Discovered {} models:", models.len());
                for m in models.iter().take(25) {
                    println!("- {}", m.id);
                }
                if models.len() > 25 {
                    println!("... and {} more.", models.len() - 25);
                }
            }
        }
        Some(Commands::Config { show }) => {
            if show {
                println!("Current Configuration:\n{}", serde_yaml::to_string(&settings)?);
            } else {
                println!("Config location: {:?}", Settings::global_config_path());
            }
        }
        None => {
            let mut app = TuiApp::new(&settings.llm.model, &settings.llm.reasoning_effort, event_bus);
            app.run().await?;
        }
    }

    Ok(())
}
