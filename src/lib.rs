//! `zcast-theme-davidhewitson` — the editorial sliding-panes theme for `zcast`.
//!
//! Implements the `zcast:ui/theme@1.0` contract from `zcast-interfaces` using
//! [`maud`] for server-side rendering and [`rust_embed`] for self-hosted assets
//! (CSS, sliding-panes JS and the six OFL Libre Franklin WOFF2 faces).
//!
//! Highlights:
//! - **Colour-scheme lock** (`dark` / `light` / `auto`). Locked modes are baked
//!   into `<html data-theme>` so there is zero flash-of-wrong-theme and no
//!   toggle. `auto` restores the stored preference from `localStorage` (or the
//!   system preference) in a tiny inline script *before first paint* and then
//!   exposes a header toggle.
//! - **Horizontal sliding panes** with breadcrumbs, backlinks and URL sync via
//!   `?stack=a,b,c`.
//! - **Five standard UI slots** (`head`, `header_actions`, `card_toolbar`,
//!   `card_footer`, `body_end`) plus the `data-zcast-*` DOM conventions so
//!   independent modules (`zcast-popover`, `zcast-cmd-k`, `zcast-editor-inline`)
//!   drop in without coupling.

use maud::{DOCTYPE, Markup, PreEscaped, html};
use rust_embed::RustEmbed;
use zcast_interfaces::{
    Asset, BreadcrumbItem, CardRenderContext, CardView, Html, IFACE_THEME, InterfaceReq,
    InterfaceSpec, ModuleManifest, PageContext, ThemeColorMode, ThemeProvider,
};

/// Embedded theme assets (`assets/theme.css`, `assets/panes.js`, `assets/fonts/*.woff2`).
#[derive(RustEmbed)]
#[folder = "assets/"]
struct ThemeAssets;

const THEME_ID: &str = "zcast-theme-davidhewitson";
const THEME_NAME: &str = "David Hewitson";
const THEME_VERSION: &str = env!("CARGO_PKG_VERSION");
const THEME_DESCRIPTION: &str =
    "Editorial dark/light/auto sliding-panes theme inspired by davidhewitson.co.uk";

const THEME_CSS_HREF: &str = "/_zcast/theme/theme.css";
const PANES_JS_HREF: &str = "/_zcast/theme/panes.js";
const CUSTOM_CSS_HREF: &str = "/_zcast/custom.css";

/// Font preloaded for the first paint (regular weight, latin subset).
const PRELOAD_FONT: &str = "/_zcast/theme/fonts/libre-franklin-v12-latin-ext_latin-regular.woff2";

/// Inline bootstrap that resolves `auto` mode *before* the first paint.
///
/// It is only emitted for [`ThemeColorMode::Auto`]; locked themes are fully
/// determined by the server-rendered `data-theme` attribute.
const AUTO_THEME_SCRIPT: &str = r#"(function(){var r=document.documentElement,s=null;try{s=localStorage.getItem('zcast-color-mode');}catch(e){}if(s!=='dark'&&s!=='light'){s=(window.matchMedia&&window.matchMedia('(prefers-color-scheme: light)').matches)?'light':'dark';}r.setAttribute('data-theme',s);})();"#;

static PROVIDES: &[InterfaceSpec] = &[InterfaceSpec {
    name: IFACE_THEME,
    version: "1.0.0",
}];
static REQUIRES: &[InterfaceReq] = &[];
static RECOMMENDS: &[&str] = &[];

/// The first-party `zcast` theme matching `davidhewitson.co.uk`.
#[derive(Debug, Clone, Copy, Default)]
pub struct DavidHewitsonTheme;

impl DavidHewitsonTheme {
    /// Create a new theme instance.
    pub fn new() -> Self {
        Self
    }
}

impl ThemeProvider for DavidHewitsonTheme {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            id: THEME_ID,
            name: THEME_NAME,
            version: THEME_VERSION,
            description: THEME_DESCRIPTION,
            provides: PROVIDES,
            requires: REQUIRES,
            recommends: RECOMMENDS,
        }
    }

    fn render_page(&self, ctx: &PageContext) -> Html {
        let breadcrumbs = if ctx.breadcrumbs.is_empty() {
            ctx.cards
                .iter()
                .map(|card| BreadcrumbItem {
                    slug: card.slug.clone(),
                    title: card.title.clone(),
                })
                .collect()
        } else {
            ctx.breadcrumbs.clone()
        };

        let page_title = breadcrumbs
            .last()
            .map(|item| item.title.as_str())
            .or_else(|| ctx.cards.first().map(|card| card.title.as_str()));

        let main = html! {
            main id="zcast-main" class="zcast-main" {
                div class="zcast-panes" data-zcast-panes role="region" aria-label="Notes" {
                    @for card in &ctx.cards {
                        (render_card_markup(
                            card,
                            &ctx.slots.card_toolbar,
                            &ctx.slots.card_footer,
                            ctx.is_admin,
                        ))
                    }
                }
            }
        };

        render_document(DocumentShell {
            site_title: &ctx.site_title,
            page_title,
            color_mode: ctx.color_mode,
            is_admin: ctx.is_admin,
            breadcrumbs: &breadcrumbs,
            header_actions: &ctx.slots.header_actions,
            head_slot: &ctx.slots.head,
            custom_css_enabled: ctx.custom_css_enabled,
            body_end: &ctx.slots.body_end,
            main,
        })
    }

    fn render_card(&self, card: &CardView, ctx: &CardRenderContext) -> Html {
        Html(
            render_card_markup(
                card,
                &ctx.card_toolbar_slot,
                &ctx.card_footer_slot,
                ctx.is_admin,
            )
            .into_string(),
        )
    }

    fn render_error(
        &self,
        status: u16,
        title: &str,
        message: &str,
        site_title: &str,
        color_mode: ThemeColorMode,
    ) -> Html {
        let main = html! {
            main id="zcast-main" class="zcast-main zcast-main--error" {
                section class="zcast-error" data-zcast-error data-zcast-error-status=(status) {
                    p class="zcast-error__status" { (status) }
                    h1 class="zcast-error__title" { (title) }
                    p class="zcast-error__message" { (message) }
                    p class="zcast-error__home" {
                        a class="zcast-error__home-link" href="/" { "Back to " (site_title) }
                    }
                }
            }
        };

        render_document(DocumentShell {
            site_title,
            page_title: Some(title),
            color_mode,
            is_admin: false,
            breadcrumbs: &[],
            header_actions: "",
            head_slot: "",
            custom_css_enabled: false,
            body_end: "",
            main,
        })
    }

    fn get_asset(&self, path: &str) -> Option<Asset> {
        let path = normalise_asset_path(path);
        let file = ThemeAssets::get(path)?;
        Some(Asset {
            content_type: content_type_for(path),
            bytes: file.data,
        })
    }
}

