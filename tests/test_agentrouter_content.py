import json
import urllib.request

API_KEY = "sk-ZTteZtXRIjm2HTA8TOb40W17aNsDraXN5FB5cA0einnpuZ1y"
URL = "https://agentrouter.org/v1/chat/completions"

headers = {
    "Authorization": f"Bearer {API_KEY}",
    "Content-Type": "application/json",
    "User-Agent": "claude-cli/1.0.108 (external, cli)",
    "anthropic-version": "2023-06-01",
    "anthropic-beta": "claude-code-20250219,oauth-2025-04-20",
    "anthropic-dangerous-direct-browser-access": "true",
    "x-app": "cli"
}

with open("workspace/audit/audit_report.json") as f:
    raw_audit = json.load(f)

# 1. Test with raw audit
print("[Test 1] Testing with full raw audit (with yandex tracking URLs)...")
body1 = {
    "model": "gpt-5.6-sol",
    "messages": [
        {"role": "system", "content": "You are a senior frontend engineer."},
        {"role": "user", "content": f"Here is the raw audit:\n{json.dumps(raw_audit)}\nGenerate index.html"}
    ],
    "stream": False
}

try:
    req = urllib.request.Request(URL, data=json.dumps(body1).encode(), headers=headers, method="POST")
    with urllib.request.urlopen(req) as resp:
        print("[Test 1 Result] Status:", resp.status)
except urllib.error.HTTPError as e:
    print(f"[Test 1 FAILED] HTTP {e.code}: {e.read().decode('utf-8')}")

# 2. Test with sanitized clean audit
print("\n[Test 2] Testing with clean semantic site audit...")
clean_site = {
    "title": raw_audit.get("siteAnalysis", {}).get("title"),
    "metaDescription": raw_audit.get("siteAnalysis", {}).get("metaDescription"),
    "headings": raw_audit.get("siteAnalysis", {}).get("headings"),
    "buttons": raw_audit.get("siteAnalysis", {}).get("buttons"),
    "bodyText": raw_audit.get("siteAnalysis", {}).get("bodyText"),
    "images": [img for img in raw_audit.get("siteAnalysis", {}).get("images", []) if "yandex" not in img.get("src", "")]
}

body2 = {
    "model": "gpt-5.6-sol",
    "messages": [
        {"role": "system", "content": "You are a senior frontend engineer."},
        {"role": "user", "content": f"Here is the clean site structure:\n{json.dumps(clean_site, ensure_ascii=False)}\nGenerate a concise greeting and verify you can read this."}
    ],
    "stream": False
}

try:
    req = urllib.request.Request(URL, data=json.dumps(body2).encode(), headers=headers, method="POST")
    with urllib.request.urlopen(req) as resp:
        res_data = json.loads(resp.read().decode())
        print("[Test 2 Result] Status:", resp.status)
        print("[Test 2 Output]:", res_data["choices"][0]["message"]["content"][:200])
except urllib.error.HTTPError as e:
    print(f"[Test 2 FAILED] HTTP {e.code}: {e.read().decode('utf-8')}")
