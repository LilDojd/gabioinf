import assert from "node:assert/strict";
import { after, before, test } from "node:test";
import { readFile } from "node:fs/promises";
import { chromium } from "playwright";

// Run against `just serve-test-content`, never a production write path.
const origin = process.env.APP_URL ?? "http://localhost:8080";
const article = `${origin}/blog/blog-rendering-showcase?viewer=test`;
let browser;
before(async () => {
  browser = await chromium.launch({ headless: true, channel: process.env.PLAYWRIGHT_CHANNEL });
});
after(async () => { await browser?.close(); });

async function hydrated(page) {
  await page.locator("[data-dioxus-id]").first().waitFor({ state: "attached" });
}

async function openArticle(options = {}) {
  const context = await browser.newContext({ permissions: ["clipboard-read", "clipboard-write"] });
  if (options.offlineHighlighting) await context.route("https://cdn.jsdelivr.net/**", (route) => route.abort());
  const page = await context.newPage();
  await page.goto(article);
  const block = page.locator(".code-block").first();
  await block.getByRole("button", { name: "Copy code", exact: true }).waitFor();
  await hydrated(page);
  return { context, page, block };
}

async function displayedSource(block) {
  return (await block.locator(".code-line-text").allTextContents()).join("\n");
}

test("line selection belongs to one block and preserves the page query", async () => {
  const { context, page, block } = await openArticle();
  try {
    const id = await block.getAttribute("id");
    await block.getByRole("button", { name: "Select line 7", exact: true }).click();
    await block.getByRole("button", { name: "Select line 3", exact: true }).click({ modifiers: ["Shift"] });
    assert.equal(new URL(page.url()).hash, `#${id}-L3-L7`);
    assert.equal(new URL(page.url()).search, "?viewer=test");
    assert.equal(await block.locator(".code-line.is-selected").count(), 5);
    assert.equal(await page.locator(".code-block").nth(1).locator(".is-selected").count(), 0);
    await page.reload();
    await block.locator(".code-line.is-selected").first().waitFor();
    assert.equal(await block.locator(".code-line.is-selected").count(), 5);
    await block.getByRole("button", { name: "Clear selected lines" }).click();
    assert.equal(await block.locator(".code-line.is-selected").count(), 0);
  } finally { await context.close(); }
});

test("copy and wrapping stay usable when grammar loading fails", async () => {
  const { context, page, block } = await openArticle({ offlineHighlighting: true });
  try {
    const source = await displayedSource(block);
    await block.getByRole("button", { name: "Wrap lines", exact: true }).click();
    assert.match(await block.getAttribute("class"), /code-wrap/);
    assert.equal(await displayedSource(block), source);
    await block.getByRole("button", { name: "Copy code", exact: true }).click();
    await block.getByText("Code copied.", { exact: true }).waitFor();
    assert.equal(await page.evaluate(() => navigator.clipboard.readText()), source);
  } finally { await context.close(); }
});

test("a denied clipboard write reports failure rather than success", async () => {
  const { context, page, block } = await openArticle();
  try {
    await page.evaluate(() => {
      Object.defineProperty(navigator, "clipboard", { value: {
        writeText: async () => { throw new DOMException("denied", "NotAllowedError"); },
      } });
    });
    await block.getByRole("button", { name: "Copy code", exact: true }).click();
    await block.getByText("Copy failed. Select the code and copy manually.", { exact: true }).waitFor();
    assert.equal(await block.getByText("Code copied.", { exact: true }).count(), 0);
  } finally { await context.close(); }
});

test("pinned CDN grammars highlight multiple languages without altering source", async () => {
  const context = await browser.newContext();
  try {
    const bridge = await readFile(new URL("../assets/code_highlighting.js", import.meta.url), "utf8");
    await context.route(`${origin}/__highlight-test.js`, (route) => route.fulfill({ contentType: "text/javascript", body: bridge }));
    const page = await context.newPage();
    await page.goto(origin);
    const results = await page.evaluate(async () => {
      const { highlightCode } = await import("/__highlight-test.js");
      const cases = [
        ["python", "def greet():\n    return '<script>café 🦀</script>'"],
        ["typescript", "const answer: number = 42;"],
        ["nix", "{ pkgs }: pkgs.mkShell { packages = [ pkgs.rustc ]; }"],
        ["rust", "fn main() { println!(\"hello\"); }"],
      ];
      const results = [];
      for (const [language, source] of cases) {
        const html = await highlightCode(language, source);
        const node = document.createElement("div");
        node.innerHTML = html ?? "";
        results.push({ language, source, text: node.textContent, tokens: node.children.length, scripts: node.querySelectorAll("script").length });
      }
      results.push({ unknown: await highlightCode("not-a-language", "<script>literal</script>") });
      return results;
    });
    for (const result of results.slice(0, -1)) {
      assert.equal(result.text, result.source, result.language);
      assert.ok(result.tokens > 0, `${result.language} was not highlighted`);
      assert.equal(result.scripts, 0);
    }
    assert.equal(results.at(-1).unknown, null);
  } finally { await context.close(); }
});