/// Strip a leading `/` and any known mount prefix so both the logical asset
/// key (`theme.css`) and the served URL (`/_zcast/theme/theme.css`) resolve.
fn normalise_asset_path(path: &str) -> &str {
    let mut path = path.trim().trim_start_matches('/');
    for prefix in ["_zcast/theme/assets/", "_zcast/theme/", "assets/"] {
        if let Some(rest) = path.strip_prefix(prefix) {
            path = rest;
            break;
        }
    }
    path
}

fn content_type_for(path: &str) -> &'static str {
    match path
        .rsplit('.')
        .next()
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    }
}

/// Everything the shared document shell needs, regardless of page vs. error.
struct DocumentShell<'a> {
    site_title: &'a str,
    page_title: Option<&'a str>,
    color_mode: ThemeColorMode,
    is_admin: bool,
    breadcrumbs: &'a [BreadcrumbItem],
    header_actions: &'a str,
    head_slot: &'a str,
    custom_css_enabled: bool,
    body_end: &'a str,
    main: Markup,
}

fn render_document(shell: DocumentShell<'_>) -> Html {
    let head = render_head(&shell);
    let body = render_body(&shell);

    let document = match shell.color_mode {
        ThemeColorMode::Dark => html! {
            (DOCTYPE)
            html lang="en" data-theme="dark" data-color-mode="dark" {
                (head)
                (body)
            }
        },
        ThemeColorMode::Light => html! {
            (DOCTYPE)
            html lang="en" data-theme="light" data-color-mode="light" {
                (head)
                (body)
            }
        },
        ThemeColorMode::Auto => html! {
            (DOCTYPE)
            html lang="en" data-color-mode="auto" {
                (head)
                (body)
            }
        },
    };

    Html(document.into_string())
}

fn render_head(shell: &DocumentShell<'_>) -> Markup {
    let full_title = match shell.page_title {
        Some(title) if !title.is_empty() && title != shell.site_title => {
            format!("{title} — {}", shell.site_title)
        }
        _ => shell.site_title.to_owned(),
    };

    let color_scheme = match shell.color_mode {
        ThemeColorMode::Dark => "dark",
        ThemeColorMode::Light => "light",
        ThemeColorMode::Auto => "dark light",
    };

    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1";
            title { (full_title) }
            meta name="color-scheme" content=(color_scheme);
            meta name="generator" content="zcast";
            @match shell.color_mode {
                ThemeColorMode::Dark => {
                    meta name="theme-color" content="#1d1d1d";
                }
                ThemeColorMode::Light => {
                    meta name="theme-color" content="#ffffff";
                }
                ThemeColorMode::Auto => {
                    meta name="theme-color" content="#ffffff" media="(prefers-color-scheme: light)";
                    meta name="theme-color" content="#1d1d1d" media="(prefers-color-scheme: dark)";
                }
            }
            link rel="preload" href=(PRELOAD_FONT) as="font" type="font/woff2" crossorigin;
            link rel="stylesheet" href=(THEME_CSS_HREF);
            @if shell.color_mode == ThemeColorMode::Auto {
                script { (PreEscaped(AUTO_THEME_SCRIPT)) }
            }
            (PreEscaped(shell.head_slot))
            @if shell.custom_css_enabled {
                link rel="stylesheet" href=(CUSTOM_CSS_HREF);
            }
        }
    }
}

