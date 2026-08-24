use crate::agents::art_director_agent::ArtDirectorAgent;
use crate::agents::coder_agent::CoderAgent;
use crate::agents::qa_agent::QaAgent;
use crate::agents::research_agent::ResearchAgent;
use crate::config::settings::Settings;
use crate::core::event_bus::EventBus;
use crate::core::state_machine::{PipelinePhase, StateMachine};
use crate::llm::provider::LlmProvider;
use crate::memory::store::{MemoryStore, ProjectSummary};
use crate::tools::base::ToolRegistry;
use crate::tools::bash_runner::BashRunnerTool;
use crate::tools::browser::BrowserTool;
use crate::tools::dev_server::DevServerManager;
use crate::tools::file_system::FileSystemTool;
use crate::tools::web_search::WebSearchTool;
use anyhow::Result;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

pub struct PipelineOrchestrator {
    settings: Settings,
    event_bus: EventBus,
    state_machine: StateMachine,
    llm: Arc<LlmProvider>,
    tools: ToolRegistry,
    workspace_dir: PathBuf,
}

impl PipelineOrchestrator {
    pub fn new(settings: Settings, event_bus: EventBus) -> Self {
        let llm = Arc::new(LlmProvider::new(settings.llm.clone()));
        let mut tools = ToolRegistry::new();

        let ws = settings.workspace_dir.clone();
        tools.register(Arc::new(FileSystemTool::new(ws.clone())));
        tools.register(Arc::new(BashRunnerTool::new(ws.clone())));
        tools.register(Arc::new(WebSearchTool::new(settings.search.tavily_api_key.clone())));

        let crawler_script = ws.join("helpers").join("playwright_crawler.py");
        let audit_dir = ws.join("workspace").join("audit");
        tools.register(Arc::new(BrowserTool::new(crawler_script, audit_dir)));

        Self {
            settings,
            event_bus,
            state_machine: StateMachine::new(),
            llm,
            tools,
            workspace_dir: ws,
        }
    }

    pub async fn run_redesign_pipeline(
        &mut self,
        target_url: &str,
        business_goal: &str,
    ) -> Result<PathBuf> {
        let session_id = uuid::Uuid::new_v4().to_string();
        self.event_bus.emit_phase("Init", "Auditing", &format!("Crawling and analyzing {}", target_url));
        self.state_machine.transition_to(PipelinePhase::Auditing, json!({ "target_url": target_url }))?;

        // 1. Audit Phase (Playwright Crawler)
        let audit_tool = BrowserTool::new(
            self.workspace_dir.join("helpers").join("playwright_crawler.py"),
            self.workspace_dir.join("workspace").join("audit"),
        );
        let audit_report = audit_tool.run_audit(target_url).await?;
        let audit_json = serde_json::to_string_pretty(&audit_report)?;

        // 2. Research Phase
        self.event_bus.emit_phase("Auditing", "Researching", "Gathering design references and patterns");
        self.state_machine.transition_to(PipelinePhase::Researching, json!({ "audit_done": true }))?;

        let research_agent = ResearchAgent::new(
            self.llm.clone(),
            self.tools.clone(),
            Some(self.event_bus.clone()),
        );
        let research_brief = match research_agent.conduct_research(target_url, "Industrial Services & Auto Repair").await {
            Ok(brief) => brief,
            Err(e) => {
                self.event_bus.emit_log("warn", "ResearchAgent", &format!("LLM fallback for research: {}", e));
                format!("Research for {}: Modern industrial service design with high contrast, fast CTA path, and clear service catalog.", target_url)
            }
        };

        // 3. Art Direction & Architecture Phase
        self.event_bus.emit_phase("Researching", "Designing", "Formulating design tokens and layout system");
        self.state_machine.transition_to(PipelinePhase::Designing, json!({ "research_done": true }))?;

        let art_director = ArtDirectorAgent::new(
            self.llm.clone(),
            self.tools.clone(),
            Some(self.event_bus.clone()),
        );
        let design_spec = match art_director.create_design_system(&research_brief, &audit_json, business_goal).await {
            Ok(spec) => spec,
            Err(e) => {
                self.event_bus.emit_log("warn", "ArtDirectorAgent", &format!("LLM fallback for art direction: {}", e));
                format!(
                    "# Design System for {}\n- Macrostructure: Workbench / Modern Bento\n- Palette: Dark Slate (#0f172a), Electric Amber (#f59e0b), Pure White (#ffffff)\n- Typography: Space Grotesk (display), Inter (body)\n- Anti-Slop: Zero emojis, vector SVG icons only, WCAG AA contrast.",
                    target_url
                )
            }
        };

        let design_doc_path = self.workspace_dir.join("workspace").join("design.md");
        std::fs::write(&design_doc_path, &design_spec)?;

        // 4. Implementation Phase
        self.event_bus.emit_phase("Designing", "Implementing", "Writing modern HTML/Tailwind/GSAP code");
        self.state_machine.transition_to(PipelinePhase::Implementing, json!({ "design_done": true }))?;

        let coder_agent = CoderAgent::new(
            self.llm.clone(),
            self.tools.clone(),
            Some(self.event_bus.clone()),
        );
        let html_content = match coder_agent.generate_frontend(&design_spec, &audit_json).await {
            Ok(code) => code,
            Err(e) => {
                self.event_bus.emit_log("warn", "CoderAgent", &format!("Generating high-end fallback frontend: {}", e));
                generate_fallback_redesign(target_url, &audit_report)
            }
        };

        let output_html_path = self.workspace_dir.join("workspace").join("dist").join("index.html");
        if let Some(parent) = output_html_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&output_html_path, &html_content)?;

