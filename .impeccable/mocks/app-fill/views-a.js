/* Helpers and the first screens. One frame for all: header, tabs, a body that is the window's width. */
const T = {}
const W = { key: 'standard', menu: false }
const WIDTHS = [['narrow', 'Narrow', 760], ['standard', 'Standard', 1040], ['wide', 'Wide', 1360], ['full', 'Full', null]]
const colMax = () => { const w = WIDTHS.find((x) => x[0] === W.key)[2]; return w === null ? 'none' : w + 'px' }
const fmt = (n) => n.toLocaleString('en-US')
const icon = (n) => `<svg aria-hidden="true"><use href="#i-${n}"/></svg>`
const glyph = (h) => `<span class="glyph" style="background:linear-gradient(135deg,hsl(${h} 45% 42%),hsl(${h} 45% 24%))"></span>`
const pj = (id) => PROJECTS.find((p) => p.id === id)
const co = (id) => COMPANIES.find((c) => c.id === id)
const pill = (k, l) => `<span class="pill ${k}">${l}</span>`
const bar = (pct, cls = '') => `<span class="fillbar ${cls}"><i style="width:${pct}%"></i></span>`
const planBar = ([d, n], st) => `<span class="steps"><span class="seg-run">${Array.from({ length: n }, (_, i) => `<i class="${i < d ? 'on' : ''} ${st === 'run' && i === d ? 'cur' : ''}"></i>`).join('')}</span><span class="n">${d}/${n}</span></span>`
const seg = (key, items, cur) => `<div class="seg" role="group">${items.map(([v, l, c]) => `<button data-set="${key}:${v}" aria-pressed="${cur === v}">${l}${c != null ? `<span class="count">${c}</span>` : ''}</button>`).join('')}</div>`
const finder = (t) => `<label class="finder">${icon('search')}${t}</label>`
const panel = (title, body, { actions = '', sub = '', foot = '', cls = '' } = {}) => `<section class="panel ${cls}"><div class="panel-h"><h2>${title}</h2>${sub ? `<span class="sub">${sub}</span>` : ''}<span class="sp"></span>${actions}</div><div class="panel-b">${body}</div>${foot ? `<div class="panel-f">${foot}</div>` : ''}</section>`
const tabsRow = (items, cur, tools = '') => `<div class="tabs" role="tablist">${items.map(([v, l, c, hot]) => `<button class="tab" role="tab" data-tab="${v}" ${cur === v ? 'aria-current="page"' : ''}>${l}${hot ? `<span class="hot">${c}</span>` : c != null ? `<span class="c">${c}</span>` : ''}</button>`).join('')}${tools ? `<div class="tools">${tools}</div>` : ''}</div>`
const sectionTabs = (cur) => `<div class="tabs">${[['projects', 'Projects', PROJECTS.length], ['companies', 'Companies', COMPANIES.length], ['connections', 'Connections', CONNECTIONS.length]].map(([v, l, c]) => `<button class="tab" data-go="${v}" ${cur === v ? 'aria-current="page"' : ''}>${l}<span class="c">${c}</span></button>`).join('')}</div>`
function frame({ title, count, actions = '', meta = '', tabs = '', body }) {
  return `<div class="page"><div class="ph"><h1>${title}${count != null ? `<span class="n">${count}</span>` : ''}</h1><span class="sp"></span>${meta ? `<div class="meta">${meta}</div>` : ''}${actions}</div>${tabs}<div class="body">${body}<span class="guide-badge"></span></div></div>`
}
function scopeHead(p) {
  return `<h1><span style="display:flex;align-items:center;gap:10px">${glyph(p.hue)}<span class="crumb">${co(p.co).name}</span><span class="slash">/</span>${p.title}</span></h1>`
}

