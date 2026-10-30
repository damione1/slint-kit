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
for (const preview of document.querySelectorAll('.live-preview')) {
  const button = preview.querySelector('.run-preview');
  const status = preview.querySelector('.preview-status');
  const host = preview.querySelector('.preview-frame');
  const theme = preview.querySelector('.preview-theme');
  const french = document.documentElement.lang === 'fr';
  let frame, timeout;
  let width = Number(preview.dataset.width), height = Number(preview.dataset.height) + 30;
  const resize = () => {
    if (!frame) return;
    const ratio = Math.min(1, host.clientWidth / width);
    frame.style.width = `${width}px`; frame.style.height = `${height}px`;
    frame.style.transform = `translateX(-50%) scale(${ratio})`;
    host.style.height = `${Math.ceil(height * ratio) + 2}px`;
  };
  const failed = () => {
    clearTimeout(timeout); frame?.remove(); frame = null; host.hidden = true;
    preview.querySelector('figure').hidden = false; button.disabled = false;
    button.textContent = french ? 'Réessayer' : 'Retry';
    status.textContent = french ? 'Aperçu indisponible. L’exemple natif reste utilisable ci-dessous.' : 'Preview unavailable. The native example below remains available.';
  };
  const start = () => {
    clearTimeout(timeout); frame?.remove(); frame = null;
    button.disabled = true; status.textContent = french ? 'Chargement…' : 'Loading…';
    frame = document.createElement('iframe');
    frame.title = french ? 'Aperçu interactif du composant' : 'Interactive component preview';
    frame.src = `/preview/?component=${encodeURIComponent(preview.dataset.component)}&theme=${theme.value}&lang=${french ? 'fr' : 'en'}`;
    frame.setAttribute('allow', 'clipboard-write');
    host.hidden = false; host.classList.add('preview-loading'); host.append(frame); resize();
    timeout = setTimeout(failed, 60000);
  };
  button.addEventListener('click', () => {
    if (frame) {
      clearTimeout(timeout); frame.remove(); frame = null; host.hidden = true;
      preview.querySelector('figure').hidden = false;
      button.textContent = french ? 'Lancer l’aperçu' : 'Run preview'; status.textContent = ''; return;
    }
    start();
  });
  theme.addEventListener('change', start);
  window.addEventListener('message', event => {
    if (event.origin !== location.origin || !frame || event.source !== frame.contentWindow || event.data?.type !== 'slint-kit-preview') return;
    if (event.data.state === 'error') { failed(); return; }
    if (event.data.state !== 'ready') return;
    clearTimeout(timeout); width = Number(event.data.width) || width; height = Number(event.data.height) || height;
    host.classList.remove('preview-loading'); preview.querySelector('figure').hidden = true; resize();
    button.disabled = false; button.textContent = french ? 'Arrêter l’aperçu' : 'Stop preview';
    status.textContent = french ? 'Interactif — thème ' + theme.selectedOptions[0].textContent : 'Interactive — ' + theme.selectedOptions[0].textContent + ' theme';
  });
  new ResizeObserver(resize).observe(host);
}
