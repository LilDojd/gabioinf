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
- **Styling**: Tailwind CSS

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

Run `just prepare-sqlx` after changing a SQL query or migration.

### Writing blog posts

Create a draft with `just new-post my-post`, then edit the generated Markdown in
`content/blog/`. Frontmatter is validated at compile time, drafts are excluded from
routes and feeds, and posts are sorted newest-first. Set `draft: false` to publish.

Post bodies use GitHub-flavoured Markdown and should start at heading level 2 because
the page supplies the title. Reading time is estimated automatically at 200 words per
minute. Rust code fences are highlighted with Tree-sitter during the build; unsupported
fence languages remain readable plain text without shipping a highlighting runtime or
JavaScript library to the browser.

Two allowlisted Dioxus elements may appear on their own line:

```md
<GcCalculator />
<Video src="https://example.com/demo.mp4" title="Optional caption" />
```

Unknown elements, unknown attributes, unsafe URLs, and arbitrary raw HTML fail the build.
Add a new variant to `PostBlock` and its build-time parser when another component is
actually needed.

Published posts load Giscus comments lazily from the repository's GitHub Discussions.
The post slug is the stable discussion key; changing it starts a new comment thread.
Giscus is the blog's only handwritten-JavaScript exception.

Run `just check-posts` before publishing.

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
