import asyncio
import json
import os
import sys
from urllib.parse import urlparse
from playwright.async_api import async_playwright

async def crawl(target_url: str, output_dir: str):
    screenshots_dir = os.path.join(output_dir, "screenshots")
    assets_dir = os.path.join(output_dir, "assets")
    os.makedirs(screenshots_dir, exist_ok=True)
    os.makedirs(assets_dir, exist_ok=True)

    intercepted_assets = []

    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        context = await browser.new_context(
            viewport={"width": 1920, "height": 1080},
            user_agent="Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36 FrontHarness/1.0"
        )

        async def on_response(response):
            try:
                url = response.url
                status = response.status
                if 200 <= status < 300:
                    content_type = response.headers.get("content-type", "")
                    is_asset = any(url.lower().endswith(ext) for ext in [".svg", ".png", ".jpg", ".jpeg", ".webp", ".woff2", ".woff", ".ttf", ".css", ".json", ".glb", ".gltf"]) or "image/" in content_type or "font/" in content_type or "svg" in content_type
                    if is_asset:
                        parsed = urlparse(url)
                        raw_name = os.path.basename(parsed.path) or f"asset_{len(intercepted_assets)}"
                        clean_name = "".join(c if c.isalnum() or c in "._-" else "_" for c in raw_name)
                        save_path = os.path.join(assets_dir, clean_name)
                        try:
                            body = await response.body()
                            with open(save_path, "wb") as f:
                                f.write(body)
                            intercepted_assets.append({
                                "url": url,
                                "localPath": save_path,
                                "filename": clean_name,
                                "contentType": content_type,
                                "sizeBytes": len(body)
                            })
                        except Exception:
                            pass
            except Exception:
                pass

        context.on("response", on_response)
        page = await context.new_page()

        print(f"[Crawler] Navigating to {target_url}...")
        try:
            await page.goto(target_url, wait_until="networkidle", timeout=30000)
        except Exception:
            try:
                await page.goto(target_url, wait_until="load", timeout=15000)
            except Exception as e:
                print(f"[Crawler] Navigation warning: {e}")

        await page.wait_for_timeout(2000)

        # 1. Desktop Screenshot
        desktop_screenshot = os.path.join(screenshots_dir, "desktop_1920x1080.png")
        await page.screenshot(path=desktop_screenshot, full_page=True)
        print(f"[Crawler] Captured desktop screenshot: {desktop_screenshot}")

        # 2. Mobile Screenshot
        await page.set_viewport_size({"width": 375, "height": 812})
        await page.wait_for_timeout(1000)
        mobile_screenshot = os.path.join(screenshots_dir, "mobile_375x812.png")
        await page.screenshot(path=mobile_screenshot, full_page=True)
        print(f"[Crawler] Captured mobile screenshot: {mobile_screenshot}")

        # 3. Computed CSS and DOM structure
        await page.set_viewport_size({"width": 1920, "height": 1080})
        site_analysis = await page.evaluate("""() => {
            const title = document.title;
            const metaDescription = document.querySelector('meta[name="description"]')?.content || '';
            const colors = new Set();
            const fonts = new Set();
            const headings = [];

            const elements = document.querySelectorAll('h1, h2, h3, h4, p, a, button, nav, section, header, footer');
            elements.forEach((el) => {
                const style = window.getComputedStyle(el);
                if (style.color) colors.add(style.color);
                if (style.backgroundColor && style.backgroundColor !== 'rgba(0, 0, 0, 0)') colors.add(style.backgroundColor);
                if (style.fontFamily) fonts.add(style.fontFamily);
            });

            document.querySelectorAll('h1, h2, h3').forEach((h) => {
                headings.push({
                    tag: h.tagName.toLowerCase(),
                    text: h.innerText.trim().slice(0, 100)
                });
            });

            const buttons = Array.from(document.querySelectorAll('button, a.btn, .btn, a[href*="tel:"], a[href*="order"]'))
                .map(b => b.innerText.trim())
                .filter(t => t.length > 0 && t.length < 50);

            return {
                title,
                metaDescription,
                fonts: Array.from(fonts).slice(0, 10),
                colors: Array.from(colors).slice(0, 20),
                headings: headings.slice(0, 15),
                buttons: buttons.slice(0, 10),
                bodyTextSnippet: document.body.innerText.slice(0, 2500)
            };
        }""")

        await browser.close()

        result = {
            "targetUrl": target_url,
            "timestamp": "now",
            "screenshots": {
                "desktop": desktop_screenshot,
                "mobile": mobile_screenshot
            },
            "siteAnalysis": site_analysis,
            "interceptedAssetsCount": len(intercepted_assets),
            "assets": intercepted_assets
        }

        analysis_path = os.path.join(output_dir, "audit_report.json")
        with open(analysis_path, "w", encoding="utf-8") as f:
            json.dump(result, f, indent=2, ensure_ascii=False)

        print(f"[Crawler] Saved audit report to {analysis_path}")
        print(json.dumps({"status": "success", "report": analysis_path}))

if __name__ == "__main__":
    target = sys.argv[1] if len(sys.argv) > 1 else "https://as-chelyabinsk.ru/"
    out = sys.argv[2] if len(sys.argv) > 2 else os.path.join(os.getcwd(), "workspace", "audit")
    asyncio.run(crawl(target, out))
