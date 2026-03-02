# Overleaf Workshop (Zed)

This repository is a first-slice port of the VS Code extension
[`overleaf-workshop`](https://github.com/overleaf-workshop/Overleaf-Workshop)
to Zed.

## Implemented v1 Slice

- Slash command: `/overleaf-projects`
  - Input format: `<server-url> <cookie-header>`
  - Example:
    - `/overleaf-projects https://www.overleaf.com overleaf_session2=<cookie>`
  - Behavior:
    - Validates login by requesting `/project` and checking `ol-user_id`.
    - Fetches project list from `/user/projects`.
    - Returns a Markdown table with project status and direct project URLs.

## Local Development

1. Build check:
   - `cargo check`
2. Install in Zed:
   - `zed: install dev extension`
   - Select this folder.
3. Open assistant panel and run:
   - `/overleaf-projects https://www.overleaf.com overleaf_session2=<cookie>`

## Notes

- This does not attempt one-to-one VS Code parity.
- Slash command arguments are visible in assistant history; use short-lived cookies.
- Large VS Code features (virtual filesystem, custom explorer trees, PDF webview,
  collaboration cursors/chat, SCM/history sidebars) need follow-up Zed-specific
  designs.