const timestamp = [2026, 1, 0, 0, 0, 0, 0, 0, 0];
const guest = { id: 1, github_id: 1, name: "Browser Test", username: "browser-test", created_at: timestamp, updated_at: timestamp };
const signaturePage = { entries: [{ id: 1, message: "A cached visitor message", signature: null, author_id: 1, author_username: "browser-test", created_at: timestamp, updated_at: timestamp }], next_cursor: null, total: 1 };
const json = (route, value) => route.fulfill({ contentType: "application/json", body: JSON.stringify(value) });

test("guestbook loads public cards before auth and reuses them on return navigation", async () => {
  const context = await browser.newContext();
  let releaseAuth;
  const authGate = new Promise((resolve) => { releaseAuth = resolve; });
  let pageReads = 0;
  try {
    await context.route("**/api/load_guestbook_user*", async (route) => { await authGate; await json(route, "Unauthenticated"); });
    await context.route("**/api/load_signatures*", (route) => { pageReads++; return json(route, signaturePage); });
    const page = await context.newPage();
    await page.goto(`${origin}/guestbook`);
    await page.getByText("A cached visitor message", { exact: true }).waitFor();
    assert.equal(await page.getByText("checking sign-in…", { exact: true }).count(), 1);
    releaseAuth();
    await page.getByRole("link", { name: "sign in with github" }).waitFor();
    const initialReads = pageReads;
    await page.locator('a[href="/blog"]').first().click();
    await page.getByRole("heading", { name: "blog", exact: true }).waitFor();
    await page.locator('a[href="/guestbook"]').first().click();
    await page.getByText("A cached visitor message", { exact: true }).waitFor();
    assert.equal(pageReads, initialReads, "fresh return navigation must not reload the public page");
  } finally { releaseAuth(); await context.close(); }
});

test("article controls do not wait for database-backed discussion", async () => {
  const context = await browser.newContext();
  let release;
  const gate = new Promise((resolve) => { release = resolve; });
  try {
    await context.route("**/api/load_reactions*", async (route) => { await gate; await json(route, { post: [], comments: {} }); });
    const page = await context.newPage();
    await page.goto(`${origin}/blog`);
    await page.getByRole("link", { name: /Blog rendering showcase/ }).click();
    const block = page.locator(".code-block").first();
    await block.getByRole("button", { name: "Wrap lines", exact: true }).click({ timeout: 5000 });
    assert.match(await block.getAttribute("class"), /code-wrap/);
  } finally { release(); await context.close(); }
});

test("reply composer opens beside its comment, focuses, and preserves the draft on cancel", async () => {
  const context = await browser.newContext();
  try {
    await context.route("**/api/get_user*", (route) => json(route, guest));
    await context.route("**/api/load_reactions*", (route) => json(route, { post: [], comments: {} }));
    await context.route("**/api/load_comments*", (route) => json(route, [
      { id: 1, parent_id: null, author: { username: "visitor", github_id: 2, is_owner: false }, body_html: "<p>First discussion comment</p>", created_at: timestamp },
      { id: 2, parent_id: null, author: { username: "visitor", github_id: 2, is_owner: false }, body_html: "<p>Second discussion comment</p>", created_at: timestamp },
    ]));
    const page = await context.newPage();
    // Enter through the client router so the API boundary can be controlled without a test login route.
    await page.goto(`${origin}/blog`);
    await page.getByRole("link", { name: /Blog rendering showcase/ }).click();
    await page.getByRole("textbox", { name: "Comment", exact: true }).waitFor();
    await page.getByRole("button", { name: "reply", exact: true }).first().click();
    const reply = page.getByRole("textbox", { name: "Reply", exact: true });
    await reply.waitFor();
    assert.ok(await reply.evaluate((element) => document.activeElement === element));
    const first = await page.getByText("First discussion comment", { exact: true }).boundingBox();
    const second = await page.getByText("Second discussion comment", { exact: true }).boundingBox();
    const composer = await reply.boundingBox();
    assert.ok(first.y < composer.y && composer.y < second.y, "reply must appear between the selected comment and the next thread");
    await reply.fill("An unfinished reply");
    await page.getByRole("button", { name: "cancel", exact: true }).click();
    assert.equal(await page.getByRole("textbox", { name: "Comment", exact: true }).inputValue(), "An unfinished reply");
  } finally { await context.close(); }
});

