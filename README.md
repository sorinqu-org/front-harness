# FrontHarness

FrontHarness is an event-driven CLI/TUI system for automated frontend generation and deep website redesigns. Built in Rust with Ratatui and Tokio, it features a decoupled Pi agent loop, an interactive LazyVim-style terminal user interface, a live in-TUI configuration editor, a multi-provider LLM engine, a Playwright browser crawler, a local MCP multiplexer, and long-term SQLite memory.

---

## Visual Interface & Redesign Results

### Terminal User Interface (Ratatui / LazyVim Style)
![FrontHarness TUI Interface](docs/screenshots/tui_interface.png)

### Redesign Comparison (Desktop & Mobile)

| Original Target Site (Audit) | FrontHarness Redesigned Site (Generated) |
| :---: | :---: |
| ![Original Desktop](docs/screenshots/original_desktop.png) | ![Redesign Desktop](docs/screenshots/redesign_desktop.png) |
| *Original Desktop (1920x1080)* | *Redesigned Desktop with GSAP & Bento Layout (1920x1080)* |

---

## Architecture Overview

1. **Pi Core & Event Bus**: Asynchronous message bus (`tokio::sync::broadcast`) that decouples the agent execution engine from the terminal UI.
2. **LazyVim / OpenCode Style TUI**: Full keyboard navigation using Vim keybindings, in-TUI interactive configuration editor (`c`), real-time token streaming, unified diffs, log inspection, and dynamic statusline.
3. **Multi-Provider LLM Engine**: OpenAI-compatible client with dynamic `/v1/models` discovery, streaming SSE parser, and normalized reasoning effort levels (`low`, `medium`, `high`, `custom`).
4. **Browser Crawler & Asset Dumper**: Playwright crawler that captures desktop (1920x1080) and mobile (375x812) screenshots, extracts computed CSS styles and DOM hierarchy, and dumps network assets (SVGs, fonts, stylesheets).
5. **Multi-Agent DAG Orchestrator**:
   - **Research Agent**: Collects design patterns and competitive references via Tavily Search API.
   - **Art Director Agent**: Establishes typography scales, color palettes, and macrostructures (`design.md`).
   - **Implementation Agent**: Generates accessible, responsive HTML5/Tailwind/GSAP code.
   - **QA & Browser Verification Agent**: Launches an isolated local server and runs Playwright validation.
6. **Skills Matrix**: Embedded directives for Hallmark structural variety, Design Taste dials, Stop-Slop filters, motion physics, and Lucide SVG icons.
7. **Long-Term Memory**: SQLite store for design decisions, successful layouts, and user ratings.

---

## Installation

Run the POSIX installer script:

```bash
chmod +x install.sh
./install.sh
```

Or build manually from source:

```bash
cargo build --release
cp target/release/frontharness ~/.local/bin/frontharness
ln -sf ~/.local/bin/frontharness ~/.local/bin/fh
```

### Prerequisites
- Rust 1.80+ / Cargo
- Python 3.10+ with `playwright` installed (`pip install playwright && playwright install chromium`)
- Node.js (v18+)
- Git

---

## Configuration & In-TUI Editing

You can edit settings directly in the TUI by pressing `c`, or configure `~/.config/frontharness/config.yaml` / `.env`:

```ini
LLM_BASE_URL=https://agentrouter.org/v1
LLM_API_KEY=your_api_key
LLM_MODEL=gpt-5.6-sol
LLM_REASONING_EFFORT=high

TAVILY_API_KEY=your_tavily_key
DEV_SERVER_PORT=3000
BROWSER_HEADLESS=true
```

---

## CLI Usage

### 1. Website Redesign & Audit
Run an end-to-end audit and redesign pipeline against a target website:

```bash
frontharness audit --url https://as-chelyabinsk.ru/ --goal "Elevate readability and visual design, implement smooth animations, and optimize conversion paths."
```

For headless execution without TUI:

```bash
frontharness audit --url https://as-chelyabinsk.ru/ --headless
```

### 2. Greenfield Generation
Generate a new frontend from scratch:

```bash
frontharness greenfield --goal "Build a high-performance B2B SaaS landing page for an observability platform with Bento Grid macrostructure and dark mode."
```

### 3. Model Discovery
List all accessible models from the configured endpoint:

```bash
frontharness models
```

### 4. Configuration Inspection
View active settings and paths:

```bash
frontharness config --show
```

---

## TUI Keybindings

| Keybinding | Action |
| --- | --- |
| `c` | **Open Interactive In-TUI Config Editor** (Edit API Keys, Models, Ports, Headless mode) |
| `Tab` / `Shift+Tab` | Cycle focus between DAG tree, streaming buffer, and logs |
| `j` / `k` or `Down` / `Up` | Scroll content in active pane |
| `d` | Open Code Diff viewer modal |
| `l` | Open full Event Bus & System Logs modal |
| `a` | Open Extracted Assets & Screenshots modal |
| `m` | Open Long-Term Memory & Feedback modal |
| `?` | Toggle Help popup |
| `Esc` / `q` | Close active modal / Quit application |

---

## License
MIT