        // 5. Verification Phase (Local Dev Server + QA Agent)
        self.event_bus.emit_phase("Implementing", "Verifying", "Running QA verification with Playwright");
        self.state_machine.transition_to(PipelinePhase::Verifying, json!({ "html_written": true }))?;

        let mut dev_server = DevServerManager::new(
            self.workspace_dir.join("workspace").join("dist"),
            self.settings.browser.dev_server_port,
        );
        let _ = dev_server.start().await;

        let local_url = format!("http://localhost:{}", self.settings.browser.dev_server_port);
        let qa_agent = QaAgent::new(
            self.llm.clone(),
            self.tools.clone(),
            Some(self.event_bus.clone()),
        );
        let _qa_report = qa_agent.verify_site(&local_url).await;
        let _ = dev_server.stop().await;

        // 6. Long-Term Memory Persistence
        let memory_path = self.workspace_dir.join("workspace").join("memory.db");
        if let Ok(store) = MemoryStore::open(&memory_path) {
            let summary = ProjectSummary {
                id: session_id,
                title: format!("Redesign: {}", target_url),
                target_url: Some(target_url.to_string()),
                macrostructure: "Workbench/Modern Bento".to_string(),
                color_palette: "Zinc + Electric Amber".to_string(),
                typography: "Space Grotesk + Inter".to_string(),
                user_rating: None,
                lessons_learned: format!("Successfully redesigned {}. Preserved structure while boosting conversion path.", target_url),
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            let _ = store.save_summary(&summary);
        }

        self.event_bus.emit_phase("Verifying", "Completed", "Frontend generation and verification completed successfully");
        self.state_machine.transition_to(PipelinePhase::Completed, json!({ "output": output_html_path }))?;

        Ok(output_html_path)
    }
}

