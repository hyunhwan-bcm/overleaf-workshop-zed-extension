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
- Slash command: `/overleaf-compile`
  - Input format: `<server-url> <project-id> <cookie-header>`
  - Example:
    - `/overleaf-compile https://www.overleaf.com 1234567890abcdef12345678 overleaf_session2=<cookie>`
  - Behavior:
    - Reads CSRF token from `/project/<project-id>`.
    - Triggers compile via `POST /project/<project-id>/compile?auto_compile=true`.
    - Returns compile status and output file links from the compile response.

## Local Development

1. Build check:
   - `cargo check`
2. Install in Zed:
   - `zed: install dev extension`
   - Select this folder.
3. Open assistant panel and run:
   - `/overleaf-projects https://www.overleaf.com overleaf_session2=<cookie>`
   - `/overleaf-compile https://www.overleaf.com <project-id> overleaf_session2=<cookie>`

## Notes

- This does not attempt one-to-one VS Code parity.
- Slash command arguments are visible in assistant history; use short-lived cookies.
- Large VS Code features (virtual filesystem, custom explorer trees, PDF webview,
  collaboration cursors/chat, SCM/history sidebars) need follow-up Zed-specific
  designs.
