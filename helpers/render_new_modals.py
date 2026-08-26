import re
import html
import asyncio
from playwright.async_api import async_playwright

def ansi_to_html(ansi_text, title="FrontHarness Modal"):
    html_content = html.escape(ansi_text)
    html_content = re.sub(r'\x1b\[\d+;\d+H', '\n', html_content)
    html_content = re.sub(r'\x1b\[\d+m', '', html_content)
    html_content = re.sub(r'\x1b\[\?[0-9]+[lh]', '', html_content)
    html_content = re.sub(r'\x1b\[[0-9;]*[a-zA-Z]', '', html_content)
    
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
      <span class="term-title">{title}</span>
    </div>
    <pre>{ansi_text}</pre>
  </div>
</body>
</html>"""

async def capture():
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        page = await browser.new_page(viewport={"width": 1280, "height": 800})
        
        # 1. Capture Greenfield Studio Modal
        try:
            with open("docs/screenshots/tui_greenfield_modal.ansi", "r", encoding="utf-8") as f:
                raw_gf = f.read()
            clean_gf = re.sub(r'\x1b\[\?[0-9]+[a-zA-Z]', '', raw_gf)
            clean_gf = re.sub(r'\x1b\[\d+;\d+H', '\n', clean_gf)
            clean_gf = re.sub(r'\x1b\[[0-9;]*m', '', clean_gf)
            clean_gf = '\n'.join([line for line in clean_gf.split('\n') if '┌' in line or '│' in line or '└' in line or '─' in line or 'Greenfield' in line])
            
            html_doc = ansi_to_html(clean_gf, "FrontHarness — Greenfield (From Scratch) Mode")
            await page.set_content(html_doc)
            await page.screenshot(path="docs/screenshots/tui_greenfield_modal.png")
            print("[Render] Saved docs/screenshots/tui_greenfield_modal.png")
        except Exception as e:
            print("[Render Error] Greenfield modal:", e)
            
        # 2. Capture Review & Refinement Modal
        try:
            with open("docs/screenshots/tui_review_modal.ansi", "r", encoding="utf-8") as f:
                raw_rev = f.read()
            clean_rev = re.sub(r'\x1b\[\?[0-9]+[a-zA-Z]', '', raw_rev)
            clean_rev = re.sub(r'\x1b\[\d+;\d+H', '\n', clean_rev)
            clean_rev = re.sub(r'\x1b\[[0-9;]*m', '', clean_rev)
            clean_rev = '\n'.join([line for line in clean_rev.split('\n') if '┌' in line or '│' in line or '└' in line or '─' in line or 'Evaluation' in line or 'Critique' in line])
            
            html_doc = ansi_to_html(clean_rev, "FrontHarness — Project Review & Iterative Refinement")
            await page.set_content(html_doc)
            await page.screenshot(path="docs/screenshots/tui_review_modal.png")
            print("[Render] Saved docs/screenshots/tui_review_modal.png")
        except Exception as e:
            print("[Render Error] Review modal:", e)
            
        await browser.close()

if __name__ == "__main__":
    asyncio.run(capture())
