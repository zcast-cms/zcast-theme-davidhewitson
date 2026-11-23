# `zcast-theme-davidhewitson`

Editorial dark/light/auto sliding-panes theme for [`zcast`](https://github.com/zcast-cms/zcast), inspired by the `Libre Franklin` typography and `#1d1d1d` dark aesthetic of `davidhewitson.co.uk`.

## Features
- **Colour scheme lock:** Supports `dark`-only, `light`-only, or `auto` (system preference + toggle), server-rendered via `<html data-theme="...">` for zero flash-of-wrong-theme.
- **Sliding panes:** Clicking any `[data-zcast-note]` link opens the linked card to the right, scrolls smoothly, syncs the URL (`/<root>?stack=a,b,c`), truncates right-hand panes when branching from a middle pane, and scrolls to already-open cards rather than duplicating.
- **Breadcrumbs & Backlinks:** Interactive breadcrumb trail and per-card "Linked from" section.
- **Standard `--zcast-*` CSS Design Tokens & UI Slots:** Decoupled from `zcast-kernel`; depends solely on [`zcast-interfaces`](https://github.com/zcast-cms/zcast-interfaces).