fn render_body(shell: &DocumentShell<'_>) -> Markup {
    let body_class = if shell.is_admin {
        "zcast-body zcast-body--admin"
    } else {
        "zcast-body"
    };

    html! {
        body class=(body_class) data-zcast-color-mode=(shell.color_mode.as_str()) {
            a class="zcast-skip-link" href="#zcast-main" { "Skip to content" }
            header class="zcast-header" data-zcast-header {
                div class="zcast-header__inner" {
                    a class="zcast-site-title" href="/" data-zcast-site-title {
                        (shell.site_title)
                    }
                    (render_breadcrumbs(shell.breadcrumbs))
                    div class="zcast-header__actions" data-zcast-header-actions {
                        (PreEscaped(shell.header_actions))
                        @if shell.color_mode == ThemeColorMode::Auto {
                            (render_theme_toggle())
                        }
                    }
                }
            }
            (shell.main)
            script src=(PANES_JS_HREF) defer {}
            (PreEscaped(shell.body_end))
        }
    }
}

fn render_theme_toggle() -> Markup {
    html! {
        button
            type="button"
            class="zcast-theme-toggle"
            data-zcast-theme-toggle
            aria-label="Toggle colour scheme"
            title="Toggle colour scheme"
        {
            svg
                class="zcast-theme-toggle__icon zcast-theme-toggle__icon--moon"
                viewBox="0 0 24 24"
                width="18"
                height="18"
                aria-hidden="true"
                focusable="false"
            {
                path
                    d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8Z"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.6"
                    stroke-linejoin="round"
                {}
            }
            svg
                class="zcast-theme-toggle__icon zcast-theme-toggle__icon--sun"
                viewBox="0 0 24 24"
                width="18"
                height="18"
                aria-hidden="true"
                focusable="false"
            {
                circle cx="12" cy="12" r="4" fill="none" stroke="currentColor" stroke-width="1.6" {}
                path
                    d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.6"
                    stroke-linecap="round"
                {}
            }
        }
    }
}

fn render_breadcrumbs(items: &[BreadcrumbItem]) -> Markup {
    let last_index = items.len().saturating_sub(1);

    // The `<nav>` and its `<ol>` are always rendered (even when empty) so the
    // client-side `rebuildBreadcrumbs()` always has a stable container to fill
    // after opening a pane.
    html! {
        nav class="zcast-breadcrumbs" data-zcast-breadcrumbs aria-label="Breadcrumb" {
            ol class="zcast-breadcrumbs__list" {
                @for (index, item) in items.iter().enumerate() {
                    li class=(
                        if index == last_index { "zcast-breadcrumbs__item zcast-breadcrumbs__item--current" }
                        else { "zcast-breadcrumbs__item" }
                    ) {
                        a
                            class="zcast-breadcrumbs__link"
                            href=(format!("/{}", item.slug))
                            data-zcast-breadcrumb
                            data-zcast-breadcrumb-slug=(item.slug)
                            aria-current=(if index == last_index { "page" } else { "false" })
                        {
                            (item.title)
                        }
                    }
                }
            }
        }
    }
}

fn render_card_markup(card: &CardView, toolbar: &str, footer: &str, is_admin: bool) -> Markup {
    let show_toolbar = is_admin || !toolbar.is_empty();

    html! {
        article
            class="zcast-card"
            data-zcast-card
            data-zcast-card-id=(card.id.as_str())
            data-zcast-card-slug=(card.slug)
            data-zcast-card-private=(if card.private { "true" } else { "false" })
            data-zcast-card-admin=(if is_admin { "true" } else { "false" })
        {
            @if show_toolbar {
                div class="zcast-card__toolbar" data-zcast-card-toolbar {
                    (PreEscaped(toolbar))
                }
            }
            header class="zcast-card__header" {
                h1 class="zcast-card-title" data-zcast-card-title { (card.title) }
            }
            div class="zcast-card__body" {
                div class="zcast-card-body" data-zcast-card-body {
                    (PreEscaped(&card.rendered_html))
                }
            }
            footer class="zcast-card__footer" data-zcast-card-footer {
                (render_backlinks(&card.backlinks))
                (PreEscaped(footer))
            }
        }
    }
}

