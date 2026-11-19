/* Shared mock data (taken from the seeded demo console) and tiny render helpers. Mock only. */
const ICONS = {
  copy: '<rect x="8" y="8" width="11" height="11" rx="2"/><path d="M5 15V6a2 2 0 0 1 2-2h8"/>',
  search: '<circle cx="11" cy="11" r="6"/><path d="m20 20-4-4"/>',
  settings: '<circle cx="12" cy="12" r="3"/><path d="M12 3v3M12 18v3M3 12h3M18 12h3M5.6 5.6l2.1 2.1M16.3 16.3l2.1 2.1M18.4 5.6l-2.1 2.1M7.7 16.3l-2.1 2.1"/>',
  tag: '<path d="M3 12V4h8l10 10-8 8z"/><circle cx="7.5" cy="8.5" r="1"/>',
  chevron: '<path d="m7 10 5 5 5-5"/>',
  right: '<path d="m10 7 5 5-5 5"/>',
  left: '<path d="m14 7-5 5 5 5"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  play: '<path d="M8 5v14l11-7z"/>',
  terminal: '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="m7 9 3 3-3 3M13 15h4"/>',
  note: '<path d="M6 3h9l4 4v14H6z"/><path d="M14 3v5h5"/>',
  clock: '<circle cx="12" cy="12" r="8"/><path d="M12 8v4l3 2"/>',
  check: '<path d="m5 12 5 5 9-10"/>',
  grid: '<rect x="4" y="4" width="7" height="7" rx="1.5"/><rect x="13" y="4" width="7" height="7" rx="1.5"/><rect x="4" y="13" width="7" height="7" rx="1.5"/><rect x="13" y="13" width="7" height="7" rx="1.5"/>',
  list: '<path d="M8 6h12M8 12h12M8 18h12M4 6h.01M4 12h.01M4 18h.01"/>',
  board: '<rect x="3" y="4" width="5" height="16" rx="1.5"/><rect x="10" y="4" width="5" height="10" rx="1.5"/><rect x="17" y="4" width="4" height="13" rx="1.5"/>',
  filter: '<path d="M4 5h16l-6 7v6l-4 2v-8z"/>',
  close: '<path d="m6 6 12 12M18 6 6 18"/>',
  wrap: '<path d="M12 3 21 12 12 21 3 12z"/>',
  dev: '<path d="M12 4 3 20h18z"/><path d="M12 10v4M12 17h.01"/>',
  more: '<path d="M5 12h.01M12 12h.01M19 12h.01"/>',
  sort: '<path d="M8 6v12m0 0-3-3m3 3 3-3M16 18V6m0 0-3 3m3-3 3 3"/>',
  dot: '<circle cx="12" cy="12" r="3" fill="currentColor"/>'
}
function icon(name, size) {
  const s = size ? ` style="width:${size}px;height:${size}px"` : ''
  return `<svg class="ic" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"${s}>${ICONS[name] || ''}</svg>`
}

/* state: todo | prog | claimed | done | filed | session */
function mk(state) {
  const r = '<circle cx="7" cy="7" r="5.2" '
  const map = {
    todo: r + 'fill="none" stroke="#8c96a6" stroke-width="1.4"/>',
    prog: r + 'fill="none" stroke="#eaa43c" stroke-width="1.4"/><path d="M7 4.2a2.8 2.8 0 0 1 0 5.6z" fill="#eaa43c"/>',
    claimed: r + 'fill="#eaa43c22" stroke="#eaa43c" stroke-width="1.4"/><path d="m4.6 7.2 1.7 1.7 3.1-3.6" fill="none" stroke="#ffd88e" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/>',
    done: r + 'fill="#3ddc97" /><path d="m4.6 7.2 1.7 1.7 3.1-3.6" fill="none" stroke="#08301f" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>',
    filed: r + 'fill="none" stroke="#8fa89d" stroke-width="1.4"/><path d="m4.6 7.2 1.7 1.7 3.1-3.6" fill="none" stroke="#8fa89d" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/>',
    draft: r + 'fill="none" stroke="#8c96a6" stroke-width="1.3" stroke-dasharray="2 2"/>',
    session: r + 'fill="none" stroke="#7c83f7" stroke-width="1.4"/><circle cx="7" cy="7" r="2" fill="#7c83f7"/>'
  }
  return `<span class="mk" aria-hidden="true"><svg viewBox="0 0 14 14">${map[state] || map.todo}</svg></span>`
}

function segs(done, claimed, total, running) {
  let out = ''
  for (let i = 0; i < total; i++) out += `<i class="${i < done ? 'd' : i < done + claimed ? 'c' : running && i === done + claimed ? 'r' : ''}"></i>`
  return `<div class="segbar" role="img" aria-label="${done} done, ${claimed} claimed of ${total}">${out}</div>`
}

const PROJECTS = [
  { id: 'mal', name: 'Malwarebytes', company: 'Acme', hue: '#a78bfa', initial: 'M', tokens: '99.3k', count: 3 },
  { id: 'lig', name: 'Lightcraft', company: 'Artcraft', hue: '#f0955a', initial: 'L', tokens: '6.84k', count: 1 },
  { id: 'cos', name: 'COSMIC', company: 'ESA', hue: '#6c8cff', initial: 'C', tokens: '1.59M', count: 8 },
  { id: 'stv', name: 'STVW', company: 'ESA', hue: '#e263b0', initial: 'S', tokens: '7.7M', count: 55 }
]

