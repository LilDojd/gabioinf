# 🛸 [gabioinf.dev](https://gabioinf.dev/)

A personal website built with Dioxus and WebAssembly, showcasing projects, writing, a guestbook, and more.

[![Dioxus](https://img.shields.io/badge/Dioxus-0.7.10-blue.svg)](https://dioxuslabs.com/)
[![MIT licensed](https://img.shields.io/github/license/LilDojd/gabioinf)](./LICENSE)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/LilDojd/gabioinf/fly-deploy.yml?label=deployment)](https://github.com/LilDojd/gabioinf/deployments)

## 🛠️ Stack

- **Frontend**: [Dioxus](https://dioxuslabs.com/) - A Rust-based framework for building cross-platform UI
- **Backend**: [Axum](https://github.com/tokio-rs/axum) - A modular web framework
- **Database**: PostgreSQL with [SQLx](https://github.com/launchbadge/sqlx)
- **Authentication**: OAuth2 with GitHub
- **Styling**: Tailwind CSS v4 (design tokens and utilities live in `input.css`); fonts are self-hosted
- **Rust UI**: interactions use Dioxus + `web-sys`; a small JavaScript bridge loads Arborium’s WASM grammar plugins

## 🚀 Deployment

- Deployed on [Fly.io](https://fly.io/)
- DB hosted on [Neon](https://neon.tech)

## 🏁 Getting Started

1. Install [devenv](https://devenv.sh/getting-started/) and enter the shell with `devenv shell`.
2. Store the GitHub OAuth secret locally:

   ```sh
   secretspec set GABIOINF_SECRET --provider local
   ```

3. Run `just serve`. This starts local PostgreSQL and the Dioxus development server with process-scoped secrets.

To invoke Dioxus directly:

```sh
devenv processes up postgres --detach
secretspec run --scope app -- env DATABASE_URL="$DATABASE_URL" dx serve
```

Run `just prepare-sqlx` after changing a SQL query or migration. `just seed` fills the
local database with fake guests, signatures and comments (regenerate the SQL with
`python3 fixtures/generate.py > fixtures/synthetic.sql`).

### Writing blog posts

Create a draft with `just new-post my-post`, then edit the generated Markdown in
`content/blog/`. Frontmatter is validated at compile time, drafts are excluded from
routes and feeds, and posts are sorted newest-first. Set `draft: false` to publish.

Post bodies use GitHub-flavoured Markdown and should start at heading level 2 because
the page supplies the title. Reading time is estimated automatically at 200 words per
minute. Code is rendered as escaped, readable text first. Arborium loads Tree-sitter
WASM grammars on demand from version-pinned jsDelivr assets (`2.18.1`), reusing loaded
grammars across blocks and client-side navigation. The full published grammar catalog
is available without Cargo language features. Missing grammars or network failures leave
plain text usable. Nested Markdown and comment code blocks are highlighted too.

Top-level code fences use a Rust viewer with keyboard-accessible line numbers, wrapping,
per-block line permalinks (for example `#blog-my-post-code-2-L3-L7`), and clipboard success
or failure feedback. Shift-click extends a range; Escape clears it. The fence may name
the file and emphasize lines: ```` ```rust title="src/main.rs" {2,5-7} ````.

Two allowlisted Dioxus elements may appear on their own line at the top level (not inside a list or quotation):

```md
<GcCalculator />
<Video src="https://example.com/demo.mp4" title="Optional caption" />
```

Unknown elements, unknown attributes, unsafe URLs, and arbitrary raw HTML fail the build.
Add a new variant to `PostBlock` and its build-time parser when another component is
actually needed.

Published posts have first-party comments backed by PostgreSQL. GitHub sign-in keeps
bots out; comments support Markdown and one level of replies, and posting is rate limited.

Run `just check-posts` before publishing.

### Guestbook loading and moderation

The guestbook shell renders immediately; public signatures and sign-in status load
independently. A per-app, in-memory first-page cache makes return navigation immediate
for up to 30 seconds. Stale cards stay visible during refresh and retry. The cache holds
at most 10 entries / 1 MiB, never caches authentication, and is invalidated after successful
writes or identity changes. It does not eliminate the initial cross-region database trip.

Guestbook messages and comments reject **severe-only** content using rustrict. Mild and
moderate language remains allowed. Comments check both raw Markdown and decoded visible
text so entities and link formatting cannot bypass the same policy.

### Checks

- `just check`: formatting, strict Clippy, and the WASM build.
- `just test`: start local PostgreSQL and run Rust unit/integration tests.
- `just test-browser`: run against an already-running `just serve-test-content` on port 8080.
  This explicitly enables the debug-only `test-content` fixture catalog; normal builds
  never publish those posts. Rust tests use the same fixtures independently of live content.
  Run `npx playwright install chromium` once, or use
  `PLAYWRIGHT_CHANNEL=chrome just test-browser` with installed Chrome.
  `APP_URL` overrides the test server URL. The grammar smoke test needs CDN access.

Tests follow [Testing on the Toilet’s behavior-first guidance](https://testing.googleblog.com/2015/01/testing-on-toilet-change-detector-tests.html):
exercise outcomes and public boundaries, not copies of implementation details or
whole-page snapshots. Browser checks cover failed grammar loads, clipboard errors,
per-block selection, and real multi-language highlighting.

### Publishing secrets to Fly.io

SecretSpec's Fly provider requires SecretSpec 0.20 or newer and an authenticated `flyctl`:

```sh
just publish-fly-secret DATABASE_URL
just publish-fly-secret GABIOINF_SECRET
just publish-fly-secret SESSION_SECRET
just publish-fly-secret SENTRY_DSN # optional
```

Fly secrets are write-only, so existing values cannot be pulled back into the local provider.

## 📝 License

This project is [MIT](https://opensource.org/licenses/MIT) licensed.

## 👤 Author

**Georgiy Andreev**

- Website (this one): [gabioinf.dev](https://gabioinf.dev)
- GitHub: [@LilDojd](https://github.com/LilDojd)
- LinkedIn: [@georgiy-andreev](https://linkedin.com/in/georgiy-andreev)

🙏 Acknowledgements and inspiration

- [duncan.land](https://duncan.land/)
- [Tania Rasca](https://www.taniarascia.com/)
- Liza Korkunova
- My wife who drew the visuals
