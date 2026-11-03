# Checks every interactive visual for horizontal overflow at phone width (390 px).
# Usage: build the site, start `npx astro preview --port 4321`, then:
#   python3 scripts/qa/check_visual_overflow.py relational-model-and-sql/subqueries/ hardware/numa/ ...
# Needs Python Playwright (`pip install playwright && playwright install chromium`).
# For every interactive visual (custom elements named qe-*), click each of its buttons at 390px
# and record how far right any visible descendant reaches, ignoring content inside an intentional
# scroll container. Anything past the viewport is an overflow bug.
import sys
from playwright.sync_api import sync_playwright

B = "http://localhost:4321/learning-guide/"
JS = """() => {
  const vw = 390; const bad = []; // not innerWidth: mobile emulation zooms out to fit overflow
  for (const root of document.querySelectorAll('*')) {
    if (!root.tagName.startsWith('QE-')) continue;
    for (const e of root.querySelectorAll('*')) {
      let a = e.parentElement, clipped = false;
      while (a && a !== root) {
        const ox = getComputedStyle(a).overflowX;
        if (ox === 'auto' || ox === 'scroll' || ox === 'hidden') { clipped = true; break; }
        a = a.parentElement;
      }
      if (clipped) continue;
      const r = e.getBoundingClientRect();
      if (r.width && r.right > vw + 1) bad.push(root.tagName.toLowerCase() + ' > ' + e.tagName.toLowerCase() + '.' + (e.className || '') + ' @' + Math.round(r.right));
    }
  }
  return bad;
}"""

with sync_playwright() as p:
    b = p.chromium.launch()
    ctx = b.new_context(viewport={"width": 390, "height": 900}, is_mobile=True, has_touch=True)
    pg = ctx.new_page()
    errs = []
    pg.on("pageerror", lambda e: errs.append(str(e)))
    total = 0
    for path in sys.argv[1:]:
        errs.clear()
        pg.goto(B + path, wait_until="networkidle")
        bad = set(pg.evaluate(JS))
        btns = pg.query_selector_all("[class*='qe-'] button")
        for bt in btns:
            try:
                if bt.is_enabled() and bt.is_visible():
                    bt.click(timeout=1000)
                    bad |= set(pg.evaluate(JS))
            except Exception:
                pass
        total += len(btns)
        if bad or errs:
            print("PROBLEM", path, sorted(bad)[:4], errs[:2])
    print("checked", len(sys.argv) - 1, "pages,", total, "buttons clicked")
    b.close()