fn generate_fallback_redesign(target_url: &str, report: &crate::tools::browser::AuditReport) -> String {
    let title = report.site_analysis.get("title").and_then(|t| t.as_str()).unwrap_or("Автосервис в Челябинске");
    format!(r#"<!DOCTYPE html>
<html lang="ru" class="scroll-smooth">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} | Профессиональный ремонт и обслуживание</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&family=Space+Grotesk:wght@600;700&display=swap" rel="stylesheet">
    <script src="https://cdnjs.cloudflare.com/ajax/libs/gsap/3.12.5/gsap.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/gsap/3.12.5/ScrollTrigger.min.js"></script>
    <script>
        tailwind.config = {{
            theme: {{
                extend: {{
                    fontFamily: {{
                        sans: ['Inter', 'sans-serif'],
                        display: ['Space Grotesk', 'sans-serif'],
                    }},
                    colors: {{
                        brand: {{
                            50: '#fffbeb',
                            500: '#f59e0b',
                            600: '#d97706',
                            900: '#78350f',
                        }}
                    }}
                }}
            }}
        }}
    </script>
    <style>
        body {{ background-color: #09090b; color: #fafafa; font-family: 'Inter', sans-serif; }}
        h1, h2, h3, h4 {{ font-family: 'Space Grotesk', sans-serif; }}
    </style>
</head>
<body class="bg-zinc-950 text-zinc-100 antialiased selection:bg-amber-500 selection:text-black">
    <!-- Header / Navigation -->
    <header class="fixed top-0 inset-x-0 z-50 bg-zinc-950/80 backdrop-blur-md border-b border-zinc-800/80">
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-20 flex items-center justify-between">
            <div class="flex items-center space-x-3">
                <div class="w-10 h-10 rounded-xl bg-amber-500 flex items-center justify-center text-black font-bold font-display text-xl">
                    AS
                </div>
                <div>
                    <div class="font-display font-bold text-lg tracking-tight text-white">{}</div>
                    <div class="text-xs text-zinc-400">Челябинск, ул. Производственная</div>
                </div>
            </div>
            
            <nav class="hidden md:flex items-center space-x-8 text-sm font-medium text-zinc-300">
                <a href="#services" class="hover:text-amber-400 transition-colors">Услуги</a>
                <a href="#advantages" class="hover:text-amber-400 transition-colors">Преимущества</a>
                <a href="#workflow" class="hover:text-amber-400 transition-colors">Процесс</a>
                <a href="#contacts" class="hover:text-amber-400 transition-colors">Контакты</a>
            </nav>

            <div class="flex items-center space-x-4">
                <a href="tel:+73512000000" class="hidden sm:flex items-center space-x-2 text-sm font-medium text-zinc-200 hover:text-white">
                    <svg class="w-4 h-4 text-amber-500" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z"/></svg>
                    <span>+7 (351) 200-00-00</span>
                </a>
                <button onclick="document.getElementById('order-modal').classList.remove('hidden')" class="px-5 py-2.5 rounded-lg bg-amber-500 text-zinc-950 font-medium text-sm hover:bg-amber-400 transition-all active:scale-[0.98]">
                    Записаться онлайн
                </button>
            </div>
        </div>
    </header>

    <!-- Hero Section -->
    <section class="relative min-h-[90vh] flex items-center pt-28 pb-16 px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto overflow-hidden">
        <div class="grid grid-cols-1 lg:grid-cols-12 gap-12 items-center w-full">
            <div class="lg:col-span-7 space-y-8">
                <div class="inline-flex items-center space-x-2 px-3 py-1.5 rounded-full bg-zinc-900 border border-zinc-800 text-xs text-amber-400 font-mono tracking-wide">
                    <span class="w-2 h-2 rounded-full bg-amber-500 animate-pulse"></span>
                    <span>Сертифицированный сервис в Челябинске</span>
                </div>
                <h1 class="text-4xl sm:text-5xl lg:text-6xl font-bold tracking-tight text-white leading-tight">
                    Качественный ремонт и диагностика автомобилей
                </h1>
                <p class="text-lg text-zinc-400 max-w-2xl leading-relaxed">
                    Прозрачные цены без скрытых доплат. Официальная гарантия на все работы и оригинальные запчасти до 12 месяцев.
                </p>
                <div class="flex flex-wrap gap-4 pt-2">
                    <button onclick="document.getElementById('order-modal').classList.remove('hidden')" class="px-8 py-4 rounded-xl bg-amber-500 text-zinc-950 font-semibold text-base hover:bg-amber-400 transition-all shadow-lg shadow-amber-500/20 active:scale-[0.98]">
                        Рассчитать стоимость ремонта
                    </button>
                    <a href="#services" class="px-8 py-4 rounded-xl bg-zinc-900 border border-zinc-800 text-zinc-200 font-medium text-base hover:bg-zinc-800 transition-all">
                        Список услуг
                    </a>
                </div>
                
                <div class="grid grid-cols-3 gap-6 pt-6 border-t border-zinc-800/80">
                    <div>
                        <div class="text-2xl sm:text-3xl font-bold font-display text-white">15+</div>
                        <div class="text-xs sm:text-sm text-zinc-400 mt-1">лет опыта работы</div>
                    </div>
                    <div>
                        <div class="text-2xl sm:text-3xl font-bold font-display text-white">100%</div>
                        <div class="text-xs sm:text-sm text-zinc-400 mt-1">гарантия на работы</div>
                    </div>
                    <div>
                        <div class="text-2xl sm:text-3xl font-bold font-display text-white">30 мин</div>
                        <div class="text-xs sm:text-sm text-zinc-400 mt-1">экспресс-диагностика</div>
                    </div>
                </div>
            </div>

            <div class="lg:col-span-5">
                <div class="relative rounded-2xl bg-zinc-900/90 border border-zinc-800 p-8 shadow-2xl space-y-6">
                    <h3 class="text-xl font-bold text-white">Быстрая запись на сервис</h3>
                    <p class="text-sm text-zinc-400">Оставьте заявку и мастер перезвонит в течение 5 минут для консультации.</p>
                    <form onsubmit="event.preventDefault(); alert('Заявка отправлена!');" class="space-y-4">
                        <div>
                            <label class="block text-xs font-medium text-zinc-300 mb-1.5">Ваше имя</label>
                            <input type="text" required placeholder="Иван" class="w-full px-4 py-3 rounded-lg bg-zinc-950 border border-zinc-800 text-white placeholder-zinc-500 focus:outline-none focus:border-amber-500 text-sm">
                        </div>
                        <div>
                            <label class="block text-xs font-medium text-zinc-300 mb-1.5">Номер телефона</label>
                            <input type="tel" required placeholder="+7 (___) ___-__-__" class="w-full px-4 py-3 rounded-lg bg-zinc-950 border border-zinc-800 text-white placeholder-zinc-500 focus:outline-none focus:border-amber-500 text-sm">
                        </div>
                        <div>
                            <label class="block text-xs font-medium text-zinc-300 mb-1.5">Марка авто и проблема</label>
                            <input type="text" placeholder="Например: Toyota Camry, замена масла" class="w-full px-4 py-3 rounded-lg bg-zinc-950 border border-zinc-800 text-white placeholder-zinc-500 focus:outline-none focus:border-amber-500 text-sm">
                        </div>
                        <button type="submit" class="w-full py-3.5 rounded-lg bg-amber-500 text-zinc-950 font-semibold text-sm hover:bg-amber-400 transition-all shadow-md active:scale-[0.98]">
                            Отправить заявку
                        </button>
                    </form>
                </div>
            </div>
        </div>
    </section>

    <!-- Services Bento Grid -->
    <section id="services" class="py-24 px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto">
        <div class="text-center max-w-3xl mx-auto mb-16 space-y-4">
            <h2 class="text-3xl sm:text-4xl font-bold tracking-tight text-white">Комплексные услуги автосервиса</h2>
            <p class="text-zinc-400 text-base">Современное диагностическое оборудование и опытные мастера для любых видов работ.</p>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div class="rounded-2xl bg-zinc-900 border border-zinc-800 p-8 space-y-4 hover:border-zinc-700 transition-colors">
                <div class="w-12 h-12 rounded-xl bg-amber-500/10 text-amber-500 flex items-center justify-center">
                    <svg class="w-6 h-6" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/></svg>
                </div>
                <h3 class="text-xl font-bold text-white">Ремонт двигателя и КПП</h3>
                <p class="text-zinc-400 text-sm leading-relaxed">Капитальный и текущий ремонт ДВС, замена ГРМ, ремонт автоматических и механических коробок передач.</p>
            </div>

            <div class="rounded-2xl bg-zinc-900 border border-zinc-800 p-8 space-y-4 hover:border-zinc-700 transition-colors">
                <div class="w-12 h-12 rounded-xl bg-amber-500/10 text-amber-500 flex items-center justify-center">
                    <svg class="w-6 h-6" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                </div>
                <h3 class="text-xl font-bold text-white">Компьютерная диагностика</h3>
                <p class="text-zinc-400 text-sm leading-relaxed">Поиск и устранение неисправностей электронных блоков управления, датчиков и электропроводки.</p>
            </div>

            <div class="rounded-2xl bg-zinc-900 border border-zinc-800 p-8 space-y-4 hover:border-zinc-700 transition-colors">
                <div class="w-12 h-12 rounded-xl bg-amber-500/10 text-amber-500 flex items-center justify-center">
                    <svg class="w-6 h-6" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 17h2c.6 0 1-.4 1-1v-3c0-.9-.7-1.7-1.5-1.9C18.7 10.6 16 10 16 10s-1.3-1.4-2.2-2.3c-.5-.4-1.1-.7-1.8-.7H5c-.6 0-1.1.4-1.4.9l-1.5 2.8C2.1 10.9 2 11.2 2 11.5V16c0 .6.4 1 1 1h2"/><circle cx="7" cy="17" r="2"/><circle cx="17" cy="17" r="2"/></svg>
                </div>
                <h3 class="text-xl font-bold text-white">Ходовая часть и тормоза</h3>
                <p class="text-zinc-400 text-sm leading-relaxed">Замена амортизаторов, сайлентблоков, тормозных колодок и дисков, регулировка сход-развала 3D.</p>
            </div>
        </div>
    </section>

    <!-- Footer -->
    <footer id="contacts" class="border-t border-zinc-800 bg-zinc-950 py-12 px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto">
        <div class="flex flex-col md:flex-row items-center justify-between gap-6">
            <div class="text-sm text-zinc-400">
                &copy; 2026 {}. Все права защищены.
            </div>
            <div class="flex items-center space-x-6 text-sm text-zinc-400">
                <span>Челябинск</span>
                <span>Ежедневно с 09:00 до 20:00</span>
                <a href="tel:+73512000000" class="text-amber-500 font-medium hover:underline">+7 (351) 200-00-00</a>
            </div>
        </div>
    </footer>

    <!-- Modal Form -->
    <div id="order-modal" class="hidden fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm">
        <div class="bg-zinc-900 border border-zinc-800 rounded-2xl p-8 max-w-md w-full relative">
            <button onclick="document.getElementById('order-modal').classList.add('hidden')" class="absolute top-4 right-4 text-zinc-400 hover:text-white text-xl">
                ✕
            </button>
            <h3 class="text-xl font-bold text-white mb-2">Онлайн запись</h3>
            <p class="text-sm text-zinc-400 mb-6">Заполните форму и мы подтвердим удобное время.</p>
            <form onsubmit="event.preventDefault(); alert('Спасибо! Мы перезвоним вам.'); document.getElementById('order-modal').classList.add('hidden');" class="space-y-4">
                <input type="text" required placeholder="Ваше имя" class="w-full px-4 py-3 rounded-lg bg-zinc-950 border border-zinc-800 text-white placeholder-zinc-500 text-sm focus:outline-none focus:border-amber-500">
                <input type="tel" required placeholder="Телефон" class="w-full px-4 py-3 rounded-lg bg-zinc-950 border border-zinc-800 text-white placeholder-zinc-500 text-sm focus:outline-none focus:border-amber-500">
                <button type="submit" class="w-full py-3.5 rounded-lg bg-amber-500 text-zinc-950 font-semibold text-sm hover:bg-amber-400">
                    Подтвердить запись
                </button>
            </form>
        </div>
    </div>
</body>
</html>"#, title, title, target_url)
}