/* status: todo | prog | claimed | done. steps: [done, total]. awaiting: claimed steps waiting for the person. */
const TASKS = [
  { p: 'mal', title: 'Create the requested documents', label: 'create-the-requested-documents', status: 'prog', steps: [0, 0], notes: 0, tokens: '0', ago: '2d' },
  { p: 'mal', title: 'Review based on the interview', label: 'review-based-on-the-interview', status: 'claimed', steps: [0, 0], notes: 0, tokens: '5.54k', ago: '16h', review: true, wrapup: true },
  { p: 'mal', title: 'Solution improvements', label: 'solution-improvements', status: 'claimed', steps: [0, 0], notes: 0, tokens: '93.7k', ago: '1d', review: true },
  { p: 'lig', title: 'Delete images on mac', label: 'delete-images-on-mac', status: 'prog', steps: [0, 0], notes: 0, tokens: '6.84k', ago: '3d' },
  { p: 'cos', title: 'Prometheus configuration in AINABLER', label: 'metrics-in-ainabler', status: 'prog', steps: [0, 0], notes: 2, tokens: '33.6k', ago: '4d' },
  { p: 'cos', title: 'Refactoring applicatio', label: 'refactoring-applicatio', status: 'prog', steps: [0, 0], notes: 1, tokens: '95.1k', ago: '5d' },
  { p: 'cos', title: 'Work experience', label: 'voice-in-the-cv', status: 'prog', steps: [0, 0], notes: 1, tokens: '15.8k', ago: '6d' },
  { p: 'stv', title: 'access to pypi repository in local', label: 'access-to-pypi-repository-in-local', status: 'prog', steps: [0, 0], notes: 1, tokens: '27.8k', ago: '8d', drafts: 1 },
  { p: 'stv', title: 'check metricbridge integration situation', label: 'check-metricbridge-integration-situation', status: 'claimed', steps: [1, 24], claimed: 23, notes: 6, tokens: '731k', ago: '1d', review: true, wrapup: true, deviations: 1, selected: true },
  { p: 'stv', title: 'frontend notification', label: 'frontend-notification', status: 'prog', steps: [0, 1], notes: 6, tokens: '23.6k', ago: '3d' },
  { p: 'stv', title: 'MCP servers: spans and trace ids in logs', label: 'mcp-spans-and-trace-logs', status: 'prog', steps: [0, 0], notes: 0, tokens: '0', ago: '9d' },
  { p: 'stv', title: 'Metricbridge integration: open points', label: 'metricbridge-open-points', status: 'claimed', steps: [0, 4], claimed: 2, notes: 0, tokens: '162k', ago: '2d', review: true },
  { p: 'stv', title: 'Report work done', label: 'report-work-done', status: 'prog', steps: [0, 0], notes: 0, tokens: '6.95k', ago: '2d' },
  { p: 'stv', title: 'Testing the current span id in h5 services', label: 'testing-span-id-h5', status: 'claimed', steps: [0, 0], notes: 0, tokens: '41.2k', ago: '4d', review: true }
]

const STEPS = [
  { s: 'done', t: 'Read solution-overview.md and the merge-requests note' },
  { s: 'claimed', t: 'Integrate metricbridge in stvv-core-backend (log-preprocessing, knowledge-manager, ingestion-registry) and push', ago: '7 days ago', sel: true },
  { s: 'claimed', t: 'Verify stvv-core-backend integration after propagation', ago: '7 days ago' },
  { s: 'claimed', t: 'Add the metricbridge deploy settings to the stvv-core-backend apps', ago: '7 days ago' },
  { s: 'claimed', t: 'Verify the metricbridge deploy settings of the stvv-core-backend apps', ago: '7 days ago' },
  { s: 'claimed', t: 'Check the stvv-core-backend apps in Grafana', ago: '6 days ago' },
  { s: 'claimed', t: 'Integrate metricbridge in stvv-knowledge-loader and push', ago: '6 days ago' },
  { s: 'claimed', t: 'Verify stvv-knowledge-loader integration after propagation', ago: '6 days ago' },
  { s: 'claimed', t: 'Integrate metricbridge in the registry-mcp service (FastAPI)', ago: '5 days ago' },
  { s: 'claimed', t: 'Integrate metricbridge in the email-mcp and jira-zephyr-mcp services (FastMCP)', ago: '5 days ago' },
  { s: 'claimed', t: 'Upgrade gitlab-mcp to Spring Boot 4.1.0 and add metricbridge-lib-java 0.3.0', ago: '4 days ago' }
]

function projectById(id) { return PROJECTS.find((p) => p.id === id) }

function topbar(active, anchor) {
  const tabs = [['Console', ''], ['Review', '38'], ['Time', ''], ['Diagrams', ''], ['Security', '']]
  return `<header class="top">
    <div class="top-l"><span class="logo"></span>${tabs.map(([n, c]) => `<button class="tab ${n === active ? 'on' : ''}">${n}${c ? `<span class="cnt">${c}</span>` : ''}</button>`).join('')}</div>
    <div class="locbar"><b>/rk</b>${anchor || '<span>project:vega task:setup</span>'}<kbd>⌘K</kbd></div>
    <div class="top-r">
      <div class="status-cap"><span class="num">3% · 4h 55m</span><span class="amber"><b>38</b> to review</span></div>
      <button class="ibtn" aria-label="Tags">${icon('tag')}</button><button class="ibtn" aria-label="Settings">${icon('settings')}</button>
    </div>
  </header>`
}
function anchorBar(project, task) {
  return `<span class="chip">project:${project}</span><span class="dim">›</span><span class="chip">task:${task}</span>`
}
