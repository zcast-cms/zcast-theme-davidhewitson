/* ==========================================================================
 * zcast-theme-davidhewitson — sliding panes
 *
 * - Intercepts clicks on `a[data-zcast-note]` inside `.zcast-panes` and opens
 *   the target card as a new pane to the right (falls back to a normal
 *   navigation if the fetch fails).
 * - Modifier clicks (Cmd / Ctrl / Shift / Alt), middle clicks, `target=_blank`
 *   and `download` links are left alone so "open in new tab" keeps working.
 * - Re-clicking an already-open note scrolls to it and pulses the pane instead
 *   of duplicating it.
 * - Syncs the stack to `/<root>?stack=a,b,c` with `history.pushState` and
 *   restores it on `popstate`.
 * - Exposes `window.ZcastPanes` for `zcast-editor-inline` (fresh draft panes,
 *   renames, breadcrumb refreshes).
 * ========================================================================== */

(function () {
  'use strict';

  var HIGHLIGHT_CLASS = 'zcast-card--highlight';
  var MOBILE_QUERY = '(max-width: 768px)';
  var CARD_ENDPOINT = '/_zcast/cards/';
  var TOGGLE_KEY = 'zcast-color-mode';

  function panesContainer() {
    return document.querySelector('[data-zcast-panes]');
  }

  function allCards() {
    var container = panesContainer();
    if (!container) {
      return [];
    }
    var nodes = container.querySelectorAll('.zcast-card');
    return Array.prototype.slice.call(nodes);
  }

  function isMobile() {
    return !!(window.matchMedia && window.matchMedia(MOBILE_QUERY).matches);
  }

  function slugOf(card) {
    return (card && card.getAttribute('data-zcast-card-slug')) || '';
  }

  function titleOf(card) {
    var node = card && card.querySelector('[data-zcast-card-title]');
    var text = node && node.textContent ? node.textContent.trim() : '';
    return text || slugOf(card);
  }

  function paneForSlug(slug) {
    var cards = allCards();
    for (var i = 0; i < cards.length; i += 1) {
      if (slugOf(cards[i]) === slug) {
        return cards[i];
      }
    }
    return null;
  }

  function scrollToPane(pane) {
    if (!pane) {
      return;
    }

    if (isMobile()) {
      if (typeof pane.scrollIntoView === 'function') {
        pane.scrollIntoView({ behavior: 'smooth', block: 'start' });
      }
      return;
    }

    var container = panesContainer();
    if (!container) {
      return;
    }

    var left = pane.offsetLeft;
    if (typeof container.scrollTo === 'function') {
      container.scrollTo({ left: left, behavior: 'smooth' });
    } else {
      container.scrollLeft = left;
    }
  }

  function pulse(pane) {
    if (!pane) {
      return;
    }
    pane.classList.remove(HIGHLIGHT_CLASS);
    // Force a reflow so the animation can restart on rapid re-clicks.
    void pane.offsetWidth;
    pane.classList.add(HIGHLIGHT_CLASS);
    window.setTimeout(function () {
      pane.classList.remove(HIGHLIGHT_CLASS);
    }, 1400);
  }

  function truncateAfter(index) {
    var cards = allCards();
    for (var i = cards.length - 1; i > index; i -= 1) {
      if (cards[i].parentNode) {
        cards[i].parentNode.removeChild(cards[i]);
      }
    }
  }

  function stackSlugs() {
    return allCards()
      .map(slugOf)
      .filter(function (slug) {
        return !!slug;
      });
  }

  function urlForStack() {
    var stack = stackSlugs();
    if (!stack.length) {
      return window.location.pathname;
    }
    var path = '/' + encodeURIComponent(stack[0]);
    var tail = stack.slice(1);
    if (tail.length) {
      path += '?stack=' + tail.map(encodeURIComponent).join(',');
    }
    return path;
  }

  function pushUrl() {
    if (!window.history || typeof window.history.pushState !== 'function') {
      return;
    }
    var url = urlForStack();
    var current = window.location.pathname + window.location.search;
    if (url !== current) {
      window.history.pushState({ zcastPanes: true }, '', url);
    }
  }

  function siteTitle() {
    var node = document.querySelector('[data-zcast-site-title]');
    var text = node && node.textContent ? node.textContent.trim() : '';
    return text;
  }

  function syncDocumentTitle() {
    var cards = allCards();
    var site = siteTitle();
    if (!cards.length || !site) {
      return;
    }
    var title = titleOf(cards[cards.length - 1]);
    document.title = title && title !== site ? title + ' \u2014 ' + site : site;
  }

  function rebuildBreadcrumbs() {
    var nav = document.querySelector('[data-zcast-breadcrumbs]');
    if (!nav) {
      return;
    }

    var cards = allCards();
    var list = document.createElement('ol');
    list.className = 'zcast-breadcrumbs__list';

    cards.forEach(function (card, index) {
      var slug = slugOf(card);
      if (!slug) {
        return;
      }
      var isCurrent = index === cards.length - 1;

      var item = document.createElement('li');
      item.className =
        'zcast-breadcrumbs__item' + (isCurrent ? ' zcast-breadcrumbs__item--current' : '');

      var link = document.createElement('a');
      link.className = 'zcast-breadcrumbs__link';
      link.href = '/' + encodeURIComponent(slug);
      link.setAttribute('data-zcast-breadcrumb', '');
      link.setAttribute('data-zcast-breadcrumb-slug', slug);
      link.setAttribute('aria-current', isCurrent ? 'page' : 'false');
      link.textContent = titleOf(card);

      item.appendChild(link);
      list.appendChild(item);
    });

    nav.innerHTML = '';
    nav.appendChild(list);
  }

  function fetchCard(slug, done) {
    var url = CARD_ENDPOINT + encodeURIComponent(slug);
    window
      .fetch(url, {
        headers: { Accept: 'text/html' },
        credentials: 'same-origin'
      })
      .then(function (response) {
        if (!response.ok) {
          throw new Error('HTTP ' + response.status);
        }
        return response.text();
      })
      .then(function (html) {
        done(null, html);
      })
      .catch(function (error) {
        done(error || new Error('fetch failed'));
      });
  }

  function insertAfter(anchor, html) {
    if (anchor) {
      anchor.insertAdjacentHTML('afterend', html);
      return anchor.nextElementSibling;
    }
    var container = panesContainer();
    if (!container) {
      return null;
    }
    container.insertAdjacentHTML('beforeend', html);
    return container.lastElementChild;
  }

  function openSlug(slug, anchor) {
    if (!slug) {
      return;
    }

    var existing = paneForSlug(slug);
    if (existing) {
      scrollToPane(existing);
      pulse(existing);
      return;
    }

    if (!anchor || !anchor.isConnected) {
      return;
    }

    fetchCard(slug, function (error, html) {
      if (error) {
        // Graceful degradation: behave like a plain link.
        window.location.href = '/' + encodeURIComponent(slug);
        return;
      }

      if (!anchor.isConnected) {
        return;
      }

      var cards = allCards();
      var index = cards.indexOf(anchor);
      if (index < 0) {
        return;
      }

      truncateAfter(index);
      var card = insertAfter(anchor, html);
      if (!card) {
        return;
      }

      if (!slugOf(card)) {
        card.setAttribute('data-zcast-card-slug', slug);
      }

      rebuildBreadcrumbs();
      scrollToPane(card);
      syncDocumentTitle();
      pushUrl();
    });
  }

  function onDocumentClick(event) {
    if (event.defaultPrevented || event.button !== 0) {
      return;
    }
    if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) {
      return;
    }

    var target = event.target;
    if (!target || typeof target.closest !== 'function') {
      return;
    }

    // Breadcrumb navigation: scroll within the desktop stack, navigate on mobile.
    var crumb = target.closest('[data-zcast-breadcrumbs] [data-zcast-breadcrumb-slug]');
    if (crumb) {
      event.preventDefault();
      var crumbSlug = crumb.getAttribute('data-zcast-breadcrumb-slug');
      if (isMobile()) {
        window.location.href = crumb.href;
        return;
      }
      var crumbPane = paneForSlug(crumbSlug);
      if (crumbPane) {
        scrollToPane(crumbPane);
      } else {
        window.location.href = crumb.href;
      }
      return;
    }

    var link = target.closest('a[data-zcast-note]');
    if (!link) {
      return;
    }

    // Admin-only unresolved links are owned by `zcast-editor-inline`.
    if (link.hasAttribute('data-zcast-unresolved')) {
      return;
    }
    if (link.hasAttribute('download')) {
      return;
    }
    if (link.target && link.target !== '' && link.target !== '_self') {
      return;
    }

    var pane = link.closest('.zcast-card');
    if (!pane || !link.closest('[data-zcast-panes]')) {
      return;
    }

    var slug = link.getAttribute('data-zcast-note');
    if (!slug) {
      return;
    }

    try {
      var url = new URL(link.href, window.location.href);
      if (url.origin !== window.location.origin) {
        return;
      }
    } catch (error) {
      // Older engines without `URL` still get the in-page behaviour.
    }

    event.preventDefault();
    openSlug(slug, pane);
  }

  function readStackFromUrl() {
    var stack = [];
    var search = window.location.search || '';
    if (search.length > 1) {
      var params = new URLSearchParams(search);
      var raw = params.get('stack');
      if (raw) {
        stack = raw.split(',').filter(function (slug) {
          return !!slug;
        });
      }
    }
    return stack;
  }

  function rootSlugFromUrl() {
    var path = window.location.pathname || '/';
    var trimmed = path.replace(/^\/+/, '').replace(/\/+$/, '');
    try {
      trimmed = decodeURIComponent(trimmed);
    } catch (error) {
      // Leave undecoded on malformed input.
    }
    return trimmed;
  }

  function restoreFromUrl() {
    var cards = allCards();
    if (!cards.length) {
      return;
    }

    var root = rootSlugFromUrl();
    if (!root) {
      root = slugOf(cards[0]);
    }
    var desired = [root].concat(readStackFromUrl());

    if (slugOf(cards[0]) !== root) {
      // A different root note — let the server render it.
      window.location.reload();
      return;
    }

    var keep = 0;
    while (keep < cards.length && keep < desired.length && slugOf(cards[keep]) === desired[keep]) {
      keep += 1;
    }

    truncateAfter(keep - 1);

    var anchor = keep > 0 ? allCards()[keep - 1] : null;
    var remaining = desired.slice(keep);

    function loadNext() {
      if (!remaining.length) {
        var current = allCards();
        if (current.length) {
          scrollToPane(current[current.length - 1]);
        }
        rebuildBreadcrumbs();
        syncDocumentTitle();
        return;
      }

      if (!anchor || !anchor.isConnected) {
        return;
      }

      var slug = remaining.shift();
      fetchCard(slug, function (error, html) {
        if (error) {
          window.location.href = '/' + encodeURIComponent(slug);
          return;
        }

        var index = allCards().indexOf(anchor);
        if (index < 0) {
          return;
        }

        truncateAfter(index);
        anchor = insertAfter(anchor, html);

        if (anchor && !slugOf(anchor)) {
          anchor.setAttribute('data-zcast-card-slug', slug);
        }

        loadNext();
      });
    }

    loadNext();
  }

  function appendCardHtml(afterPaneEl, html, slug, title) {
    var container = panesContainer();
    if (!container || !html) {
      return null;
    }

    var anchor = afterPaneEl;
    var anchorIndex = anchor ? allCards().indexOf(anchor) : -1;
    if (anchorIndex < 0) {
      anchor = allCards()[allCards().length - 1] || null;
      anchorIndex = anchor ? allCards().indexOf(anchor) : -1;
    }
    if (anchorIndex >= 0) {
      truncateAfter(anchorIndex);
    }

    var card = insertAfter(anchor, html);
    if (!card) {
      return null;
    }

    if (slug || title) {
      updateCardMeta(card, slug, title);
    }

    rebuildBreadcrumbs();
    scrollToPane(card);
    syncDocumentTitle();
    pushUrl();
    return card;
  }

  function updateCardMeta(cardEl, slug, title) {
    if (!cardEl) {
      return;
    }

    if (slug) {
      cardEl.setAttribute('data-zcast-card-slug', slug);
    }

    if (typeof title === 'string' && title.length) {
      var node = cardEl.querySelector('[data-zcast-card-title]');
      if (node) {
        node.textContent = title;
      }
    }

    rebuildBreadcrumbs();
    syncDocumentTitle();
    pushUrl();
  }

  function onThemeToggle(event) {
    var target = event.target;
    if (!target || typeof target.closest !== 'function') {
      return;
    }

    var button = target.closest('[data-zcast-theme-toggle]');
    if (!button) {
      return;
    }

    event.preventDefault();

    var root = document.documentElement;
    var next = root.getAttribute('data-theme') === 'light' ? 'dark' : 'light';
    root.setAttribute('data-theme', next);
    root.setAttribute('data-color-mode', 'auto');
    button.setAttribute('aria-pressed', next === 'dark' ? 'true' : 'false');

    try {
      window.localStorage.setItem(TOGGLE_KEY, next);
    } catch (error) {
      // Private mode / disabled storage — the visual change still applies.
    }

    try {
      document.dispatchEvent(
        new CustomEvent('zcast:themechange', { detail: { theme: next } })
      );
    } catch (error) {
      // `CustomEvent` unavailable — skip the notification.
    }
  }

  function init() {
    document.addEventListener('click', onDocumentClick);
    document.addEventListener('click', onThemeToggle);
    window.addEventListener('popstate', restoreFromUrl);

    var container = panesContainer();
    var cards = allCards();

    // Land on the deepest pane when the server rendered a stack.
    if (!isMobile() && container && cards.length > 1) {
      container.scrollLeft = cards[cards.length - 1].offsetLeft;
    }
  }

  window.ZcastPanes = {
    appendCardHtml: appendCardHtml,
    updateCardMeta: updateCardMeta,
    rebuildBreadcrumbs: rebuildBreadcrumbs,
    scrollToPane: scrollToPane
  };

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
