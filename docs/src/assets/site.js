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
