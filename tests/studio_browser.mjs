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
  } else if (process.env.SCENARIO === 'draft-number') {
    await page.locator('#prompt').fill('Synthetic draft control retention');
    await page.getByText('Draft saved locally.', {exact: true}).waitFor();
    const width = page.locator('#setting-width');
    await width.fill('768');
    await page.getByText('Draft saved locally.', {exact: true}).waitFor();
    assert.equal(await focused('#setting-width'), true, 'Autosave retains the active control');
    await page.reload();
    await page.locator('#prompt:not([disabled])').waitFor();
    assert.equal(await width.inputValue(), '768', 'An edited width survives reload without first leaving the control');
  } else if (process.env.SCENARIO === 'draft-status') {
    await page.locator('#prompt').fill('Synthetic overlapping draft edits');
    await page.getByText('Draft saved locally.', {exact: true}).waitFor();
    const held = new Map([768, 512].map(width => [width, {
      observed: Promise.withResolvers(), release: Promise.withResolvers(), used: false,
    }]));
    await page.route('**/api/state', async route => {
      const response = await route.fetch();
      const state = await response.json();
      const gate = held.get(state.draft.settings.width);
      if (gate && !gate.used) {
        gate.used = true;
        gate.observed.resolve();
        await gate.release.promise;
      }
      // Delay a real server response; no local API state is fabricated.
      await route.fulfill({response});
    });
    try {
      await page.locator('#setting-width').fill('768');
      await held.get(768).observed.promise;
      await page.locator('#setting-width').fill('512');
      held.get(768).release.resolve();
      await held.get(512).observed.promise;
      assert.equal(await page.locator('#draft-status').innerText(), 'Saving draft…', 'An older save response cannot report a newer edit as saved');
      assert.equal(await page.locator('#generate').isDisabled(), true);
      held.get(512).release.resolve();
      await page.getByText('Draft saved locally.', {exact: true}).waitFor();
      assert.equal(await focused('#setting-width'), true);
      await page.reload();
      await page.locator('#prompt:not([disabled])').waitFor();
      assert.equal(await page.locator('#setting-width').inputValue(), '512');
    } finally {
      for (const gate of held.values()) gate.release.resolve();
    }
  } else if (process.env.SCENARIO === 'draft-advanced') {
    const chooseModel = async id => {
      await page.locator('#browse-models').click();
      await page.locator('#model-search').fill(id);
      await page.locator('.model-row').first().click();
      await page.getByRole('button', {name: 'Use this model', exact: true}).click();
      await page.getByText('Draft saved locally.', {exact: true}).waitFor();
      await page.locator('#model-defaults').click();
      await page.getByText('Draft saved locally.', {exact: true}).waitFor();
    };
    await page.locator('#prompt').fill('Synthetic advanced draft retention');
    await page.getByText('Draft saved locally.', {exact: true}).waitFor();
    await chooseModel('venice-sd35');
    await page.locator('#settings summary').click();
    const negative = 'Synthetic blur, cropped text 🦊  ';
    await page.locator('#setting-negative_prompt').fill(negative);
    await page.getByText('Draft saved locally.', {exact: true}).waitFor();
    assert.equal(await focused('#setting-negative_prompt'), true);
    await page.reload();
    await page.locator('#prompt:not([disabled])').waitFor();
    assert.equal(await page.locator('#setting-negative_prompt').inputValue(), negative, 'Free-text advanced choices survive reload without blur');

    await chooseModel('z-image-turbo');
    await page.locator('#settings summary').click();
    const seed = page.locator('#setting-seed');
    await seed.fill('0');
    await page.getByText('Draft saved locally.', {exact: true}).waitFor();
    await page.reload();
    await page.locator('#prompt:not([disabled])').waitFor();
    assert.equal(await seed.inputValue(), '0', 'Explicit zero stays distinct from omission');
    await page.locator('#settings summary').click();
    await seed.fill('');
    await page.getByText('Draft saved locally.', {exact: true}).waitFor();
    await page.reload();
    await page.locator('#prompt:not([disabled])').waitFor();
    assert.equal(await seed.inputValue(), '', 'A blank supported seed remains omitted/random');
    await page.locator('#settings summary').click();
    await seed.focus();
    await page.keyboard.type('1e');
    assert.equal(await seed.evaluate(node => node.validity.badInput), true);
    await page.waitForFunction(() => document.querySelector('#draft-status').textContent.startsWith('Draft not saved'), null, {timeout: 2000});
    assert.equal(await page.locator('#generate').isDisabled(), true, 'Incomplete numeric input cannot authorize a saved omission');
    assert.equal(await focused('#setting-seed'), true);
    await seed.fill('-12');
    await page.getByText('Draft saved locally.', {exact: true}).waitFor();
    assert.equal(await page.locator('#generate').isEnabled(), true);
    await page.locator('#setting-width').fill('767');
    await page.getByText('Draft saved locally.', {exact: true}).waitFor();
    assert.equal(await page.locator('#generate').isDisabled(), true, 'Rust preflight still rejects incompatible numeric choices');
    await page.reload();
    await page.locator('#prompt:not([disabled])').waitFor();
    assert.equal(await page.locator('#setting-width').inputValue(), '767');
    assert.equal(await seed.inputValue(), '-12');
    assert.equal(await page.locator('#generate').isDisabled(), true);
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
