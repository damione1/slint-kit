// SPDX-License-Identifier: GPL-3.0-or-later
(() => {
  if (window.slintKitAnalytics) return;
  const events = new Set(['github_visit', 'starter_download', 'source_download', 'code_copy',
    'preview_start', 'preview_ready', 'preview_error', 'preview_interaction', 'preview_theme',
    'search_open', 'view_search_results', 'search_no_results', 'search_result_click', 'search_error', 'catalogue_filter',
    'workspace_ready', 'workspace_error', 'workspace_interaction', 'workspace_setting', 'web_vital']);
  const parameters = new Set(['component_name', 'page_locale', 'content_type', 'snippet_language',
    'link_placement', 'error_type', 'result_count', 'result_position', 'destination_path',
    'theme', 'setting_name', 'setting_value', 'metric_name', 'metric_rating', 'metric_id', 'metric_value', 'value']);
  const source = document.currentScript?.dataset.webVitals;
  const debug = () => {
    try { return window.sessionStorage.getItem('slint-kit-analytics-debug') === '1'; } catch { return false; }
  };
  function context() {
    const route = window.location.pathname.replace(/^\/fr(?=\/)/, '');
    const component = route.match(/^\/components\/([a-z]+(?:-[a-z]+)*)\/$/)?.[1];
    const group = component ? 'component' : route.startsWith('/guides/') ? 'guide'
      : route === '/components/' ? 'catalogue' : route === '/examples/workspace/' ? 'workspace' : 'home';
    return {page_locale:document.documentElement.lang === 'fr' ? 'fr' : 'en', content_group:group,
      ...(component ? {component_name:component} : {})};
  }
  function track(name, values = {}) {
    if (!events.has(name) || window.location.hostname !== 'slint-kit.dev' || typeof window.gtag !== 'function') return;
    const clean = context();
    for (const [key, value] of Object.entries(values)) {
      if (!parameters.has(key)) continue;
      if (typeof value === 'string' && value.length > 0 && value.length <= 100) clean[key] = value;
      else if (typeof value === 'number' && Number.isFinite(value) && (value >= 0 || (name === 'web_vital' && key === 'value'))) clean[key] = value;
    }
    if (debug()) clean.debug_mode = true;
    window.gtag('event', name, clean);
  }
  window.slintKitAnalytics = {track};

  function placement(element) {
    if (element.closest('header')) return 'header';
    if (element.closest('footer')) return 'footer';
    if (element.closest('aside')) return 'sidebar';
    return 'content';
  }
  function linkClick(event) {
    if (event.type === 'auxclick' && event.button !== 1) return;
    if (event.type === 'click' && event.button > 0) return;
    const link = event.target.closest?.('a[href]');
    if (!link) return;
    let url;
    try { url = new URL(link.href, window.location.href); } catch { return; }
    if (url.hostname === 'github.com' && /^\/damione1\/slint-kit(?:\/|$)/.test(url.pathname)) {
      track('github_visit', {link_placement:placement(link), content_type:
        /^\/damione1\/slint-kit\/?$/.test(url.pathname) ? 'repository' :
        /\/issues(?:\/|$)/.test(url.pathname) ? 'issues' : 'source'});
    }
    if (url.origin !== window.location.origin) return;
    const starter = url.pathname.match(/^\/quickstart\/projects\/([a-z]+(?:-[a-z]+)*)\.zip$/);
    if (starter) track('starter_download', {component_name:starter[1], content_type:'starter', link_placement:placement(link)});
    else if (/\.(?:slint|rs|toml)$/.test(url.pathname) ||
      /^\/assets\/(?:preview-sources|workspace-preview)\/.+\/source\.tar\.gz$/.test(url.pathname)) {
      track('source_download', {content_type:context().content_group, link_placement:placement(link)});
    }
    if (link.closest('.doc-search-results')) {
      const links = [...link.closest('.doc-search-results').querySelectorAll('a')];
      track('search_result_click', {destination_path:url.pathname, result_position:links.indexOf(link) + 1});
    }
  }
  document.addEventListener('click', linkClick);
  document.addEventListener('auxclick', linkClick);

  // Never send canvas contents, form values or raw search/clipboard text.
  if (window.location.pathname === '/examples/workspace/') {
    let interacted = false;
    const interact = event => {
      if (interacted || !event.isTrusted || event.target.tagName !== 'CANVAS' || event.target.dataset.ready !== 'true') return;
      interacted = true; track('workspace_interaction');
    };
    document.addEventListener('pointerdown', interact);
    document.addEventListener('keydown', interact);
  }

  if (source && window.location.hostname === 'slint-kit.dev') {
    import(source).then(({onCLS, onINP, onLCP}) => {
      // Sum deltas when a metric updates after a tab becomes visible again.
      // Keep the generated metric ID unregistered to avoid a high-cardinality dimension.
      const report = metric => track('web_vital', {metric_name:metric.name, metric_rating:metric.rating,
        metric_id:metric.id, metric_value:metric.value, value:metric.delta});
      onCLS(report); onINP(report); onLCP(report);
    }).catch(() => { /* Optional measurement must never interrupt the documentation. */ });
  }
})();
