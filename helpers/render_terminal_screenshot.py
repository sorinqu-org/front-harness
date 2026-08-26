import re
import html
import asyncio
from playwright.async_api import async_playwright

def ansi_to_html(ansi_text):
    # Strip cursor movement escapes and clean basic ANSI
    clean_lines = []
    
    # Split text into lines or process escapes
    # Replace ANSI color escapes with spans
    ansi_color_map = {
        "30": "#1e1e1e", "31": "#ef4444", "32": "#22c55e", "33": "#eab308",
        "34": "#3b82f6", "35": "#a855f7", "36": "#06b6d4", "37": "#f3f4f6",
        "90": "#6b7280", "91": "#f87171", "92": "#4ade80", "93": "#facc15",
        "94": "#60a5fa", "95": "#c084fc", "96": "#22d3ee", "97": "#ffffff",
    }
    
    # Simple ANSI renderer
    html_content = html.escape(ansi_text)
    # Remove cursor positioning
    html_content = re.sub(r'\x1b\[\d+;\d+H', '\n', html_content)
    html_content = re.sub(r'\x1b\[\d+m', '', html_content)
    html_content = re.sub(r'\x1b\[\?[0-9]+[lh]', '', html_content)
    html_content = re.sub(r'\x1b\[[0-9;]*[a-zA-Z]', '', html_content)
    
    lines = [l for l in html_content.split('\n') if l.strip()]
    
    return f"""<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
  body {{
    background-color: #0b0f19;
    color: #e2e8f0;
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 14px;
    line-height: 1.4;
    padding: 24px;
    margin: 0;
  }}
  .window {{
    background: #0f172a;
    border: 1px solid #1e293b;
    border-radius: 8px;
    box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.5);
    overflow: hidden;
  }}
  .titlebar {{
    background: #1e293b;
    padding: 8px 16px;
    display: flex;
    align-items: center;
    gap: 8px;
  }}
  .dot {{ width: 12px; height: 12px; border-radius: 50%; display: inline-block; }}
  .red {{ background: #ef4444; }}
  .yellow {{ background: #eab308; }}
  .green {{ background: #22c55e; }}
  .term-title {{ color: #94a3b8; font-size: 12px; margin-left: 8px; }}
  pre {{
    margin: 0;
    padding: 20px;
    color: #38bdf8;
    white-space: pre;
    font-weight: 500;
  }}
</style>
</head>
<body>
  <div class="window">
    <div class="titlebar">
      <span class="dot red"></span>
      <span class="dot yellow"></span>
      <span class="dot green"></span>
      <span class="term-title">FrontHarness TUI — LazyVim Style Event-Driven Architecture</span>
    </div>
    <pre>{ansi_text}</pre>
  </div>
</body>
</html>"""

async def capture():
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        page = await browser.new_page(viewport={"width": 1280, "height": 800})
        
        # 1. Capture TUI Studio Modal
        try:
            with open("docs/screenshots/tui_screen_2_studio.ansi", "r", encoding="utf-8") as f:
                raw_studio = f.read()
            clean_studio = re.sub(r'\x1b\[\?[0-9]+[a-zA-Z]', '', raw_studio)
            clean_studio = re.sub(r'\x1b\[\d+;\d+H', '\n', clean_studio)
            clean_studio = re.sub(r'\x1b\[[0-9;]*m', '', clean_studio)
            clean_studio = '\n'.join([line for line in clean_studio.split('\n') if '┌' in line or '│' in line or '└' in line or '─' in line])
            
            html_doc = ansi_to_html(clean_studio)
            await page.set_content(html_doc)
            await page.screenshot(path="docs/screenshots/tui_studio_modal.png")
            print("[Render] Saved docs/screenshots/tui_studio_modal.png")
        except Exception as e:
            print("[Render Error] Studio modal:", e)
            
        # 2. Capture TUI Main Interface
        try:
            with open("docs/screenshots/tui_screen_1_idle.ansi", "r", encoding="utf-8") as f:
                raw_idle = f.read()
            clean_idle = re.sub(r'\x1b\[\?[0-9]+[a-zA-Z]', '', raw_idle)
            clean_idle = re.sub(r'\x1b\[\d+;\d+H', '\n', clean_idle)
            clean_idle = re.sub(r'\x1b\[[0-9;]*m', '', clean_idle)
            clean_idle = '\n'.join([line for line in clean_idle.split('\n') if '┌' in line or '│' in line or '└' in line or '─' in line or 'Pipeline DAG' in line])
            
            html_doc = ansi_to_html(clean_idle)
            await page.set_content(html_doc)
            await page.screenshot(path="docs/screenshots/tui_interface.png")
            print("[Render] Saved docs/screenshots/tui_interface.png")
        except Exception as e:
            print("[Render Error] Main interface:", e)
            
        await browser.close()

if __name__ == "__main__":
    asyncio.run(capture())
