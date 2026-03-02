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
- Slash command: `/overleaf-errors`
  - Input format: `<server-url> <project-id> <cookie-header>`
  - Example:
    - `/overleaf-errors https://www.overleaf.com 1234567890abcdef12345678 overleaf_session2=<cookie>`
  - Behavior:
    - Triggers compile with the same endpoint as `/overleaf-compile`.
    - Fetches `output.log` from compile artifacts.
    - Summarizes unique error (`! ...`) and warning (`Warning:`) lines.
- Slash command: `/overleaf-set-context`
  - Input format:
    - `<project-id> <session-id>` (defaults server to `https://www.overleaf.com`)
    - or `<server-url> <project-id> <session-id>`
  - Examples:
    - `/overleaf-set-context 699f54729b18bea9d5fbf71d <session-id>`
    - `/overleaf-set-context https://www.overleaf.com 699f54729b18bea9d5fbf71d overleaf_session2=<cookie>`
  - Behavior:
    - Stores server/project/session in extension memory for this Zed session.
    - After setting context, `/overleaf-projects`, `/overleaf-compile`, and `/overleaf-errors` can be run with no arguments.

## Local Development

1. Build check:
   - `cargo check`
2. Install in Zed:
   - `zed: install dev extension`
   - Select this folder.
3. Open assistant panel and run:
   - `/overleaf-set-context <project-id> <session-id>`
   - `/overleaf-projects https://www.overleaf.com overleaf_session2=<cookie>`
   - `/overleaf-compile https://www.overleaf.com <project-id> overleaf_session2=<cookie>`
   - `/overleaf-errors https://www.overleaf.com <project-id> overleaf_session2=<cookie>`
   - Or, after setting context:
     - `/overleaf-projects`
     - `/overleaf-compile`
     - `/overleaf-errors`

## Notes

- This does not attempt one-to-one VS Code parity.
- Slash command arguments are visible in assistant history; use short-lived cookies.
- Large VS Code features (virtual filesystem, custom explorer trees, PDF webview,
  collaboration cursors/chat, SCM/history sidebars) need follow-up Zed-specific
  designs.