/* a document pane: it fills, and the prose is centred in it at the width the menu chose */
function docPane({ title, sub = '', html, heads = [], gauge = '', actions = '', cls = '' }) {
  const toc = heads.length > 1 ? `<aside class="toc" aria-label="Outline"><div class="toc-h"><span>On this page</span><span class="mono">${heads.length}</span></div>${heads.map(([l, id, t], i) => `<a class="${l} ${i === 0 ? 'on' : ''}" data-jump="${id}">${t}</a>`).join('')}</aside>` : ''
  return `<section class="panel ${cls}" style="--col:${colMax()}"><div class="panel-h"><div style="min-width:0"><h2>${title}</h2>${sub ? `<p class="sub">${sub}</p>` : ''}</div><span class="sp"></span>${actions}<button class="iconbtn" data-act="width" id="wbtn" aria-label="Page width" title="Page width: ${W.key}" aria-haspopup="menu" aria-expanded="${W.menu}">${icon('width')}</button><div class="seg"><button aria-pressed="false">Write</button><button aria-pressed="true">Read</button></div></div>${gauge}
    <div class="reader ${toc ? '' : 'noToc'}"><div class="doc" id="doc"><div class="doc-col">${html}</div></div>${toc}</div></section>`
}

/* ---------- Work ---------- */
function ledgerTasks(list, { grouped = true, cols = '124px 84px minmax(0, 1fr) 220px 90px 110px 32px' } = {}) {
  const row = (t) => `<div class="lr"><span>${pill(t.status, TASK_LABEL[t.status])}</span><span class="code">${t.code}</span><span class="t">${t.title}</span>${planBar(t.plan, t.status)}<span class="num">${t.tokens}</span><span class="when">${t.when}</span><span class="go">${icon('open')}</span></div>`
  const head = `<div class="lh"><span>Status</span><span>Task</span><span>Title</span><span>Plan</span><span style="text-align:right">Tokens</span><span>Updated</span><span></span></div>`
  let rows = ''
  if (grouped) PROJECTS.forEach((p) => { const own = list.filter((t) => t.project === p.id); if (own.length) rows += `<div class="lg">${glyph(p.hue)}<span>${p.title}</span><span class="dim" style="font-weight:400">${co(p.co).name}</span><span class="count">${own.length}</span></div>${own.map(row).join('')}` })
  else rows = list.map(row).join('')
  return `<div class="ledger" style="--cols:${cols}">${head}${rows}</div>`
}
function viewWork() {
  const f = T.work || 'needs'
  const c = Object.fromEntries(Object.keys(RULE).map((k) => [k, TASKS.filter(RULE[k]).length]))
  return frame({
    title: 'Work', count: TASKS.filter(RULE.open).length, meta: `<span>All work</span>`,
    actions: `<button class="btn secondary">${icon('terminal')}Open terminal</button><button class="btn primary">${icon('plus')}New task</button>`,
    tabs: tabsRow([['needs', 'Needs me', c.needs, 1], ['running', 'Running', c.running], ['open', 'Open', c.open], ['filed', 'Filed', c.filed]], f, `${finder('Filter by title or anchor')}<button class="btn quiet">${icon('view')}View</button>`),
    body: `<div class="composer">${icon('plus')}<span>Describe a change, or a task to plan</span><span class="sp"></span><span class="anchor">…/Northwind</span><button class="btn secondary">Start quick task</button></div>
      <section class="panel"><div class="panel-b">${ledgerTasks(TASKS.filter(RULE[f]))}</div></section>`
  })
}

