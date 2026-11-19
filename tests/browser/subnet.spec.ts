import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

const route = 'nec/subnet-lab/';

test.describe('IPv4 Subnet Learning Lab', () => {
  test('initial load renders complete SSR state with accessible 32-bit strip and facts', async ({ page }) => {
    await page.goto(route);

    // Verify main headings and syllabus alignment
    await expect(page.locator('h1')).toContainText('IPv4 Subnetting & Network Calculation Lab');
    await expect(page.locator('.lab-card')).toBeVisible();

    // Verify initial values (Lok Sewa model problem)
    const ipInput = page.locator('[data-ip-input]');
    await expect(ipInput).toHaveValue('200.168.10.75');

    const prefixBadge = page.locator('[data-prefix-badge]');
    await expect(prefixBadge).toHaveText('24');

    // Verify facts grid
    await expect(page.locator('[data-fact-mask]')).toContainText('255.255.255.0 (/24)');
    await expect(page.locator('[data-fact-network]')).toContainText('200.168.10.0/24');
    await expect(page.locator('[data-fact-broadcast]')).toHaveText('200.168.10.255');
    await expect(page.locator('[data-fact-range]')).toHaveText('200.168.10.1 – 200.168.10.254');
    await expect(page.locator('[data-fact-usable-hosts]')).toHaveText('254');

    // Verify 32-bit visual strip
    const stripAlt = page.locator('[data-strip-accessible-text]');
    await expect(stripAlt).toContainText('Network bits 1–24');
    await expect(stripAlt).toContainText('Host bits 25–32');

    // Verify 4 octet bitwise AND steps
    const andRows = page.locator('[data-and-tbody] tr');
    await expect(andRows).toHaveCount(4);
    await expect(andRows.first()).toContainText('Octet 1');
    await expect(andRows.last()).toContainText('Octet 4');

    // Verify equal-size split table
    const splitRows = page.locator('[data-split-tbody] tr');
    await expect(splitRows).toHaveCount(8);

    // Subnet 2 (200.168.10.64/27) holds the host .75
    const hostRow = page.locator('[data-split-tbody] tr.active-host-subnet');
    await expect(hostRow).toBeVisible();
    await expect(hostRow).toContainText('200.168.10.64/27');
    await expect(hostRow).toContainText('Your Host');
  });

  test('accessibility: passes axe-core WCAG AA audit on interactive lab', async ({ page }) => {
    await page.goto(route, { waitUntil: 'networkidle' });
    await page.locator('[data-ip-input]').waitFor({ state: 'visible' });

    const audit = await new AxeBuilder({ page })
      .include('main')
      .withTags(['wcag2a', 'wcag2aa', 'wcag21aa'])
      .analyze();
    expect(audit.violations).toEqual([]);
  });

  test('preserves invalid input readable with last valid output warning and no NaN', async ({ page }) => {
    await page.goto(route);

    const ipInput = page.locator('[data-ip-input]');
    const errorBanner = page.locator('[data-error-banner]');
    const warningBanner = page.locator('[data-warning-banner]');

    // Type invalid IP with octet overflow
    await ipInput.fill('192.168.1.300');

    // Input text must be preserved exactly
    await expect(ipInput).toHaveValue('192.168.1.300');
    await expect(ipInput).toHaveAttribute('aria-invalid', 'true');

    // Error banner displayed with message
    await expect(errorBanner).toBeVisible();
    await expect(page.locator('[data-error-text]')).toContainText('out of range (must be 0–255)');

    // Warning banner displayed indicating last valid values retained
    await expect(warningBanner).toBeVisible();
    await expect(page.locator('[data-warning-text]')).toContainText('last valid input (200.168.10.75/24');

    // Verify NO NaN anywhere in facts
    const factsText = await page.locator('[data-facts-grid]').innerText();
    expect(factsText).not.toContain('NaN');
    expect(factsText).not.toContain('undefined');

    // Test empty octet error
    await ipInput.fill('192.168..1');
    await expect(ipInput).toHaveValue('192.168..1');
    await expect(errorBanner).toBeVisible();
    await expect(page.locator('[data-error-text]')).toContainText('consecutive or misplaced dots');

    // Restore valid address
    await ipInput.fill('192.168.1.50');
    await expect(ipInput).toHaveAttribute('aria-invalid', 'false');
    await expect(errorBanner).toBeHidden();
    await expect(warningBanner).toBeHidden();
    await expect(page.locator('[data-fact-network]')).toContainText('192.168.1.0/24');
  });

  test('handles boundary /31 point-to-point link per RFC 3021 semantics', async ({ page }) => {
    await page.goto(route);

    const ipInput = page.locator('[data-ip-input]');
    const prefixInput = page.locator('[data-prefix-input]');

    await ipInput.fill('10.0.0.1');
    await prefixInput.fill('31');

    // Verify /31 RFC 3021 parameters
    await expect(page.locator('[data-fact-mask]')).toContainText('255.255.255.254 (/31)');
    await expect(page.locator('[data-fact-network]')).toContainText('10.0.0.0/31');
    await expect(page.locator('[data-fact-broadcast]')).toContainText('None (RFC 3021 point-to-point link)');
    await expect(page.locator('[data-fact-usable-hosts]')).toHaveText('2');
    await expect(page.locator('[data-fact-range]')).toContainText('10.0.0.0 – 10.0.0.1 (RFC 3021 endpoints)');

    // Verify RFC context notice banner
    const contextBanner = page.locator('[data-context-banner]');
    await expect(contextBanner).toBeVisible();
    await expect(page.locator('[data-context-text]')).toContainText('RFC 3021');
  });

  test('handles boundary /32 single host loopback route', async ({ page }) => {
    await page.goto(route);

    const ipInput = page.locator('[data-ip-input]');
    const prefixInput = page.locator('[data-prefix-input]');

    await ipInput.fill('172.16.100.5');
    await prefixInput.fill('32');

    await expect(page.locator('[data-fact-mask]')).toContainText('255.255.255.255 (/32)');
    await expect(page.locator('[data-fact-network]')).toContainText('172.16.100.5/32');
    await expect(page.locator('[data-fact-broadcast]')).toContainText('None (/32 single host)');
    await expect(page.locator('[data-fact-usable-hosts]')).toHaveText('1');
    await expect(page.locator('[data-fact-range]')).toContainText('172.16.100.5 (single host / loopback)');

    const contextBanner = page.locator('[data-context-banner]');
    await expect(contextBanner).toBeVisible();
    await expect(page.locator('[data-context-text]')).toContainText('/32');
  });

  test('handles boundary /0 default route with generic arithmetic note', async ({ page }) => {
    await page.goto(route);

    const ipInput = page.locator('[data-ip-input]');
    const prefixInput = page.locator('[data-prefix-input]');

    await ipInput.fill('0.0.0.0');
    await prefixInput.fill('0');

    await expect(page.locator('[data-fact-mask]')).toContainText('0.0.0.0 (/0)');
    await expect(page.locator('[data-fact-network]')).toContainText('0.0.0.0/0');
    await expect(page.locator('[data-fact-usable-hosts]')).toHaveText('4,294,967,294');
    await expect(page.locator('[data-fact-total-addrs]')).toHaveText('4,294,967,296');

    const contextBanner = page.locator('[data-context-banner]');
    await expect(contextBanner).toBeVisible();
    await expect(page.locator('[data-context-text]')).toContainText('default route');
  });

  test('unsigned boundary >= 128 evaluates correctly without negative numbers', async ({ page }) => {
    await page.goto(route);

    const ipInput = page.locator('[data-ip-input]');
    const prefixInput = page.locator('[data-prefix-input]');

    await ipInput.fill('223.255.255.254');
    await prefixInput.fill('29');

    await expect(page.locator('[data-fact-network]')).toContainText('223.255.255.248/29');
    await expect(page.locator('[data-fact-broadcast]')).toHaveText('223.255.255.255');
    await expect(page.locator('[data-fact-range]')).toHaveText('223.255.255.249 – 223.255.255.254');
    await expect(page.locator('[data-fact-usable-hosts]')).toHaveText('6');
  });

  test('worked example preset selector updates inputs and recalculates', async ({ page }) => {
    await page.goto(route);

    const presetSelect = page.locator('[data-preset-select]');
    await presetSelect.selectOption('lok-sewa-ce-2078-host-177');

    await expect(page.locator('[data-ip-input]')).toHaveValue('192.168.1.177');
    await expect(page.locator('[data-prefix-badge]')).toHaveText('26');
    await expect(page.locator('[data-fact-network]')).toContainText('192.168.1.128/26');
    await expect(page.locator('[data-fact-broadcast]')).toHaveText('192.168.1.191');
    await expect(page.locator('[data-fact-range]')).toHaveText('192.168.1.129 – 192.168.1.190');
    await expect(page.locator('[data-fact-usable-hosts]')).toHaveText('62');
  });

  test('static worked examples section has at least 6 expandable worked solutions', async ({ page }) => {
    await page.goto(route);

    const details = page.locator('.worked-examples details');
    const count = await details.count();
    expect(count).toBeGreaterThanOrEqual(6);

    // Expand the first example and check content
    const firstEx = details.first();
    await firstEx.locator('summary').click();
    await expect(firstEx).toHaveAttribute('open', '');
    await expect(firstEx.locator('.solution-highlight')).toBeVisible();
    await expect(firstEx.locator('.static-table').first()).toBeVisible();
    await expect(firstEx.locator('.rationale-box')).toBeVisible();
  });

  test('responsive mobile layout does not trigger horizontal overflow', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto(route);

    const isWithinBounds = await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth);
    expect(isWithinBounds).toBeTruthy();
  });
});

