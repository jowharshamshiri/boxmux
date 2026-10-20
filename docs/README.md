# BoxMux documentation site

The source of [boxmux.com](https://boxmux.com): a static SvelteKit site whose pages are written
in Markdown (rendered with MDsveX) and styled with Tailwind CSS.

## Layout

```
docs/
├── src/
│   ├── content/        the documentation pages, one Markdown file per page
│   ├── lib/
│   │   ├── config.ts   site title, quick links, home page features
│   │   └── components/ layout, navigation and UI components
│   └── routes/         the home page and /docs/[...slug]
├── static/             images, logos, favicon and CNAME (boxmux.com)
└── svelte.config.js    static adapter; output in docs/build
```

Each file in `src/content/` becomes a page at `/docs/<file name>` (for example
`src/content/plugin-system.md` is `/docs/plugin-system`). A file in a subdirectory with an
`index.md` is served at the directory's name.

## Writing a page

Every page starts with frontmatter; pages without it are not served:

```markdown
---
title: Plugin System
description: One sentence describing the page.
---

## First section

...
```

Link to other pages with `/docs/<name>`. Describe only what the code in this repository does,
and make every complete YAML example one that BoxMux loads.

## Commands

You need Node.js 20 or newer.

```bash
cd docs
npm ci            # install the dependencies from package-lock.json
npm run dev       # local preview with live reload
npm run build     # build the static site into docs/build
npm run preview   # serve the built site
npm run check     # type-check the Svelte and TypeScript code
```

## Publishing

`scripts/deploy-docs.sh` (from the repository root) builds the site and publishes `docs/build`
to the `gh-pages` branch, which serves boxmux.com. `scripts/deploy-docs.sh --dry-run` builds and
commits to a local `gh-pages` worktree without pushing. The script refuses to run while `docs/`
has uncommitted changes.