/* ---------- Projects section ---------- */
function viewProjects() {
  const card = (p) => `<button class="card" data-go="project" data-pick="${p.id}"><span class="ct">${glyph(p.hue)}<span style="flex:1">${p.title}</span>${p.status === 'Paused' ? pill('warn', 'Paused') : pill('ok', 'Active')}</span>
    <span class="cm"><span>${co(p.co).name}</span><span class="anchor">/rk project:${p.id}</span></span><p class="dim" style="font-size:var(--t-small);line-height:1.5;margin:0;flex:1">${p.desc}</p>
    <span class="cm"><span><b style="color:var(--text);font-weight:500">${p.review}</b> need review</span><span><b style="color:var(--text);font-weight:500">${p.open}</b> open</span><span style="margin-left:auto">${p.tokens} tokens</span></span>
    <span style="display:flex;align-items:center;gap:10px;font-size:var(--t-caption);color:var(--subtle)">Plan adherence ${bar(p.adh, p.adh >= 95 ? 'safe' : 'warn')}<b class="mono" style="color:var(--muted);font-weight:400">${p.adh}%</b></span></button>`
  return frame({ title: 'Projects', count: PROJECTS.length, actions: `<button class="btn primary">${icon('plus')}New project</button>`, tabs: sectionTabs('projects'),
    body: `<div class="bar">${seg('pfilter', [['all', 'All', PROJECTS.length], ['active', 'Active', 6], ['paused', 'Paused', 1]], T.pfilter || 'all')}${finder('Filter projects')}</div><div class="scroll"><div class="cards">${PROJECTS.map(card).join('')}</div></div>` })
}
function viewCompanies() {
  const cols = 'minmax(0, 1fr) 110px 120px 140px 120px 260px 32px'
  const rows = COMPANIES.map((c) => { const ps = PROJECTS.filter((p) => p.co === c.id); return `<div class="lr" style="min-height:52px"><span style="display:flex;align-items:center;gap:12px">${glyph(c.hue)}<span><b style="font-weight:500">${c.name}</b><br><span class="dim" style="font-size:var(--t-caption)">${ps.map((p) => p.title).join(' · ')}</span></span></span><span class="num">${c.projects}</span><span class="num">${ps.reduce((s, p) => s + p.open, 0)}</span><span class="num">${c.connections}</span><span class="num">${c.tokens}</span><span style="display:flex;align-items:center;gap:10px">${bar(c.adh, c.adh >= 95 ? 'safe' : 'warn')}<span class="mono dim" style="font-size:var(--t-caption)">${c.adh}%</span></span><span class="go">${icon('open')}</span></div>` }).join('')
  return frame({ title: 'Companies', count: COMPANIES.length, actions: `<button class="btn primary">${icon('plus')}New company</button>`, tabs: sectionTabs('companies'),
    body: `<section class="panel"><div class="panel-b"><div class="ledger" style="--cols:${cols}"><div class="lh"><span>Company</span><span style="text-align:right">Projects</span><span style="text-align:right">Open tasks</span><span style="text-align:right">Connections</span><span style="text-align:right">Tokens</span><span>Plan adherence</span><span></span></div>${rows}</div></div></section>` })
}

/* ---------- Calendar ---------- */
function viewCalendar() {
  const GOAL = 7
  const days = []
  for (let d = 26; d <= 31; d++) days.push({ d, out: true })
  for (let d = 1; d <= 30; d++) days.push({ d, out: false })
  for (let d = 1; days.length < 42; d++) days.push({ d, out: true })
  const hrs = (d) => MONTH_HOURS[d] ?? 0
  const label = (h) => (h ? `${Math.floor(h)}h ${String(Math.round((h % 1) * 60)).padStart(2, '0')}m` : '')
  const cells = days.map((x) => { const h = x.out ? 0 : hrs(x.d); return `<div class="day ${x.out ? 'out' : ''} ${!x.out && x.d === 17 ? 'today' : ''}"><span class="d"><b>${x.d}</b><span>${label(h)}</span></span>${h ? bar(Math.min(100, h / GOAL * 100), h > GOAL ? 'accent' : '') : ''}</div>` }).join('')
  const total = Object.values(MONTH_HOURS).reduce((s, h) => s + h, 0)
  const mix = [['Vega', 41, 'anchor'], ['Atlas', 22, ''], ['Harbor', 14, ''], ['Relay', 9, 'accent']]
  return frame({ title: 'Calendar', meta: '<span>November 2026</span>', actions: `<button class="iconbtn" aria-label="Previous month">${icon('left')}</button><button class="btn secondary">Today</button><button class="iconbtn" aria-label="Next month">${icon('right')}</button>`,
    body: `<div class="wb two"><section class="panel"><div class="cal">${['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'].map((d) => `<div class="dow">${d}</div>`).join('')}${cells}</div></section>
      ${panel('November', `<dl class="kv"><dt>Worked</dt><dd class="mono">${Math.floor(total)}h ${String(Math.round((total % 1) * 60)).padStart(2, '0')}m</dd><dt>Workday goal</dt><dd class="mono">${GOAL}h</dd><dt>Days over</dt><dd class="mono">4</dd><dt>Sessions</dt><dd class="mono">63</dd></dl><div class="pad" style="border-top:1px solid var(--border);display:grid;gap:12px"><div class="dim" style="font-size:var(--t-caption)">Share of the month</div>${mix.map(([n, v, c]) => `<div style="display:grid;grid-template-columns:64px minmax(0,1fr) 36px;gap:10px;align-items:center;font-size:var(--t-small)"><span>${n}</span>${bar(v * 2, c)}<span class="mono dim" style="text-align:right">${v}%</span></div>`).join('')}</div>`)}</div>` })
}