test('blank prefix and fractional borrowing retain the last valid calculation', async({page})=>{
  await page.goto(route);
  await page.locator('[data-prefix-input]').fill('');
  await expect(page.locator('[data-prefix-input]')).toHaveValue('');
  await expect(page.locator('[data-fact-network]')).toHaveText('200.168.10.0/24');
  await expect(page.locator('[data-warning-banner]')).toBeVisible();
  await page.locator('[data-prefix-input]').fill('24');
  await page.locator('[data-borrow-input]').fill('1.5');
  await expect(page.locator('[data-error-banner]')).toBeVisible();
  await expect(page.locator('[data-split-tbody] tr')).toHaveCount(8);
  await expect(page.locator('[data-warning-text]')).toContainText('borrow 3 bits');
  await page.locator('[data-borrow-input]').fill('2');
  await page.locator('[data-ip-input]').fill('192.0.0.1');
  await expect(page.locator('[data-fact-class]')).toContainText('ietf-protocol');
  await page.locator('[data-ip-input]').fill('0.1.2.3');
  await expect(page.locator('[data-fact-class]')).toContainText('this-host');
  await page.locator('[data-prefix-input]').fill('32');
  await expect(page.locator('[data-borrow-input]')).toBeDisabled();
  await page.locator('[data-prefix-input]').fill('24');
  await expect(page.locator('[data-borrow-input]')).toBeEnabled();
  await expect(page.locator('[data-error-banner]')).toBeHidden();
  await expect(page.locator('[data-fact-network]')).toHaveText('0.1.2.0/24');
});
