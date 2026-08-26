import asyncio
import os
from playwright.async_api import async_playwright

async def capture_redesign():
    dist_html = os.path.abspath("workspace/dist/index.html")
    file_url = f"file://{dist_html}"
    
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        
        # Desktop
        page = await browser.new_page(viewport={"width": 1920, "height": 1080})
        await page.goto(file_url, wait_until="networkidle", timeout=30000)
        await page.wait_for_timeout(1000)
        await page.screenshot(path="docs/screenshots/redesign_desktop.png", full_page=True)
        print("[Screenshots] Saved docs/screenshots/redesign_desktop.png")
        
        # Mobile
        await page.set_viewport_size({"width": 375, "height": 812})
        await page.wait_for_timeout(500)
        await page.screenshot(path="docs/screenshots/redesign_mobile.png", full_page=True)
        print("[Screenshots] Saved docs/screenshots/redesign_mobile.png")
        
        await browser.close()

if __name__ == "__main__":
    asyncio.run(capture_redesign())
