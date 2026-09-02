---
title: "Blog foundation notes"
description: "A private draft used to verify the blog authoring and rendering pipeline."
published: 2026-09-03
draft: true
tags:
  - rust
  - dioxus
---

This draft exercises the production blog pipeline without publishing placeholder content.

## Code highlighting

Fenced Rust blocks are highlighted with Tree-sitter during the build, so the browser receives ordinary HTML and CSS—no highlighting library or grammar is shipped to visitors.

```rust
fn published(draft: bool) -> bool {
    !draft
}
```

## Markdown

GitHub-flavoured tables, task lists, footnotes, and explicit heading attributes are supported.

| Concern | Choice |
| --- | --- |
| Content | Markdown in Git |
| Highlighting | Build-time Tree-sitter |

- [x] safe links
- [x] semantic HTML
- [x] responsive code blocks

<GcCalculator />

This note stays hidden while `draft` is `true`.[^draft]

[^draft]: Drafts are excluded from the index, direct routes, feeds, and sitemaps.
