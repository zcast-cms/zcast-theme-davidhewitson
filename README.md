<p align="center">
  <img src=".github/logo-banner.svg" alt="zcast-theme-davidhewitson — Editorial dark/light sliding-panes theme for zcast" width="880" />
</p>

Editorial dark/light/auto sliding-panes theme for [`zcast`](https://github.com/zcast-cms/zcast), inspired by the `Libre Franklin` typography and `#1d1d1d` dark aesthetic of `davidhewitson.co.uk`.

## Features (`v0.1.2`)

- **Colour-scheme lock:** Supports `dark`-only, `light`-only, or `auto` (system preference + header toggle), server-rendered via `<html data-theme="...">` for zero flash-of-wrong-theme.
- **Sliding panes:** Clicking any `[data-zcast-note]` link slides the linked card open to the right, syncs the URL (`/<root>?stack=a,b,c`), truncates right-hand panes when branching from a middle pane, pulses already-open cards rather than duplicating, and supports native horizontal trackpad scrolling.
- **Closeable non-root panes:** Subtle `×` close button on every card except the leftmost root pane, cleanly syncing scroll position, breadcrumbs, title, and `?stack=...`.
- **Breadcrumbs & Backlinks:** Interactive breadcrumb trail and per-card **Linked from** section.
- **Standard `--zcast-*` CSS Design Tokens & UI Slots:** Full dark/light palette, high-contrast selection tokens (`--zcast-selection-bg` / `--zcast-selection-text`), flat pill header button chrome (`.zcast-header-btn`), and `window.ZcastPanes` (`openSlug`, `closePane`, `appendCardHtml`, `updateCardMeta`, `rebuildBreadcrumbs`, `scrollToPane`). Decoupled from `zcast-kernel`; depends solely on [`zcast-interfaces`](https://github.com/zcast-cms/zcast-interfaces).
