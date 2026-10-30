// SPDX-License-Identifier: GPL-3.0-or-later
const input = document.querySelector('#component-filter');
if (input) {
  const items = [...document.querySelectorAll('.catalogue-item')];
  const groups = [...document.querySelectorAll('.catalogue-family')];
  const status = document.querySelector('#filter-status');
  input.addEventListener('input', () => {
    const query = input.value.trim().toLocaleLowerCase(document.documentElement.lang);
    let count = 0;
    for (const item of items) { item.hidden = !item.dataset.search.includes(query); if (!item.hidden) count++; }
    for (const group of groups) group.hidden = ![...group.querySelectorAll('.catalogue-item')].some(item => !item.hidden);
    status.textContent = count ? (query ? `${count} / ${items.length}` : '') : status.dataset.empty;
  });
}

// Copy the displayed code verbatim; code remains selectable without JavaScript.
for (const code of document.querySelectorAll('pre > code')) {
  if (!navigator.clipboard?.writeText) continue;
  const french = document.documentElement.lang === 'fr';
  const label = french ? 'Copier' : 'Copy';
  const actions = document.createElement('div');
  actions.className = 'code-actions';
  const button = document.createElement('button');
  button.type = 'button';
  button.textContent = label;
  button.setAttribute('aria-label', french ? 'Copier ce code' : 'Copy this code');
  const status = document.createElement('span');
  status.setAttribute('role', 'status');
  button.addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText(code.textContent);
      status.textContent = french ? 'Copié' : 'Copied';
    } catch {
      status.textContent = french ? 'Sélectionnez le code pour le copier.' : 'Select the code to copy it.';
    }
  });
  actions.append(button, status);
  code.parentElement.prepend(actions);
}

// Browser code is fetched only after an explicit request to run a component.
// One preference follows the reader between components and translations.
const previewThemeStorageKey = 'slint-kit-preview-theme';
const isPreviewTheme = value => value === 'light' || value === 'dark';
const readPreviewTheme = () => {
  const requested = new URLSearchParams(location.search).get('theme');
  if (isPreviewTheme(requested)) return requested;
  try { const saved = localStorage.getItem(previewThemeStorageKey); if (isPreviewTheme(saved)) return saved; } catch {}
  return window.matchMedia?.('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
};
const savePreviewTheme = theme => {
  try { localStorage.setItem(previewThemeStorageKey, theme); } catch {}
};
for (const preview of document.querySelectorAll('.live-preview')) {
  const button = preview.querySelector('.run-preview');
  const status = preview.querySelector('.preview-status');
  const host = preview.querySelector('.preview-frame');
  const themeChoice = preview.querySelector('.preview-theme');
  const french = document.documentElement.lang === 'fr';
  let frame, timeout, ready = false;
  let themeRequest = 0;
  const themeLabel = theme => french ? (theme === 'dark' ? 'sombre' : 'clair') : (theme === 'dark' ? 'dark' : 'light');
  const interactiveStatus = () => {
    status.textContent = (french ? 'Interactif — composants Slint en WebAssembly' : 'Interactive — Slint components in WebAssembly') + ' · ' + themeLabel(themeChoice.value);
  };
  const applyTheme = (theme, persist = true, send = true) => {
    if (!isPreviewTheme(theme)) return;
    themeChoice.value = theme;
    host.dataset.theme = theme;
    if (persist) savePreviewTheme(theme);
    if (frame && send) frame.contentWindow.postMessage({type:'slint-kit-theme', theme, requestId:++themeRequest}, location.origin);
    if (ready) interactiveStatus();
  };
  themeChoice.disabled = false;
  applyTheme(readPreviewTheme(), false, false);
  themeChoice.addEventListener('change', () => applyTheme(themeChoice.value));
  window.addEventListener('storage', event => {
    if (event.key === previewThemeStorageKey && isPreviewTheme(event.newValue)) applyTheme(event.newValue, false);
  });
  let width = Number(preview.dataset.width), height = Number(preview.dataset.height) + 30;
  const resize = () => {
    if (!frame) return;
    const ratio = Math.min(1, host.clientWidth / width);
    frame.style.width = `${width}px`; frame.style.height = `${height}px`;
    frame.style.transform = `translateX(-50%) scale(${ratio})`;
    host.style.height = `${Math.ceil(height * ratio) + 2}px`;
  };
  const failed = () => {
    clearTimeout(timeout); frame?.remove(); frame = null; ready = false; host.hidden = true;
    preview.querySelector('figure').hidden = false; button.disabled = false;
    button.textContent = french ? 'Réessayer' : 'Retry';
    status.textContent = french ? 'Aperçu indisponible. L’exemple natif reste utilisable ci-dessous.' : 'Preview unavailable. The native example below remains available.';
  };
  const start = () => {
    clearTimeout(timeout); frame?.remove(); frame = null; ready = false;
    button.disabled = true; status.textContent = french ? 'Chargement…' : 'Loading…';
    frame = document.createElement('iframe');
    frame.title = french ? 'Aperçu interactif du composant' : 'Interactive component preview';
    frame.src = `/preview/?component=${encodeURIComponent(preview.dataset.component)}&lang=${french ? 'fr' : 'en'}&theme=${themeChoice.value}`;
    frame.setAttribute('allow', 'clipboard-write');
    host.hidden = false; host.classList.add('preview-loading'); host.append(frame); resize();
    timeout = setTimeout(failed, 60000);
  };
  button.addEventListener('click', () => {
    if (frame) {
      clearTimeout(timeout); frame.remove(); frame = null; ready = false; host.hidden = true;
      preview.querySelector('figure').hidden = false;
      button.textContent = french ? 'Lancer l’aperçu Slint' : 'Run Slint preview'; status.textContent = ''; return;
    }
    start();
  });
  window.addEventListener('message', event => {
    if (event.origin !== location.origin || !frame || event.source !== frame.contentWindow || event.data?.type !== 'slint-kit-preview') return;
    if (event.data.state === 'error') { failed(); return; }
    if (event.data.state === 'theme') {
      if (Number.isSafeInteger(event.data.requestId) && event.data.requestId !== themeRequest) return;
      applyTheme(event.data.theme, true, false); return;
    }
    if (event.data.state !== 'ready') return;
    clearTimeout(timeout); width = Number(event.data.width) || width; height = Number(event.data.height) || height;
    host.classList.remove('preview-loading'); preview.querySelector('figure').hidden = true; resize();
    ready = true;
    button.disabled = false; button.textContent = french ? 'Arrêter l’aperçu Slint' : 'Stop Slint preview';
    // Changes made while WASM was loading may predate the child listener.
    applyTheme(themeChoice.value, false);
  });
  new ResizeObserver(resize).observe(host);
}
