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

    /// Log file path
    #[arg(long, global = true)]
    log_file: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Perform deep redesign & audit of a target website
    Audit {
        /// Target website URL to audit and redesign
        #[arg(short, long)]
        url: String,

        /// Business objective and design goal for redesign
        #[arg(short, long, default_value = "Elevate aesthetics and readability, implement smooth modern animations and interactive components, preserving structure while optimizing conversion.")]
        goal: String,
    },
    /// Generate a brand new web application from scratch
    Greenfield {
        /// Project specification and brief
        #[arg(short, long)]
        goal: String,
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
    let settings = Settings::load()?;

    let log_path = cli.log_file.unwrap_or_else(|| {
        settings
            .workspace_dir
            .join("workspace")
            .join("frontharness.log")
    });
    let _ = init_logger(Some(&log_path));

    let event_bus = EventBus::default();

    match cli.command {
        Some(Commands::Audit { url, goal }) => {
            println!("[FrontHarness] Starting redesign audit for {}", url);
            let mut orchestrator = PipelineOrchestrator::new(settings.clone(), event_bus.clone());

            if cli.headless {
                println!("[FrontHarness] Running in headless mode...");
                let output = orchestrator.run_redesign_pipeline(&url, &goal).await?;
                println!("[FrontHarness] Success! Generated frontend saved to: {:?}", output);
            } else {
                let target_url = url.clone();
                let target_goal = goal.clone();
                let orchestrator_bus = event_bus.clone();
                let orch_settings = settings.clone();

                tokio::spawn(async move {
                    let mut orch = PipelineOrchestrator::new(orch_settings, orchestrator_bus);
                    let _ = orch.run_redesign_pipeline(&target_url, &target_goal).await;
                });

                let mut app = TuiApp::new(&settings.llm.model, &settings.llm.reasoning_effort, event_bus);
                app.run().await?;
            }
        }
        Some(Commands::Greenfield { goal }) => {
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
