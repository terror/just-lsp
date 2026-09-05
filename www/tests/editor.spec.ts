import { expect, test } from '@playwright/test';

test('parses and highlights edits and resets tree expansion', async ({
  page,
}) => {
  await page.goto('./');

  const editor = page.locator('.cm-content');
  const nodes = page.locator('#tree-panel .tree-node');

  async function caseFor(name: string) {
    await editor.press('ControlOrMeta+a');
    await page.keyboard.insertText(`${name}:\n  echo baz\n`);
    await expect(editor.locator('.cm-just-function')).toHaveText([name]);
    await expect(nodes.filter({ hasText: 'recipe_header' })).toHaveCount(1);
  }

  await caseFor('foo');

  await nodes.first().click();
  await expect(nodes).toHaveCount(1);

  await caseFor('bar');
});

test('preserves panel sizes and saves each orientation separately', async ({
  page,
}) => {
  await page.goto('./');

  const separator = page.getByRole('separator');
  const group = page.locator('[data-slot="resizable-panel-group"]');
  const editor = page.locator('#editor-panel');

  for (const { viewport, orientation, dimension } of [
    {
      viewport: { width: 1280, height: 800 },
      orientation: 'vertical',
      dimension: 'width',
    },
    {
      viewport: { width: 600, height: 900 },
      orientation: 'horizontal',
      dimension: 'height',
    },
  ] as const) {
    await page.setViewportSize(viewport);
    await expect(separator).toHaveAttribute('aria-orientation', orientation);
    await expect(separator).toHaveAttribute('aria-valuenow', '50');

    await separator.press('Home');
    await expect(separator).toHaveAttribute('aria-valuenow', '30');
    await expect
      .poll(async () => {
        const panelBounds = await editor.boundingBox();
        const groupBounds = await group.boundingBox();

        return panelBounds![dimension] / groupBounds![dimension];
      })
      .toBeCloseTo(0.3, 2);

    await page.reload();
    await expect(separator).toHaveAttribute('aria-valuenow', '30');
  }

  await page.setViewportSize({ width: 1280, height: 800 });
  await expect(separator).toHaveAttribute('aria-orientation', 'vertical');
  await expect(separator).toHaveAttribute('aria-valuenow', '30');
});

test('restores panel layouts saved before the dependency upgrade', async ({
  page,
}) => {
  await page.addInitScript(() => {
    localStorage.setItem(
      'react-resizable-panels:just-lsp:panel-layout:horizontal',
      JSON.stringify({
        'editor-panel,tree-panel': { expandToSizes: {}, layout: [40, 60] },
      })
    );
  });

  await page.goto('./');
  await expect(page.getByRole('separator')).toHaveAttribute(
    'aria-valuenow',
    '40'
  );
});
