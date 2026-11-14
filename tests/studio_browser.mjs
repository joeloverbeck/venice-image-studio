import assert from 'node:assert/strict';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const browser = await chromium.launch({executablePath: process.env.CHROME_BIN || '/usr/bin/google-chrome', args: ['--no-sandbox']});
const page = await browser.newPage({viewport: {width: 1440, height: 1000}});
const errors = [];
page.on('pageerror', error => errors.push(error.message));
const focused = selector => page.locator(selector).evaluate(node => node === document.activeElement);
try {
  await page.goto(process.env.STUDIO_URL);
  await page.locator('#prompt:not([disabled])').waitFor();
  if (process.env.SCENARIO === 'keyboard') {
    const prompt = 'Exact long prompt 🦊\n'.repeat(90) + '  unchanged ending\n';
    await page.locator('#prompt').fill(prompt);
    await page.getByText('Draft saved locally.', {exact: true}).waitFor();
    await page.locator('#browse-models').focus();
    await page.keyboard.press('Enter');
    await page.locator('#model-search').fill('chroma');
    await page.keyboard.press('Tab');
    // Native selects occupy four tab stops before the matching model row.
    for (let n = 0; n < 4; n++) await page.keyboard.press('Tab');
    assert.equal(await focused('.model-row'), true);
    await page.keyboard.press('Enter');
    assert.equal(await focused('#model-detail'), true, 'Opening a model moves focus to its readable detail instead of stranding it in the catalog');
    await page.keyboard.press('Escape');
    assert.equal(await focused('#browse-models'), true);
    const preset = page.locator('#settings button').first();
    await preset.focus();
    const label = await preset.innerText();
    await page.keyboard.press('Enter');
    await page.getByText('Draft saved locally.', {exact: true}).waitFor();
    assert.equal(await page.evaluate(() => document.activeElement.textContent), label, 'Choosing a size keeps focus on the chosen preset');
    assert.equal(await page.locator('#prompt').inputValue(), prompt);
    assert.notEqual(await page.evaluate(() => getComputedStyle(document.activeElement).outlineStyle), 'none');
    await page.locator('#settings summary').focus();
    await page.keyboard.press('Enter');
    await page.keyboard.press('Tab');
    assert.equal(await focused('#setting-variants'), true);
    await page.keyboard.press('ControlOrMeta+A');
    await page.keyboard.type('2');
    await page.keyboard.press('Tab');
    await page.waitForFunction(() => document.querySelector('#estimate').textContent.includes('$0.04'));
    assert.equal(await page.locator('#setting-seed').isDisabled(), true);
    assert.equal(await focused('#safe'), true, 'Tab skips controls without established support');
    await page.reload();
    await page.waitForFunction(() => document.querySelector('#prompt').value.endsWith('  unchanged ending\n'));
    assert.equal(await page.locator('#prompt').inputValue(), prompt);
    assert.equal(await page.locator('#setting-variants').inputValue(), '2');
  } else if (['waiting', 'saving'].includes(process.env.SCENARIO)) {
    if (process.env.SCENARIO === 'waiting') {
      await page.locator('#prompt').fill('A new explicit attempt 🦊');
      await page.getByText('Draft saved locally.', {exact: true}).waitFor();
      await page.locator('#generate').focus();
      await page.keyboard.press('Enter');
      await page.waitForFunction(() => document.querySelector('#result-status').textContent.includes('waiting'));
    }
    await page.locator(`[data-attempt="${process.env.EARLIER_ID}"]`).focus();
    await page.keyboard.press('Enter');
    await page.getByRole('button', {name: 'Inspect original', exact: true}).focus();
    await page.keyboard.press('Enter');
    const activity = page.locator('#inspector-activity');
    await activity.waitFor({timeout: 2000});
    assert.match(await activity.innerText(), process.env.SCENARIO === 'waiting' ? /waiting/ : /save_failed/);
    if (process.env.SCENARIO === 'waiting') {
      await page.waitForFunction(() => document.querySelector('#inspector-activity').textContent.includes('saved:'));
      assert.equal(await focused('#inspector-viewport'), true, 'Status updates do not steal inspection focus');
    }
    await activity.getByRole('button', {name: 'View attempt'}).focus();
    await page.keyboard.press('Enter');
    assert.equal(await page.locator('#original-inspector').evaluate(node => node.open), false);
    assert.equal(await focused('#result-status'), true);
    if (process.env.SCENARIO === 'saving') {
      await page.getByRole('button', {name: 'Retry local save'}).focus();
      await page.keyboard.press('Enter');
      await page.waitForFunction(() => document.querySelector('#result-status').textContent.startsWith('saved:'));
      assert.equal(await focused('#result-status'), true, 'Removing a successful save-retry button retains understandable focus');
    }
    const exported = page.waitForEvent('download');
    await page.getByRole('link', {name: 'Export recipe JSON'}).focus();
    await page.keyboard.press('Enter');
    await exported;
    assert.equal(await page.locator('#images img').count(), 1);
  } else if (process.env.SCENARIO === 'responsive') {
    await page.setViewportSize({width: 390, height: 844});
    const resultLink = page.getByRole('link', {name: 'Result', exact: true});
    await resultLink.waitFor({timeout: 2000});
    await resultLink.focus();
    await page.keyboard.press('Enter');
    assert.equal(await focused('#result-status'), true);
    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true, 'The narrow Writing desk has no horizontal overflow');
    await page.getByRole('button', {name: 'Inspect original', exact: true}).focus();
    await page.keyboard.press('Enter');
    await page.locator('#inspector-image').evaluate(image => image.decode());
    assert.match(await page.locator('#inspector-metadata').innerText(), /4096 × 4096/);
    assert.equal(await page.locator('#original-inspector').evaluate(node => node.scrollWidth <= node.clientWidth), true, 'Inspector controls fit inside a narrow dialog');
    await page.keyboard.press('1');
    await page.keyboard.press('ArrowRight');
    await page.keyboard.press('+');
    await page.keyboard.press('0');
    assert.match(await page.locator('#inspector-zoom').innerText(), /Fit/);
    assert.equal(await focused('#inspector-viewport'), true);
    await page.keyboard.press('Escape');
    assert.equal(await page.getByRole('button', {name: 'Inspect original', exact: true}).evaluate(node => node === document.activeElement), true);
    await page.getByRole('link', {name: 'Library', exact: true}).focus();
    await page.keyboard.press('Enter');
    assert.equal(await focused('#library-search'), true);
    await page.getByRole('link', {name: 'Prompt', exact: true}).focus();
    await page.keyboard.press('Enter');
    assert.equal(await focused('#prompt'), true);
    await page.locator('#browse-models').focus();
    await page.keyboard.press('Enter');
    await page.locator('.model-row').first().focus();
    await page.keyboard.press('Enter');
    assert.equal(await focused('#model-detail'), true);
    assert.equal(await page.locator('#model-browser').evaluate(node => node.scrollWidth <= node.clientWidth), true, 'Model descriptions and prices fit the narrow dialog');
    await page.keyboard.press('Escape');
    await page.evaluate(() => scrollTo(0, 0));
    if (process.env.STUDIO_SCREENSHOT_DIR) await page.screenshot({path: `${process.env.STUDIO_SCREENSHOT_DIR}/writing-desk-narrow.png`, fullPage: true});
    await page.setViewportSize({width: 1440, height: 1000});
    const promptBox = await page.locator('#prompt').boundingBox();
    const resultBox = await page.locator('.result').boundingBox();
    assert.ok(promptBox.x + promptBox.width < resultBox.x, 'Desktop Writing desk places the prompt to the left of results');
    await page.evaluate(() => scrollTo(0, 0));
    if (process.env.STUDIO_SCREENSHOT_DIR) await page.screenshot({path: `${process.env.STUDIO_SCREENSHOT_DIR}/writing-desk-desktop.png`, fullPage: true});
  }
  assert.deepEqual(errors, []);
  console.log(`PASS: ${process.env.SCENARIO} Writing desk browser workflow.`);
} finally { await browser.close(); }