/* ---------- Reports ---------- */
function viewReports() {
  const data = { nw: [['Vega', 38.5, '1.59M', 21, 9], ['Atlas', 21.0, '1.31M', 13, 6], ['Harbor', 14.2, '940k', 9, 4]], hl: [['Lumen', 12.4, '810k', 8, 3], ['Quill', 4.6, '400k', 4, 2]], ob: [['Relay', 9.1, '520k', 6, 3], ['Beacon', 1.2, '120k', 1, 0]] }
  const max = 38.5
  const cols = 'minmax(0, 1fr) minmax(220px, 1.4fr) 110px 100px 100px'
  const rows = COMPANIES.map((c) => { const ps = data[c.id]; return `<div class="lg">${glyph(c.hue)}<span>${c.name}</span><span class="count">${ps.reduce((s, p) => s + p[1], 0).toFixed(1)}h</span></div>${ps.map(([n, h, t, s, d]) => `<div class="lr"><span>${n}</span><span style="display:flex;align-items:center;gap:12px">${bar(h / max * 100, 'anchor')}<span class="mono dim" style="width:52px;text-align:right;font-size:var(--t-caption)">${h.toFixed(1)}h</span></span><span class="num">${t}</span><span class="num">${s}</span><span class="num">${d}</span></div>`).join('')}` }).join('')
  return frame({ title: 'Reports', meta: '<span>1–17 November 2026</span>', actions: `<button class="btn secondary">Export CSV</button>`,
    body: `<div class="bar">${seg('range', [['week', 'This week'], ['month', 'This month'], ['quarter', 'Quarter']], T.range || 'month')}<div class="fld" style="width:200px;height:28px">All companies<small>▾</small></div></div>
      <div class="sumline"><span>Time <b>101.0h</b></span><span>Tokens <b>5.69M</b></span><span>Sessions <b>62</b></span><span>Tasks accepted <b>27</b></span><span>Sent back <b>3</b></span></div>
      <section class="panel"><div class="panel-b"><div class="ledger" style="--cols:${cols}"><div class="lh"><span>Project</span><span>Time</span><span style="text-align:right">Tokens</span><span style="text-align:right">Sessions</span><span style="text-align:right">Accepted</span></div>${rows}</div></div></section>` })
}