fn render_backlinks(backlinks: &[zcast_interfaces::BacklinkRef]) -> Markup {
    html! {
        @if !backlinks.is_empty() {
            section class="zcast-backlinks" data-zcast-backlinks {
                h2 class="zcast-backlinks__title" { "Linked from" }
                ul class="zcast-backlinks__list" {
                    @for backlink in backlinks {
                        li class="zcast-backlinks__item" {
                            a
                                class="zcast-backlinks__link"
                                href=(format!("/{}", backlink.slug))
                                data-zcast-note=(backlink.slug)
                            {
                                (backlink.title)
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SITE_TITLE: &str = "David Hewitson";

    fn card(slug: &str, title: &str, private: bool) -> CardView {
        CardView {
            id: zcast_interfaces::NoteId::from("01TESTCARD0000000000000000"),
            slug: slug.to_owned(),
            title: title.to_owned(),
            markdown_body: format!("# {title}"),
            rendered_html: format!("<p>Body for {title}.</p>"),
            private,
            backlinks: Vec::new(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn breadcrumbs(items: &[(&str, &str)]) -> Vec<BreadcrumbItem> {
        items
            .iter()
            .map(|(slug, title)| BreadcrumbItem {
                slug: (*slug).to_owned(),
                title: (*title).to_owned(),
            })
            .collect()
    }

    fn base_page(mode: ThemeColorMode) -> PageContext {
        PageContext {
            site_title: SITE_TITLE.to_owned(),
            color_mode: mode,
            is_admin: false,
            breadcrumbs: breadcrumbs(&[("hello", "Hello")]),
            cards: vec![card("hello", "Hello", false)],
            slots: Default::default(),
            custom_css_enabled: false,
        }
    }

    fn render(page: &PageContext) -> String {
        DavidHewitsonTheme::new().render_page(page).into_string()
    }

    #[test]
    fn manifest_declares_theme_interface() {
        let manifest = DavidHewitsonTheme::new().manifest();
        assert_eq!(manifest.id, "zcast-theme-davidhewitson");
        assert_eq!(manifest.provides.len(), 1);
        assert_eq!(manifest.provides[0].name, IFACE_THEME);
        assert_eq!(manifest.provides[0].version, "1.0.0");
        assert_eq!(manifest.requires, &[]);
        assert_eq!(manifest.requires.len(), 0);
    }

    #[test]
    fn default_matches_new() {
        let a: DavidHewitsonTheme = Default::default();
        let b = DavidHewitsonTheme::new();
        assert_eq!(a.manifest().id, b.manifest().id);
    }

    #[test]
    fn dark_mode_is_locked_with_zero_flash() {
        let html = render(&base_page(ThemeColorMode::Dark));

        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<html lang=\"en\" data-theme=\"dark\" data-color-mode=\"dark\">"));
        assert!(html.contains("name=\"color-scheme\" content=\"dark\""));
        assert!(html.contains("name=\"theme-color\" content=\"#1d1d1d\""));
        // Locked themes never ship a toggle, a stored-preference bootstrap, or
        // a system-preference probe (so there is no way to flash the wrong theme).
        assert!(!html.contains("data-zcast-theme-toggle"));
        assert!(!html.contains("prefers-color-scheme"));
        assert!(!html.contains("localStorage"));
        assert!(!html.contains("getItem"));
        assert!(!html.contains("data-theme=\"light\""));
    }

    #[test]
    fn light_mode_is_locked_with_zero_flash() {
        let html = render(&base_page(ThemeColorMode::Light));

        assert!(html.contains("<html lang=\"en\" data-theme=\"light\" data-color-mode=\"light\">"));
        assert!(html.contains("name=\"color-scheme\" content=\"light\""));
        assert!(html.contains("name=\"theme-color\" content=\"#ffffff\""));
        assert!(!html.contains("data-zcast-theme-toggle"));
        assert!(!html.contains("prefers-color-scheme"));
        assert!(!html.contains("data-theme=\"dark\""));
    }

    #[test]
    fn auto_mode_probes_preference_before_paint_and_offers_toggle() {
        let html = render(&base_page(ThemeColorMode::Auto));

        // `data-theme` is deliberately absent server-side so the inline script
        // is the single source of truth.
        assert!(html.contains("<html lang=\"en\" data-color-mode=\"auto\">"));
        assert!(!html.contains("data-theme="));
        assert!(html.contains("prefers-color-scheme"));
        assert!(html.contains("localStorage"));
        assert!(html.contains("zcast-color-mode"));
        assert!(html.contains("name=\"color-scheme\" content=\"dark light\""));
        assert!(html.contains("data-zcast-theme-toggle"));
        assert!(html.contains("class=\"zcast-theme-toggle\""));

        // The bootstrap must sit in `<head>` (before `</head>`) to beat first paint.
        let head_end = html.find("</head>").expect("closing head");
        let script_at = html.find("prefers-color-scheme").expect("bootstrap script");
        assert!(script_at < head_end);
    }

    #[test]
    fn page_renders_all_five_ui_slots() {
        let mut page = base_page(ThemeColorMode::Dark);
        page.is_admin = true;
        page.slots = zcast_interfaces::UiSlots {
            head: "<meta name=\"slot-head\" content=\"HEAD_SLOT\">".to_owned(),
            header_actions: "<a class=\"slot-header\" href=\"/admin\">HEADER_SLOT</a>".to_owned(),
            card_toolbar: "<button class=\"slot-toolbar\">TOOLBAR_SLOT</button>".to_owned(),
            card_footer: "<div class=\"slot-footer\">FOOTER_SLOT</div>".to_owned(),
            body_end: "<script data-slot=\"body-end\">BODY_END_SLOT</script>".to_owned(),
        };

        let html = render(&page);

        assert!(html.contains("HEAD_SLOT"));
        assert!(html.contains("HEADER_SLOT"));
        assert!(html.contains("TOOLBAR_SLOT"));
        assert!(html.contains("FOOTER_SLOT"));
        assert!(html.contains("BODY_END_SLOT"));

        // Slots land in their documented locations.
        let head_end = html.find("</head>").expect("closing head");
        assert!(html.find("HEAD_SLOT").unwrap() < head_end);

        let header_start = html.find("data-zcast-header-actions").unwrap();
        let header_end = html.find("</header>").unwrap();
        let header_slot = html.find("HEADER_SLOT").unwrap();
        assert!(header_start < header_slot && header_slot < header_end);

        let body_start = html.find("data-zcast-card-body").unwrap();
        let toolbar_slot = html.find("TOOLBAR_SLOT").unwrap();
        assert!(toolbar_slot < body_start);

        let footer_slot = html.find("FOOTER_SLOT").unwrap();
        assert!(body_start < footer_slot);

        let body_end = html.rfind("</body>").unwrap();
        let body_end_slot = html.find("BODY_END_SLOT").unwrap();
        assert!(body_end_slot < body_end);
    }

    #[test]
    fn custom_css_link_is_opt_in() {
        let mut page = base_page(ThemeColorMode::Dark);
        assert!(!render(&page).contains("/_zcast/custom.css"));

        page.custom_css_enabled = true;
        let html = render(&page);
        assert!(html.contains("<link rel=\"stylesheet\" href=\"/_zcast/custom.css\">"));

        // Loaded last inside head so it can override the theme stylesheet.
        let head_end = html.find("</head>").unwrap();
        let custom_css = html.find("/_zcast/custom.css").unwrap();
        assert!(custom_css < head_end);
    }

    #[test]
    fn cards_carry_standard_data_attributes() {
        let mut page = base_page(ThemeColorMode::Dark);
        page.is_admin = true;
        page.cards = vec![card("secret-note", "Secret Note", true)];
        page.breadcrumbs = breadcrumbs(&[("secret-note", "Secret Note")]);

        let html = render(&page);

        assert!(html.contains(
            "<article class=\"zcast-card\" data-zcast-card data-zcast-card-id=\"01TESTCARD0000000000000000\" data-zcast-card-slug=\"secret-note\" data-zcast-card-private=\"true\""
        ));
        assert!(
            html.contains("<h1 class=\"zcast-card-title\" data-zcast-card-title>Secret Note</h1>")
        );
        assert!(html.contains(
            "<div class=\"zcast-card-body\" data-zcast-card-body><p>Body for Secret Note.</p></div>"
        ));
        assert!(html.contains("data-zcast-card-toolbar"));
        assert!(html.contains("data-zcast-card-footer"));
        assert!(html.contains("data-zcast-panes"));
        assert!(html.contains("<main id=\"zcast-main\" class=\"zcast-main\">"));
        assert!(html.contains("href=\"#zcast-main\""));
    }

    #[test]
    fn public_card_marks_private_false() {
        let mut page = base_page(ThemeColorMode::Light);
        page.cards = vec![card("public-note", "Public Note", false)];
        let html = render(&page);
        assert!(html.contains("data-zcast-card-private=\"false\""));
        assert!(!html.contains("data-zcast-card-private=\"true\""));
    }

    #[test]
    fn breadcrumbs_are_clickable_trail() {
        let mut page = base_page(ThemeColorMode::Dark);
        page.breadcrumbs = breadcrumbs(&[("root", "Root"), ("middle", "Middle"), ("leaf", "Leaf")]);

        let html = render(&page);

        assert!(html.contains("data-zcast-breadcrumbs"));
        assert!(html.contains(
            "<a class=\"zcast-breadcrumbs__link\" href=\"/root\" data-zcast-breadcrumb data-zcast-breadcrumb-slug=\"root\" aria-current=\"false\">Root</a>"
        ));
        assert!(html.contains("data-zcast-breadcrumb-slug=\"leaf\" aria-current=\"page\""));
        assert!(html.contains("zcast-breadcrumbs__item--current"));
    }

    #[test]
    fn empty_breadcrumbs_fall_back_to_cards() {
        let mut page = base_page(ThemeColorMode::Auto);
        page.breadcrumbs = Vec::new();
        page.cards = vec![card("solo", "Solo", false)];

        let html = render(&page);
        assert!(html.contains("data-zcast-breadcrumb-slug=\"solo\""));
        assert!(html.contains(">Solo</a>"));
    }

    #[test]
    fn backlinks_render_as_linked_from_footer() {
        let mut page = base_page(ThemeColorMode::Dark);
        let mut note = card("hello", "Hello", false);
        note.backlinks = vec![
            zcast_interfaces::BacklinkRef {
                note_id: zcast_interfaces::NoteId::from("01BACKLINK00000000000000001"),
                slug: "citing-note".to_owned(),
                title: "Citing Note".to_owned(),
            },
            zcast_interfaces::BacklinkRef {
                note_id: zcast_interfaces::NoteId::from("01BACKLINK00000000000000002"),
                slug: "other-note".to_owned(),
                title: "Other Note".to_owned(),
            },
        ];
        page.cards = vec![note];

        let html = render(&page);

        assert!(html.contains("class=\"zcast-backlinks\" data-zcast-backlinks"));
        assert!(html.contains(">Linked from</h2>"));
        assert!(html.contains(
            "<a class=\"zcast-backlinks__link\" href=\"/citing-note\" data-zcast-note=\"citing-note\">Citing Note</a>"
        ));
        assert!(html.contains("data-zcast-note=\"other-note\""));
        assert!(html.contains("zcast-backlinks__list"));
    }

    #[test]
    fn no_backlinks_section_when_empty() {
        let page = base_page(ThemeColorMode::Dark);
        let html = render(&page);
        assert!(!html.contains("data-zcast-backlinks"));
        assert!(!html.contains("Linked from"));
    }

    #[test]
    fn render_card_standalone_includes_slots_and_attributes() {
        let theme = DavidHewitsonTheme::new();
        let ctx = CardRenderContext {
            is_admin: true,
            card_toolbar_slot: "<button>EDIT</button>".to_owned(),
            card_footer_slot: "<span>TAGS</span>".to_owned(),
        };

        let frag = theme
            .render_card(&card("fragment", "Fragment", false), &ctx)
            .into_string();

        assert!(frag.starts_with("<article class=\"zcast-card\" data-zcast-card"));
        assert!(frag.contains("data-zcast-card-slug=\"fragment\""));
        assert!(frag.contains("data-zcast-card-admin=\"true\""));
        assert!(frag.contains("data-zcast-card-title>Fragment</h1>"));
        assert!(frag.contains("<button>EDIT</button>"));
        assert!(frag.contains("<span>TAGS</span>"));
        assert!(!frag.contains("<html"));
    }

    #[test]
    fn render_card_hides_toolbar_for_public_readers() {
        let theme = DavidHewitsonTheme::new();
        let ctx = CardRenderContext::default();
        let frag = theme
            .render_card(&card("fragment", "Fragment", false), &ctx)
            .into_string();

        assert!(!frag.contains("data-zcast-card-toolbar"));
        assert!(!frag.contains("data-zcast-card-admin=\"true\""));
        assert!(frag.contains("data-zcast-card-admin=\"false\""));
    }

    #[test]
    fn error_page_is_branded_and_locked() {
        let theme = DavidHewitsonTheme::new();
        let html = theme
            .render_error(
                404,
                "Not found",
                "That note has wandered off.",
                SITE_TITLE,
                ThemeColorMode::Dark,
            )
            .into_string();

        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<html lang=\"en\" data-theme=\"dark\" data-color-mode=\"dark\">"));
        assert!(html.contains("data-zcast-error"));
        assert!(html.contains("data-zcast-error-status=\"404\""));
        assert!(html.contains("class=\"zcast-error__status\">404</p>"));
        assert!(html.contains("class=\"zcast-error__title\">Not found</h1>"));
        assert!(html.contains("That note has wandered off."));
        assert!(html.contains("Back to David Hewitson"));
        assert!(html.contains("<title>Not found — David Hewitson</title>"));
        assert!(!html.contains("data-zcast-theme-toggle"));
    }

    #[test]
    fn error_page_in_auto_mode_keeps_theme_bootstrap() {
        let theme = DavidHewitsonTheme::new();
        let html = theme
            .render_error(
                500,
                "Server error",
                "Boom.",
                SITE_TITLE,
                ThemeColorMode::Auto,
            )
            .into_string();

        assert!(html.contains("<html lang=\"en\" data-color-mode=\"auto\">"));
        assert!(html.contains("prefers-color-scheme"));
        assert!(html.contains("data-zcast-theme-toggle"));
    }

    #[test]
    fn page_title_combines_note_and_site() {
        let page = base_page(ThemeColorMode::Dark);
        let html = render(&page);
        assert!(html.contains("<title>Hello — David Hewitson</title>"));
    }

    #[test]
    fn html_escapes_user_content() {
        let mut page = base_page(ThemeColorMode::Dark);
        page.site_title = "A & B <script>".to_owned();
        let mut note = card("x", "Title <img onerror=x>", false);
        note.rendered_html = "<p>Safe body.</p>".to_owned();
        page.cards = vec![note];
        page.breadcrumbs = Vec::new();

        let html = render(&page);
        assert!(html.contains("A &amp; B &lt;script&gt;"));
        assert!(html.contains("Title &lt;img onerror=x&gt;"));
        assert!(!html.contains("<img onerror=x>"));
    }

    #[test]
    fn emitted_card_script_assets_are_linked() {
        let html = render(&base_page(ThemeColorMode::Dark));
        assert!(html.contains("<link rel=\"stylesheet\" href=\"/_zcast/theme/theme.css\">"));
        assert!(html.contains("<script src=\"/_zcast/theme/panes.js\" defer></script>"));
        assert!(html.contains("libre-franklin-v12-latin-ext_latin-regular.woff2"));
    }

    #[test]
    fn css_asset_is_embedded_with_tokens_and_font_faces() {
        let theme = DavidHewitsonTheme::new();
        let asset = theme.get_asset("theme.css").expect("theme.css embedded");
        assert_eq!(asset.content_type, "text/css; charset=utf-8");

        let css = std::str::from_utf8(&asset.bytes).expect("css is utf-8");
        assert!(css.contains("@font-face"));
        assert!(css.contains("Libre Franklin"));
        assert!(css.contains("/_zcast/theme/fonts/libre-franklin-v12-latin-ext_latin-300.woff2"));
        assert!(
            css.contains("/_zcast/theme/fonts/libre-franklin-v12-latin-ext_latin-regular.woff2")
        );
        assert!(
            css.contains("/_zcast/theme/fonts/libre-franklin-v12-latin-ext_latin-italic.woff2")
        );
        assert!(css.contains("/_zcast/theme/fonts/libre-franklin-v12-latin-ext_latin-500.woff2"));        assert!(css.contains("/_zcast/theme/fonts/libre-franklin-v12-latin-ext_latin-700.woff2"));
        assert!(
            css.contains("/_zcast/theme/fonts/libre-franklin-v12-latin-ext_latin-700italic.woff2")
        );

        // Dark tokens.
        assert!(css.contains("--zcast-bg: #1d1d1d"));
        assert!(css.contains("--zcast-surface: #252525"));
        assert!(css.contains("--zcast-surface-elevated: #2e2e2e"));
        assert!(css.contains("--zcast-text: rgba(255, 255, 255, 0.92)"));
        assert!(css.contains("--zcast-text-strong: #ffffff"));
        assert!(css.contains("--zcast-text-muted: rgba(255, 255, 255, 0.55)"));
        assert!(css.contains("--zcast-border: rgba(255, 255, 255, 0.13)"));
        assert!(css.contains("--zcast-accent: #ffffff"));
        assert!(css.contains("--zcast-code-bg: rgba(255, 255, 255, 0.12)"));

        // Light tokens.
        assert!(css.contains("--zcast-bg: #ffffff"));
        assert!(css.contains("--zcast-surface: #f7f7f7"));
        assert!(css.contains("--zcast-surface-elevated: #ffffff"));
        assert!(css.contains("--zcast-text: #111111"));
        assert!(css.contains("--zcast-text-strong: #000000"));
        assert!(css.contains("--zcast-text-muted: rgba(0, 0, 0, 0.55)"));
        assert!(css.contains("--zcast-border: rgba(0, 0, 0, 0.13)"));
        assert!(css.contains("--zcast-accent: #000000"));
        assert!(css.contains("--zcast-code-bg: rgba(0, 0, 0, 0.08)"));

        // Editorial layout tokens & mobile fallback.
        assert!(css.contains("--zcast-measure: 62ch"));
        assert!(css.contains("--zcast-pane-width: 620px"));
        assert!(css.contains("@media (max-width: 768px)"));
        assert!(css.contains(".zcast-breadcrumbs"));
        assert!(css.contains(".zcast-unresolved"));
    }

    #[test]
    fn js_asset_is_embedded_with_panes_api() {
        let theme = DavidHewitsonTheme::new();
        let asset = theme.get_asset("panes.js").expect("panes.js embedded");
        assert_eq!(asset.content_type, "application/javascript; charset=utf-8");

        let js = std::str::from_utf8(&asset.bytes).expect("js is utf-8");
        assert!(js.contains("window.ZcastPanes"));
        assert!(js.contains("appendCardHtml"));
        assert!(js.contains("updateCardMeta"));
        assert!(js.contains("rebuildBreadcrumbs"));
        assert!(js.contains("scrollToPane"));
        assert!(js.contains("data-zcast-note"));
        assert!(js.contains("popstate"));
        assert!(js.contains("pushState"));
        assert!(js.contains("zcast-card--highlight"));
        assert!(js.contains("metaKey"));
        assert!(js.contains("ctrlKey"));
        assert!(js.contains("shiftKey"));
        assert!(js.contains("/_zcast/cards/"));
    }

    #[test]
    fn woff2_fonts_are_embedded() {
        let theme = DavidHewitsonTheme::new();
        let fonts = [
            "libre-franklin-v12-latin-ext_latin-300.woff2",
            "libre-franklin-v12-latin-ext_latin-regular.woff2",
            "libre-franklin-v12-latin-ext_latin-italic.woff2",
            "libre-franklin-v12-latin-ext_latin-500.woff2",
            "libre-franklin-v12-latin-ext_latin-700.woff2",
            "libre-franklin-v12-latin-ext_latin-700italic.woff2",
        ];

        for font in fonts {
            let path = format!("fonts/{font}");
            let asset = theme
                .get_asset(&path)
                .unwrap_or_else(|| panic!("{path} embedded"));
            assert_eq!(asset.content_type, "font/woff2");
            assert!(asset.bytes.len() > 1024, "{path} should not be empty");
            assert_eq!(&asset.bytes[..4], b"wOF2", "{path} must be a WOFF2 file");

            // The served URL shape matches `@font-face` src URLs.
            let served = theme
                .get_asset(&format!("/_zcast/theme/fonts/{font}"))
                .expect("leading-slash lookup");
            assert_eq!(served.bytes.len(), asset.bytes.len());
        }
    }

    #[test]
    fn unknown_assets_are_none() {
        let theme = DavidHewitsonTheme::new();
        assert!(theme.get_asset("missing.css").is_none());
        assert!(theme.get_asset("fonts/nope.woff2").is_none());
        assert!(theme.get_asset("").is_none());
        assert!(theme.get_asset("../../etc/passwd").is_none());
    }

    #[test]
    fn asset_paths_tolerate_mount_prefixes() {
        let theme = DavidHewitsonTheme::new();
        let bare = theme.get_asset("theme.css").unwrap();
        for alias in [
            "/theme.css",
            "assets/theme.css",
            "/assets/theme.css",
            "/_zcast/theme/theme.css",
            "/_zcast/theme/assets/theme.css",
        ] {
            let resolved = theme.get_asset(alias).unwrap_or_else(|| panic!("{alias}"));
            assert_eq!(resolved.content_type, bare.content_type);
            assert_eq!(resolved.bytes.len(), bare.bytes.len());
        }
    }

    fn theme_css() -> String {
        let asset = DavidHewitsonTheme::new()
            .get_asset("theme.css")
            .expect("theme.css embedded");
        String::from_utf8(asset.bytes.to_vec()).expect("css is utf-8")
    }

    fn panes_js() -> String {
        let asset = DavidHewitsonTheme::new()
            .get_asset("panes.js")
            .expect("panes.js embedded");
        String::from_utf8(asset.bytes.to_vec()).expect("js is utf-8")
    }

    // Issue #3 / #5: the breadcrumb container must always exist so the client
    // can rebuild it, even when the server has no items to render.
    #[test]
    fn breadcrumb_nav_is_always_rendered() {
        let theme = DavidHewitsonTheme::new();
        let html = theme
            .render_error(
                404,
                "Not found",
                "Missing.",
                SITE_TITLE,
                ThemeColorMode::Dark,
            )
            .into_string();

        assert!(html.contains(
            "<nav class=\"zcast-breadcrumbs\" data-zcast-breadcrumbs aria-label=\"Breadcrumb\"><ol class=\"zcast-breadcrumbs__list\"></ol></nav>"
        ));

        // An empty `breadcrumbs` page still emits the container for the JS.
        let mut page = base_page(ThemeColorMode::Dark);
        page.cards = Vec::new();
        page.breadcrumbs = Vec::new();
        let empty = render(&page);
        assert!(empty.contains("data-zcast-breadcrumbs"));
        assert!(empty.contains("<ol class=\"zcast-breadcrumbs__list\"></ol>"));
    }

    // Issue #1: the card endpoint returns a JSON envelope, not raw HTML.
    #[test]
    fn panes_js_negotiates_json_card_payload() {
        let js = panes_js();

        assert!(js.contains("Accept: 'application/json, text/html'"));
        assert!(js.contains("canonical_slug"));
        assert!(js.contains("Content-Type"));
        assert!(js.contains("response.headers.get"));
        assert!(js.contains("parseCardJson"));
        assert!(js.contains("data.html"));
        // The raw body is only used as HTML when it is not a JSON envelope.
        assert!(js.contains("text/html"));
        assert!(js.contains("card.setAttribute('data-zcast-card-slug', payload.slug || slug)"));
    }

    // Issue #2: native trackpad horizontal scrolling must not be blocked.
    #[test]
    fn css_allows_horizontal_trackpad_scrolling() {
        let css = theme_css();

        // Pane-internal overscroll stops vertically but chains horizontally.
        assert!(css.contains("overscroll-behavior-y: contain;"));
        assert!(css.contains("overscroll-behavior-x: auto;"));
        assert!(!css.contains("overscroll-behavior: contain;"));

        // Snapping is gone and native scrolling stays crisp.
        assert!(!css.contains("scroll-snap-type"));
        assert!(!css.contains("scroll-snap-align"));
        assert!(css.contains("scroll-behavior: auto;"));
        assert!(!css.contains("scroll-behavior: smooth;"));
    }

    // Issue #5: newly opened panes slide in from the right.
    #[test]
    fn css_defines_card_slide_in_animation() {
        let css = theme_css();

        assert!(css.contains("@keyframes zcast-card-slide-in"));
        assert!(css.contains("translateX(28px)"));
        assert!(css.contains(".zcast-card--entering"));
        assert!(css.contains("animation: zcast-card-slide-in 220ms cubic-bezier(0.22, 1, 0.36, 1) both"));
        assert!(css.contains("@media (prefers-reduced-motion: reduce)"));

        // Reduced motion explicitly disables the slide-in.
        let reduce_at = css.find("@media (prefers-reduced-motion: reduce)").unwrap();
        let reduced = &css[reduce_at..];
        assert!(reduced.contains(".zcast-card--entering"));
        assert!(reduced.contains("animation: none !important"));
    }

    #[test]
    fn panes_js_animates_and_reveals_inserted_cards() {
        let js = panes_js();

        assert!(js.contains("zcast-card--entering"));
        assert!(js.contains("ENTERING_CLASS"));
        assert!(js.contains("animationend"));
        assert!(js.contains("requestAnimationFrame"));
        // Layout is measured after insertion, so the reveal happens in rAF.
        let reveal_at = js.find("function revealPane").expect("revealPane helper");
        let reveal = &js[reveal_at..];
        assert!(reveal.contains("window.requestAnimationFrame"));
        assert!(reveal.contains("scrollToPane(card)"));
    }

    // Issue #4: autosave must not reset the caret while the title is focused.
    #[test]
    fn panes_js_update_card_meta_guards_focused_title() {
        let js = panes_js();

        let at = js.find("function updateCardMeta").expect("updateCardMeta");
        let body = &js[at..];
        let end = body.find("function onThemeToggle").expect("next function");
        let meta = &body[..end];

        assert!(meta.contains("document.activeElement !== node"));
        assert!(meta.contains("current.trim() !== title.trim()"));
        assert!(meta.contains("node.textContent = title"));
        assert!(meta.contains("rebuildBreadcrumbs()"));
        assert!(meta.contains("syncDocumentTitle()"));
        assert!(meta.contains("pushUrl()"));
    }

    // Issue #3: breadcrumb clicks scroll, pulse and re-mark the current crumb.
    #[test]
    fn panes_js_breadcrumb_clicks_scroll_and_pulse() {
        let js = panes_js();

        assert!(js.contains("[data-zcast-breadcrumbs] [data-zcast-breadcrumb-slug]"));
        assert!(js.contains("paneForSlug(crumbSlug)"));
        assert!(js.contains("scrollToPane(crumbPane)"));
        assert!(js.contains("pulse(crumbPane)"));
        assert!(js.contains("markCurrentCrumb(crumb)"));
        assert!(js.contains("zcast-breadcrumbs__item--current"));
        assert!(js.contains("window.location.href = crumb.href"));

        // Unsaved drafts keep an entry without a slug instead of crashing.
        assert!(js.contains("New Note"));
        assert!(js.contains("zcast-breadcrumbs__label"));
    }
}
