// SPDX-License-Identifier: GPL-3.0-or-later
// The HTML illustration follows the public table example's records and its ascending/
// descending sort. It illustrates keyed selection; the linked application runs Slint/Rust.
export const heroProjects = Object.freeze([
  Object.freeze({key:101, name:'Atlas', tasks:12, archived:false}),
  Object.freeze({key:203, name:'Boreal', tasks:3, archived:false}),
  Object.freeze({key:307, name:'Cedar', tasks:24, archived:true}),
  Object.freeze({key:409, name:'Delta', tasks:8, archived:false}),
]);
export const SortColumn = Object.freeze({Name:'name', Tasks:'tasks'});
export const SortDirection = Object.freeze({Unsorted:'none', Ascending:'ascending', Descending:'descending'});

export function nextDirection(direction) {
  switch (direction) {
    case SortDirection.Unsorted: return SortDirection.Ascending;
    case SortDirection.Ascending: return SortDirection.Descending;
    case SortDirection.Descending: return SortDirection.Ascending;
  }
  throw new Error('Unknown table sort direction');
}
export function orderedProjects(column, direction) {
  if (!Object.values(SortDirection).includes(direction)) throw new Error('Unknown table sort direction');
  if (!Object.values(SortColumn).includes(column)) throw new Error('Unknown table column');
  if (direction === SortDirection.Unsorted) return [...heroProjects];
  return [...heroProjects].sort((left, right) => {
    let comparison;
    switch (column) {
      case SortColumn.Name: comparison = left.name < right.name ? -1 : left.name > right.name ? 1 : 0; break;
      case SortColumn.Tasks: comparison = left.tasks - right.tasks; break;
    }
    comparison ||= left.key - right.key;
    return direction === SortDirection.Ascending ? comparison : -comparison;
  });
}

export function installHero(root) {
  const body = root.querySelector('tbody');
  const caption = root.querySelector('[data-hero-caption]');
  const selection = root.querySelector('[data-hero-selection]');
  const playbackButton = root.querySelector('[data-hero-playback]');
  const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)');
  const Playback = Object.freeze({Playing:Symbol('playing'), Paused:Symbol('paused')});
  const rows = new Map([...body.rows].map(row => [Number(row.dataset.projectKey), row]));
  const headers = new Map([...root.querySelectorAll('[data-sort-column]')].map(button => [button.dataset.sortColumn, button]));
  let column = SortColumn.Name, direction = SortDirection.Unsorted, selected = 101;
  let playback = reducedMotion.matches ? Playback.Paused : Playback.Playing;
  let visible = false, timer;
  const interval = 2600;

  function render() {
    for (const project of orderedProjects(column, direction)) {
      const row = rows.get(project.key);
      row.dataset.selected = String(project.key === selected);
      row.querySelector('button').setAttribute('aria-pressed', String(project.key === selected));
      body.append(row); // Move existing keyed nodes; identity never follows position.
    }
    for (const [key, button] of headers) {
      const order = key === column ? direction : SortDirection.Unsorted;
      button.closest('th').setAttribute('aria-sort', order);
      button.querySelector('[data-sort-icon]').textContent = order === SortDirection.Ascending ? '↑' : order === SortDirection.Descending ? '↓' : '↕';
    }
    const project = heroProjects.find(project => project.key === selected);
    selection.textContent = `${root.dataset.selectedLabel}: ${project.name} · #${project.key}`;
    root.dataset.sortDirection = direction;
    root.dataset.selectedKey = String(selected);
    const message = direction === SortDirection.Unsorted ? root.dataset.originalCaption
      : direction === SortDirection.Ascending ? root.dataset.ascendingCaption : root.dataset.descendingCaption;
    caption.textContent = message.replace('{name}', project.name).replace('{column}', column === SortColumn.Name ? root.dataset.projectLabel : root.dataset.tasksLabel);
  }
  function sort(nextColumn) {
    direction = nextColumn === column ? nextDirection(direction) : SortDirection.Ascending;
    column = nextColumn;
    render();
  }
  function schedule() {
    clearTimeout(timer);
    const playing = playback === Playback.Playing;
    playbackButton.textContent = playing ? root.dataset.pauseLabel : root.dataset.playLabel;
    root.dataset.playback = playing ? 'playing' : 'paused';
    if (!playing || !visible || document.hidden) return;
    timer = setTimeout(() => { sort(SortColumn.Tasks); schedule(); }, interval);
  }
  function takeControl() {
    playback = Playback.Paused;
    caption.setAttribute('aria-live', 'polite');
    schedule();
  }
  for (const [key, button] of headers) {
    button.disabled = false;
    button.addEventListener('click', () => { takeControl(); sort(key); });
  }
  for (const project of heroProjects) {
    rows.get(project.key).querySelector('button').disabled = project.archived;
    rows.get(project.key).querySelector('button').addEventListener('click', () => {
      if (project.archived) return;
      takeControl(); selected = project.key; render();
    });
  }
  playbackButton.hidden = false;
  playbackButton.addEventListener('click', () => {
    playback = playback === Playback.Playing ? Playback.Paused : Playback.Playing;
    caption.setAttribute('aria-live', 'off'); schedule();
  });
  root.addEventListener('focusin', event => { if (event.target !== playbackButton) takeControl(); });
  document.addEventListener('visibilitychange', schedule);
  window.addEventListener('pagehide', () => clearTimeout(timer));
  window.addEventListener('pageshow', schedule);
  reducedMotion.addEventListener('change', () => { if (reducedMotion.matches) playback = Playback.Paused; schedule(); });
  const observer = new IntersectionObserver(entries => { visible = entries[0].isIntersecting; schedule(); }, {threshold:0.25});
  observer.observe(root);
  render(); schedule();
}

if (typeof document !== 'undefined') {
  for (const root of document.querySelectorAll('[data-hero-table]')) installHero(root);
}