test("the command palette focuses its search every time it opens", async () => {
  const context = await browser.newContext();
  try {
    const page = await context.newPage();
    await page.goto(origin);
    await hydrated(page);
    const search = page.getByRole("textbox", { name: "where to?", exact: true });
    await assert.doesNotReject(async () => {
      for (let tries = 0; ; tries++) {
        await page.keyboard.press("/");
        try { return await search.waitFor({ timeout: 500 }); } catch (error) { if (tries === 10) throw error; }
      }
    }, "the window shortcuts install after hydration");
    await page.keyboard.press("Escape");
    await search.waitFor({ state: "detached" });
    for (let attempt = 0; attempt < 2; attempt++) {
      await page.keyboard.press("/");
      await search.waitFor();
      assert.ok(await search.evaluate((element) => document.activeElement === element));
      await page.keyboard.press("Escape");
      await search.waitFor({ state: "detached" });
    }
  } finally { await context.close(); }
});

test("drawing a signature keeps the chosen ink", async () => {
  const context = await browser.newContext();
  try {
    await context.route("**/api/load_guestbook_user*", (route) => json(route, { Authenticated: { guest, entry: null } }));
    await context.route("**/api/load_signatures*", (route) => json(route, signaturePage));
    const page = await context.newPage();
    await page.goto(`${origin}/guestbook`);
    await hydrated(page);
    await page.getByRole("button", { name: "sign guestbook", exact: true }).click();
    const green = page.getByRole("radio", { name: "alien green", exact: true });
    await green.click();
    const pad = await page.locator("canvas").boundingBox();
    await page.mouse.move(pad.x + 20, pad.y + 20);
    await page.mouse.down();
    await page.mouse.move(pad.x + 80, pad.y + 60, { steps: 5 });
    await page.mouse.up();
    assert.equal(await green.getAttribute("aria-checked"), "true");
  } finally { await context.close(); }
});

test("loading more signatures continues the card grid without a gap", async () => {
  const context = await browser.newContext();
  let releaseNextPage;
  const nextPage = new Promise((resolve) => { releaseNextPage = resolve; });
  try {
    const entry = (id) => ({ ...signaturePage.entries[0], id, message: `Visitor message ${id}` });
    await context.route("**/api/load_guestbook_user*", (route) => json(route, "Unauthenticated"));
    await context.route("**/api/load_signatures*", async (route) => {
      if (route.request().postData()?.includes("created_at")) {
        await nextPage;
        return json(route, { entries: [entry(3)], next_cursor: null, total: 3 });
      }
      return json(route, { entries: [entry(2), entry(1)], next_cursor: { created_at: timestamp, id: 1 }, total: 3 });
    });
    const page = await context.newPage();
    await page.setViewportSize({ width: 1280, height: 600 });
    await page.goto(`${origin}/guestbook`);
    await page.getByText("Visitor message 1", { exact: true }).waitFor();
    await page.mouse.wheel(0, 4000);
    const placeholder = page.locator("article.animate-pulse").first();
    await placeholder.waitFor();
    const card = await page.locator("article.card:not(.animate-pulse)").last().boundingBox();
    const skeleton = await placeholder.boundingBox();
    assert.ok(skeleton.y - (card.y + card.height) <= 12, `placeholder sits ${skeleton.y - (card.y + card.height)}px below the cards`);
    releaseNextPage();
    await page.getByText("Visitor message 3", { exact: true }).waitFor();
  } finally { releaseNextPage(); await context.close(); }
});

