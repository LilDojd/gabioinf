# 🛸 [gabioinf.dev](https://gabioinf.dev/)

A personal website built with Dioxus and wasm, showcasing projects, blog (coming soon), guestbook, and more.

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
