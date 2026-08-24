# FrontHarness

FrontHarness is an event-driven CLI/TUI system for automated frontend generation and deep website redesigns. Built in Rust with Ratatui and Tokio, it features a decoupled Pi agent loop, an interactive LazyVim-style terminal user interface with an in-TUI Design Studio (`r`), selective Skills Matrix (`[x]`/`[ ]`), explicit design style directives, local site auditing, reference attachments, a multi-provider LLM engine, a Playwright browser crawler, and long-term SQLite memory.

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

## Core Capabilities

1. **Remote URL or Local Directory Auditing**: Redesign live websites (`https://...`) or audit local folders containing existing `index.html` files, stylesheets, and assets.
2. **Custom Workspace Output**: Direct generated code, audit reports, and downloaded images to any custom workspace directory.
3. **Interactive Skills Matrix**: Selectively toggle mandatory skills with checkboxes (`[x]` / `[ ]`):
   - `Hallmark`: Macrostructure variety (Bento Grid, Workbench, Split Screen, Anti-template).
   - `Taste`: Typography hierarchy, single-accent color rules, and WCAG AA contrast.
   - `Stop-Slop`: AI gradient blocker, fake metric filter, and zero Unicode emojis.
   - `Motion`: GSAP 3.12 ScrollTrigger animations and Lenis smooth scrolling.
   - `Icons`: Lucide vector SVG icons.
   - `Modern Web`: CSS grid/flex, container queries, clamp() scaling.
   - `Security`: XSS sanitization and CSP hardening.
4. **Explicit Design Style Directives**: Enforce exact aesthetic guidelines (e.g. *Dark Slate + Electric Amber, JetBrains Mono, brutalist borders, micro-interactions*).
5. **Design References & Moodboards**: Attach inspiration links (e.g. `https://linear.app`) or local image reference paths.
6. **Search Engine Provider Toggle**: Choose between keyless **DuckDuckGo** or **Tavily Search API**.
7. **Model-Aware Reasoning Effort**: Automatically discover and cycle supported reasoning levels (`low`, `medium`, `high`, `custom`, `2048`, `8192` budget tokens).

---

## How to Run & Use Design Studio

### Method 1: In-TUI Design Studio (Recommended)
1. Run `frontharness` (or `fh`).
2. Press **`r`** (or `n`) on the keyboard to open the **Design Studio & Pipeline Launcher**:
   - **1. Target Source**: Enter remote URL (e.g. `https://as-chelyabinsk.ru/`) or local site folder path (e.g. `./my_site`).
   - **2. Workspace Output**: Specify output directory (e.g. `workspace` or `./dist_output`).
   - **3. Business Goal & Requirements**: Describe your requirements and conversion goals.
   - **4. Design Style Directives**: Define specific aesthetics, fonts, colors, and layout rules.
   - **5. Design References**: Attach comma-separated URLs or local image paths.
   - **6. Active Skills Matrix**: Use `Left`/`Right` (`h`/`l`) to navigate and press **`Space`** to toggle skills `[X]` / `[ ]`.
3. Press **`Enter`** to launch the pipeline!

### Method 2: Direct CLI Commands
- **Audit Remote URL with Custom Style & References**:
  ```bash
  frontharness audit \
    --url https://as-chelyabinsk.ru/ \
    --workspace-dir ./output \
    --style "Dark Slate (#09090b), Electric Amber (#f59e0b) accent, Space Grotesk, Bento Grid, GSAP" \
    --references "https://linear.app, /path/to/moodboard.png" \
    --skills "hallmark,taste,stop_slop,motion,icons" \
    --goal "Elevate readability and conversion, implement interactive booking modal"
  ```
- **Audit Local Site Folder**:
  ```bash
  frontharness audit --local-dir /path/to/local/site --workspace-dir ./workspace
  ```
- **Headless Mode (No TUI)**:
  ```bash
  frontharness audit --url https://as-chelyabinsk.ru/ --headless
  ```

---

## TUI Keybindings

| Keybinding | Action |
| --- | --- |
| `r` / `n` | **Open Design Studio & Pipeline Launcher** (Configure URL/Local dir, Workspace, Style, References, Skills) |
| `c` | **Open Config Editor** (Edit API Keys, Models, Reasoning Effort, Search Engine, Ports) |
| `Space` | Toggle options / Checkbox in Config Editor & Skills Matrix |
| `Tab` / `Shift+Tab` | Cycle focus between inputs / panes |
| `j` / `k` or `Down` / `Up` | Scroll active pane buffer |
| `d` | Open Code Diff viewer modal |
| `l` | Open full Event Bus & System Logs modal |
| `a` | Open Extracted Assets & Screenshots modal |
| `m` | Open Long-Term Memory & Feedback modal |
| `?` | Toggle Help popup |
| `Esc` / `q` | Close active modal / Quit application |

---

## License
MIT