/* ---------- Diagrams ---------- */
function thumb(shape) {
  const s = 'stroke="#5e6a82" stroke-width="1.4" fill="none"'
  if (shape === 'seq') return `<svg viewBox="0 0 240 120" aria-hidden="true">${[40, 120, 200].map((x) => `<rect x="${x - 26}" y="8" width="52" height="18" rx="3" fill="#1d2330" stroke="#3b4558"/><path d="M${x} 26V112" ${s} stroke-dasharray="3 3"/>`).join('')}<path d="M40 48H116" ${s}/><path d="M116 48l-6-3v6z" fill="#8c96a6"/><path d="M124 68H196" ${s}/><path d="M196 68l-6-3v6z" fill="#8c96a6"/><path d="M196 88H44" stroke="#55c9ea" stroke-width="1.4" stroke-dasharray="4 3" fill="none"/></svg>`
  if (shape === 'er') return `<svg viewBox="0 0 240 120" aria-hidden="true">${[[14, 18], [96, 50], [168, 12]].map(([x, y]) => `<rect x="${x}" y="${y}" width="58" height="46" rx="3" fill="#1d2330" stroke="#3b4558"/><path d="M${x} ${y + 14}H${x + 58}" stroke="#3b4558"/><path d="M${x + 8} ${y + 24}h34M${x + 8} ${y + 34}h26" stroke="#5e6a82" stroke-width="2"/>`).join('')}<path d="M72 40H96M154 70H168V40" ${s}/></svg>`
  return `<svg viewBox="0 0 240 120" aria-hidden="true">${[[12, 46], [88, 14], [88, 78], [164, 46]].map(([x, y]) => `<rect x="${x}" y="${y}" width="64" height="26" rx="4" fill="#1d2330" stroke="#3b4558"/>`).join('')}<path d="M76 54L88 28M76 64L88 90M152 28L164 52M152 90L164 68" ${s}/></svg>`
}
const diagramCard = (d) => `<button class="card"><span class="thumb">${thumb(d.shape)}</span><span class="ct" style="font-size:var(--t-body)">${d.title}</span><span class="cm"><span class="tag">${d.type}</span>${d.scope === 'project' ? '<span>Project</span>' : `<span class="anchor">${d.scope}</span>`}<span style="margin-left:auto">${d.when}</span></span></button>`
function viewDiagrams(list = DIAGRAMS, scoped = false) {
  const out = scoped ? `<div class="cards">${list.map(diagramCard).join('')}</div>` : `<div class="cards">${PROJECTS.map((p) => { const own = list.filter((d) => d.project === p.id); return own.length ? `<div class="cgrp">${glyph(p.hue)}${p.title}<span class="count">${own.length}</span></div>${own.map(diagramCard).join('')}` : '' }).join('')}</div>`
  if (scoped) return out
  return frame({ title: 'Diagrams', count: DIAGRAMS.length, actions: `<button class="btn primary">${icon('spark')}Generate one</button>`, body: `<div class="bar">${seg('dfilter', [['all', 'All', DIAGRAMS.length], ['project', 'Project'], ['task', 'Tasks']], T.dfilter || 'all')}${finder('Filter diagrams')}</div><div class="scroll">${out}</div>` })
}

/* ---------- Security ---------- */
function viewSecurity() {
  const f = T.sec || 'reach'
  const list = ADVISORIES.filter((a) => (f === 'reach' ? a[5] === 'Reaches' : f === 'open' ? a[5] !== 'Not reachable' : true))
  const sev = { critical: 'bad', high: 'warn', medium: 'warn', low: '' }
  const verdict = { Reaches: 'bad', 'Not reachable': 'ok', Unassessed: '' }
  const cols = '100px 140px minmax(0, 1fr) 220px 140px minmax(0, 1.2fr)'
  const rows = list.map(([s, id, pkg, v, ps, vd, why]) => `<div class="lr"><span>${pill(sev[s], s)}</span><span class="code">${id}</span><span><b style="font-weight:500">${pkg}</b> <span class="mono dim" style="font-size:var(--t-caption)">${v}</span></span><span style="display:flex;gap:6px;flex-wrap:wrap">${ps.map((p) => `<span class="tag">${p}</span>`).join('')}</span><span>${pill(verdict[vd], vd)}</span><span class="dim t" style="font-size:var(--t-small)">${why || 'Not assessed yet'}</span></div>`).join('')
  return frame({ title: 'Security', count: ADVISORIES.length, actions: `<button class="btn secondary">Refresh inventory</button>`,
    body: `<div class="bar">${seg('sec', [['reach', 'Reaches a project', 2], ['open', 'Needs a verdict', 6], ['all', 'All', 8]], f)}${finder('Filter by package or advisory')}</div>
      <div class="sumline"><span>Critical <b>2</b></span><span>High <b>3</b></span><span>Medium <b>2</b></span><span>Low <b>1</b></span><span>Projects with an inventory <b>6 of 7</b></span></div>
      <section class="panel"><div class="panel-b"><div class="ledger" style="--cols:${cols}"><div class="lh"><span>Severity</span><span>Advisory</span><span>Package</span><span>Reaches</span><span>Verdict</span><span>Why</span></div>${rows}</div></div></section>` })
}
