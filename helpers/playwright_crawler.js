/**
 * FrontHarness Playwright Crawler & Asset Interceptor
 * Performs CDP computed CSS extraction, asset dumping, and multi-viewport screenshots.
 */

const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');
const https = require('https');
const http = require('http');
const { URL } = require('url');

async function crawl(targetUrl, outputDir) {
  const screenshotsDir = path.join(outputDir, 'screenshots');
  const assetsDir = path.join(outputDir, 'assets');
  fs.mkdirSync(screenshotsDir, { recursive: true });
  fs.mkdirSync(assetsDir, { recursive: true });

  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({
    viewport: { width: 1920, height: 1080 },
    userAgent: 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36 FrontHarness/1.0'
  });

  const interceptedAssets = [];

  context.on('response', async (response) => {
    try {
      const url = response.url();
      const status = response.status();
      if (status >= 200 && status < 300) {
        const contentType = response.headers()['content-type'] || '';
        const isAsset = url.match(/\.(svg|png|jpg|jpeg|webp|woff2|woff|ttf|css|json|glb|gltf)(\?.*)?$/i) ||
                        contentType.includes('image/') || contentType.includes('font/') || contentType.includes('svg');
        if (isAsset) {
          const parsedUrl = new URL(url);
          const filename = path.basename(parsedUrl.pathname) || `asset_${Date.now()}`;
          const cleanFilename = filename.replace(/[^a-zA-Z0-9._-]/g, '_');
          const savePath = path.join(assetsDir, cleanFilename);
          
          try {
            const buffer = await response.body();
            fs.writeFileSync(savePath, buffer);
            interceptedAssets.push({
              url: url,
              localPath: savePath,
              filename: cleanFilename,
              contentType: contentType,
              sizeBytes: buffer.length
            });
          } catch (e) {}
        }
      }
    } catch (err) {}
  });

  const page = await context.newPage();
  
  console.log(`[Crawler] Navigating to ${targetUrl}...`);
  await page.goto(targetUrl, { waitUntil: 'networkidle', timeout: 30000 }).catch(async () => {
    await page.goto(targetUrl, { waitUntil: 'load', timeout: 15000 });
  });

  await page.waitForTimeout(2000);

  // 1. Desktop Screenshot
  const desktopScreenshot = path.join(screenshotsDir, 'desktop_1920x1080.png');
  await page.screenshot({ path: desktopScreenshot, fullPage: true });
  console.log(`[Crawler] Captured desktop screenshot: ${desktopScreenshot}`);

  // 2. Mobile Screenshot
  await page.setViewportSize({ width: 375, height: 812 });
  await page.waitForTimeout(1000);
  const mobileScreenshot = path.join(screenshotsDir, 'mobile_375x812.png');
  await page.screenshot({ path: mobileScreenshot, fullPage: true });
  console.log(`[Crawler] Captured mobile screenshot: ${mobileScreenshot}`);

  // 3. Computed CSS and DOM structure extraction
  await page.setViewportSize({ width: 1920, height: 1080 });
  const siteAnalysis = await page.evaluate(() => {
    const title = document.title;
    const metaDescription = document.querySelector('meta[name="description"]')?.content || '';
    
    // Color palette extraction
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
      bodyTextSnippet: document.body.innerText.slice(0, 1500)
    };
  });

  await browser.close();

  const result = {
    targetUrl,
    timestamp: new Date().toISOString(),
    screenshots: {
      desktop: desktopScreenshot,
      mobile: mobileScreenshot
    },
    siteAnalysis,
    interceptedAssetsCount: interceptedAssets.length,
    assets: interceptedAssets
  };

  const analysisPath = path.join(outputDir, 'audit_report.json');
  fs.writeFileSync(analysisPath, JSON.stringify(result, null, 2));
  console.log(`[Crawler] Saved audit report to ${analysisPath}`);
  console.log(JSON.stringify({ status: 'success', report: analysisPath }));
}

const args = process.argv.slice(2);
const targetUrl = args[0] || 'https://as-chelyabinsk.ru/';
const outputDir = args[1] || path.join(process.cwd(), 'workspace', 'audit');

crawl(targetUrl, outputDir).catch(err => {
  console.error('[Crawler Error]', err);
  process.exit(1);
});
