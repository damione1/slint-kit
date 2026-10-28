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
