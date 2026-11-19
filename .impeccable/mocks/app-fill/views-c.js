/* The project page, and the router. */
const HEADS = [['l2', 'overview', 'Overview'], ['l2', 'architecture', 'Architecture'], ['l3', 'request-path', 'Request path'], ['l3', 'background-work', 'Background work'], ['l2', 'data-model', 'Data model'], ['l2', 'services', 'Services'], ['l2', 'conventions', 'Conventions'], ['l2', 'release', 'Release and rollback']]
const REF = `<h1>Platform reference</h1><p class="lead">How Vega is put together, written once for anyone, or any session, changing it. Read the part the work touches.</p>
<h2 id="overview">Overview</h2><p>Vega is the payments backend behind the checkout. It takes an order from the storefront, prices it, charges it through the acquirer and writes the result to the ledger.</p><ul><li>One deployable per service, one repository, one release train.</li><li>Services talk over HTTP inside the cluster and through the event bus across it.</li><li>The ledger is the only source of truth for money; every other table is a projection.</li></ul>
<h2 id="architecture">Architecture</h2><p>Requests enter through the gateway, which authenticates the caller and stamps a correlation id. The id travels through every call and every event.</p><h3 id="request-path">Request path</h3><pre>storefront → gateway → billing → acquirer\n                         ↓\n                      ledger → event bus → projections</pre><h3 id="background-work">Background work</h3><p>Reconciliation, invoice rendering and webhook delivery run as workers on the same image as their service, started with <code>--worker</code>.</p>
<h2 id="data-model">Data model</h2><table><thead><tr><th>Table</th><th>Owner</th><th>Holds</th></tr></thead><tbody><tr><td><code>entry</code></td><td>ledger</td><td>One signed amount per account, append only</td></tr><tr><td><code>charge</code></td><td>billing</td><td>An attempt to take money, with its acquirer reference</td></tr><tr><td><code>invoice</code></td><td>billing</td><td>What the customer is shown, derived from charges</td></tr></tbody></table>
<h2 id="services">Services</h2><p><b>billing</b> owns prices, charges and invoices; it never writes an entry itself. <b>ledger</b> owns entries and balances; an entry is never updated or deleted.</p>
<h2 id="conventions">Conventions</h2><ul><li>Money is an integer count of minor units with a currency code.</li><li>A migration ships with the code that needs it and is safe with the old code live.</li></ul>
<h2 id="release">Release and rollback</h2><p>A tag on <code>main</code> builds every image and rolls them out one service at a time, ledger last. Rolling back is redeploying the previous tag.</p>`
const SHORT = (t, lead) => `<h1>${t}</h1><p class="lead">${lead}</p><h2 id="s1">Summary</h2><p>Short page: the pane still fills; the prose stays at a readable measure in the middle of it.</p>`
const COMP_BODY = `<h2 id="c-purpose">Purpose</h2><p>Turns an order into a charge. It is the only service that talks to the acquirer, so everything about how money is asked for lives here.</p><h2 id="c-resp">Responsibilities</h2><ul><li>Price an order from the catalogue and the customer's currency.</li><li>Create a charge, call the acquirer and keep its reference.</li><li>Ask the ledger to record the result; never write an entry itself.</li></ul><h2 id="c-if">Interfaces</h2><table><thead><tr><th>Endpoint</th><th>Caller</th><th>Does</th></tr></thead><tbody><tr><td><code>POST /charges</code></td><td>gateway</td><td>Creates and takes a charge; idempotent on the request key</td></tr><tr><td><code>POST /charges/:id/reverse</code></td><td>support tools</td><td>Asks the ledger for a reversing entry, with a reason</td></tr></tbody></table><h2 id="c-gotcha">Gotchas</h2><p>A late acquirer answer leaves the charge <code>pending</code>; the reconciler settles it. Do not retry the call from here.</p>`
const DEP_STEPS = {
  push: { title: 'Push commit', conn: 'gitlab · vega', x: 30, y: 40, text: 'Push the task branch and open the merge request with the task code in its title.', budget: 1, routes: [['pushed', 'build']] },
  build: { title: 'Pipeline build', conn: 'gitlab · vega', x: 310, y: 40, text: 'Wait for the pipeline of the branch. Read the failing job log if it is red.', budget: 3, routes: [['green', 'deploy'], ['red', 'fix']] },
  deploy: { title: 'Deploy', conn: 'kubernetes · prod', x: 590, y: 40, text: 'Apply the chart for the built tag and wait for the rollout.', budget: 2, routes: [['ready', 'check']] },
  check: { title: 'Check', conn: 'grafana · dash', x: 870, y: 40, text: 'Open the Vega dashboard. Confirm error rate and charge latency are inside their usual band for ten minutes.', budget: 2, routes: [['healthy', 'done'], ['failing', 'fix']] },
  fix: { title: 'Fix', conn: 'gitlab · vega', x: 310, y: 220, text: 'Change the code, commit and push again. Name what failed in the commit message.', budget: 3, routes: [['fixed', 'build']] },
  done: { title: 'Released', conn: '', x: 960, y: 220, text: 'The flow ends here.', budget: 0, routes: [] }
}
const EDGES = [['M230 72H310', 270, 62, 'pushed', ''], ['M510 72H590', 550, 62, 'green', ''], ['M790 72H870', 830, 62, 'ready', ''], ['M360 104V220', 370, 168, 'red', 'bad'], ['M460 220V104', 470, 168, 'fixed', 'loop'], ['M900 104V252H510', 700, 244, 'failing', 'bad'], ['M1030 104V220', 1040, 168, 'healthy', '']]
const edgeSvg = ([d, x, y, label, cls]) => `<path class="edge ${cls}" d="${d}" marker-end="url(#ah)"/><text class="elabel" x="${x}" y="${y}" ${y < 100 || y > 240 ? 'text-anchor="middle"' : ''}>${label}</text>`

function projTasks(p) {
  const list = TASKS.filter((t) => t.project === p.id)
  const f = T.ptf || 'open'
  const shown = list.filter(RULE[f])
  return `<div class="composer">${icon('plus')}<span>Describe a change, or a task to plan</span><span class="sp"></span><span class="anchor">…/${co(p.co).name}/${p.title}</span><button class="btn secondary">Start quick task</button></div>
    <div class="bar">${seg('ptf', [['needs', 'Needs me', list.filter(RULE.needs).length], ['open', 'Open', list.filter(RULE.open).length], ['filed', 'Filed', list.filter(RULE.filed).length]], f)}${finder('Filter by title or anchor')}</div>
    <section class="panel"><div class="panel-b">${ledgerTasks(shown, { grouped: false })}</div></section>
    <div class="sumline"><span>Plan adherence</span><span style="display:flex;width:240px">${bar(p.adh, 'safe')}</span><span><b>${p.adh}%</b> accepted first time</span><span><b>0</b> sent back</span><span><b>5</b> deviations</span></div>`
}
function projBlueprint(p) {
  const parts = [['Blueprint', 'Read in every context of the project', 2100, 'var(--accent)'], ['Components', '6 summaries; a description only when a task names it', 1400, 'var(--anchor)'], ['Connections', '5 bindings, labels and targets', 300, 'var(--filed)'], ['Pages', '4 titles, one line each', 60, 'var(--session)']]
  const total = parts.reduce((s, x) => s + x[2], 0)
  return `<div class="wb two">${docPane({ title: 'Blueprint', sub: `What <span class="anchor">/rk project:${p.id}</span> hands to Claude`, html: `<h1>${p.title}</h1><p class="lead">${p.desc}</p><h2 id="b1">Stack</h2><p>Go services on Kubernetes, PostgreSQL per service, an event bus between them. One repository, one release train.</p><h2 id="b2">How to work here</h2><ul><li>Money is an integer count of minor units with a currency code.</li><li>A migration ships with the code that needs it and is safe with the old code still live.</li><li>Every change goes through a merge request linked to its task.</li></ul>`, heads: [['l2', 'b1', 'Stack'], ['l2', 'b2', 'How to work here']] })}
    ${panel('Handed to a session', `<div class="facts"><div style="display:flex;justify-content:space-between;align-items:baseline"><b class="mono" style="font-size:var(--t-heading);font-weight:500">≈ ${(total / 1000).toFixed(1)}k</b><span class="dim" style="font-size:var(--t-caption)">tokens, in every context</span></div><div style="display:flex;gap:2px;height:8px">${parts.map((x) => `<i style="flex:${x[2]};background:${x[3]};border-radius:2px"></i>`).join('')}</div>
      ${parts.map((x) => `<div style="display:grid;grid-template-columns:10px minmax(0,1fr) auto;gap:2px 10px;align-items:baseline;font-size:var(--t-small);padding-top:10px;border-top:1px solid var(--border)"><i style="width:8px;height:8px;border-radius:2px;background:${x[3]};align-self:center"></i><b style="font-weight:500">${x[0]}</b><span class="mono dim" style="font-size:var(--t-caption)">${fmt(x[2])}</span><span class="dim" style="grid-column:2/4;font-size:var(--t-caption)">${x[1]}</span></div>`).join('')}
      <p class="dim" style="margin:6px 0 0;padding-top:14px;border-top:1px solid var(--border-strong);font-size:var(--t-caption);line-height:1.55">Kept out of context: the text of every page, 71,570 characters. A session reads one with <span class="mono" style="color:var(--anchor)">rekall_read_sections</span>.</p></div>`)}</div>`
}
function projPages() {
  const id = T.ppage || 'ref', pg = PAGES.find((x) => x.id === id)
  const rows = PAGES.map((x) => `<button class="row" data-set="ppage:${x.id}" ${x.id === id ? 'aria-current="page"' : ''}><span class="name">${x.title}</span><span class="sub"><span class="mono">${x.chars >= 10000 ? Math.round(x.chars / 1000) + 'k' : fmt(x.chars)}</span><span>${x.edited}</span><span style="margin-left:auto" class="mono">${Math.max(1, Math.round(x.chars / 1000))}%</span></span>${bar(x.chars / 1000, '')}</button>`).join('')
  const docs = { ref: [REF, HEADS], dep: [SHORT('Deployment', 'How a change reaches production, and how to take it back.'), []], con: [SHORT('Contributing', 'From an issue to a merged change.'), []], glo: [SHORT('Glossary', 'Terms the code and the team use.'), []] }
  const gauge = `<div class="gauge"><span class="fig">${fmt(pg.chars)} <span class="dim">of 100,000 characters</span></span>${bar(pg.chars / 1000)}<span class="fig">≈ ${(pg.chars / 4000).toFixed(1)}k tokens when read</span><span style="flex:1"></span><span class="dim">Not in context. Costs one line, its title.</span></div>`
  return `<div class="wb"><nav class="rail" aria-label="Pages"><div class="rail-h"><span class="t">Pages<span class="n">${PAGES.length}</span></span><button class="iconbtn" title="Autoconfigure">${icon('spark')}</button><button class="iconbtn" title="Import a .md file">${icon('upload')}</button><button class="iconbtn">${icon('plus')}</button></div>${rows}<p class="rail-note">Pages stay out of Claude's context. A session adds one with <code>rekall_project_section</code>.</p></nav>
    ${docPane({ title: pg.title, sub: `Edited ${pg.edited} · Saved`, html: docs[id][0], heads: docs[id][1], gauge, actions: `<button class="btn quiet">${icon('upload')}Replace from file</button>` })}</div>`
}
function projComponents() {
  const c = COMPONENTS.find((x) => x.id === (T.pcomp || 'billing-api'))
  const folders = [...new Set(COMPONENTS.map((x) => x.folder.split('/')[0]))]
  const heads = c.id === 'billing-api' ? [['l2', 'c-purpose', 'Purpose'], ['l2', 'c-resp', 'Responsibilities'], ['l2', 'c-if', 'Interfaces'], ['l2', 'c-gotcha', 'Gotchas']] : []
  const body = c.id === 'billing-api' ? COMP_BODY : `<h2 id="c-purpose">Purpose</h2><p>${c.summary}</p>`
  return `<div class="wb"><nav class="rail" aria-label="Components"><div class="rail-h"><span class="t">Components<span class="n">${COMPONENTS.length}</span></span><button class="iconbtn" title="Scan folder">${icon('spark')}</button><button class="iconbtn">${icon('plus')}</button></div>${folders.map((f) => `<div class="grp">${f}/</div>${COMPONENTS.filter((x) => x.folder.split('/')[0] === f).map((x) => `<button class="row" data-set="pcomp:${x.id}" ${x.id === c.id ? 'aria-current="page"' : ''}><span class="name">${x.name}</span><span class="sum">${x.summary}</span></button>`).join('')}`).join('')}<p class="rail-note">Every context carries each summary. A description is read with <code>rekall_read_components</code>.</p></nav>
    ${docPane({ title: c.name, sub: `<span class="mono">${c.folder}</span> · edited ${c.edited}`, html: `<p class="lead" style="font-size:var(--t-title);color:var(--text);margin-top:0">${c.summary}</p>${body}`, heads })}</div>`
}
function projDeployment() {
  const s = DEP_STEPS[T.pgstep || 'check']
  const nodes = Object.entries(DEP_STEPS).map(([id, n]) => `<g class="node ${id === (T.pgstep || 'check') ? 'sel' : ''} ${id === 'done' ? 'end' : ''}" data-set="pgstep:${id}" tabindex="0" role="button" aria-label="${n.title}"><rect class="box" x="${n.x}" y="${n.y}" width="200" height="64" rx="8"/><text class="t" x="${n.x + 16}" y="${n.y + 27}">${n.title}</text>${n.conn ? `<text class="c" x="${n.x + 16}" y="${n.y + 50}">${n.conn}</text>` : ''}</g>`).join('')
  return `<div class="wb insp"><nav class="rail" aria-label="Graphs"><div class="rail-h"><span class="t">Graphs<span class="n">2</span></span><button class="iconbtn">${icon('plus')}</button></div><button class="row" aria-current="page"><span class="name">Release loop <span class="tag">release</span></span><span class="sub"><span>6 steps · 7 routes</span></span></button><button class="row"><span class="name">Hotfix</span><span class="sub"><span>3 steps · 3 routes</span></span></button><p class="rail-note">A graph is the route a commit takes to a running application, written once and run on any task.</p></nav>
    ${panel('Release loop', `<div class="canvas"><svg viewBox="0 0 1190 310" preserveAspectRatio="xMidYMid meet" role="group" aria-label="Release loop graph"><defs><marker id="ah" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0 0L10 5L0 10z" fill="#8c96a6"/></marker></defs>${EDGES.map(edgeSvg).join('')}${nodes}</svg></div>`, { sub: 'From a pushed commit to a checked release', actions: `<button class="btn quiet">Tidy layout</button><button class="btn secondary">${icon('plus')}Add step</button>` })}
    ${panel(s.title, `<dl class="facts"><div><dt>Instructions</dt><dd>${s.text}</dd></div>${s.conn ? `<div><dt>Reaches</dt><dd><span class="anchor">${s.conn}</span></dd></div><div><dt>Visit budget</dt><dd>${s.budget} visits before the flow stops and asks</dd></div>` : ''}${s.routes.length ? `<div><dt>Routes by outcome</dt><dd style="display:grid;gap:6px;margin-top:6px">${s.routes.map(([o, to]) => `<span class="tag" style="height:28px;justify-content:space-between;font-size:var(--t-small)"><b class="mono" style="font-weight:500">${o}</b>→ ${DEP_STEPS[to].title}</span>`).join('')}</dd></div>` : ''}</dl>`)}</div>`
}
function projConnections(p) {
  const cs = CONNECTIONS.filter((c) => c.binds.some((b) => b[0] === p.id))
  const block = (c) => { const [tag, hue, kind] = KINDS[c.kind]; return panel(`<span style="display:inline-flex;align-items:center;gap:10px"><span style="width:24px;height:24px;border-radius:6px;display:grid;place-items:center;background:${hue};color:#0d1017;font:600 var(--t-caption) var(--font-mono)">${tag}</span>${c.title}</span>`, `<div class="ledger" style="--cols:160px minmax(0,1fr) 240px 90px"><div class="lh"><span>Label</span><span>Target</span><span>Credential</span><span></span></div>${c.binds.filter((b) => b[0] === p.id).map(([, l, t]) => `<div class="lr"><span class="code">${l}</span><span class="mono t" style="font-size:var(--t-small)">${t}</span><span>${c.cred ? pill('ok', 'Stored') : pill('warn', 'No credential')}</span><span style="justify-self:end"><button class="btn unbind">Unbind</button></span></div>`).join('')}<div class="lr"><button class="btn quiet" style="margin-left:-10px">${icon('plus')}Target</button></div></div>`, { sub: c.meta, actions: `<button class="btn quiet">Open in Connections</button>` }) }
  return `<div class="bar"><span class="sp"></span><button class="btn quiet">Manage connections</button><button class="btn secondary">${icon('spark')}Find repositories</button><button class="btn primary">${icon('plus')}New connection</button></div><div class="scroll" style="display:grid;gap:16px;align-content:start">${cs.map(block).join('')}${panel('Shared with ' + co(p.co).name + ', not used here', `<div class="ledger" style="--cols:minmax(0,1fr) 220px 90px">${CONNECTIONS.filter((c) => !c.binds.some((b) => b[0] === p.id)).map((c) => `<div class="lr"><span>${c.title}</span><span>${c.cred ? pill('ok', 'Stored') : pill('warn', 'No credential')}</span><span style="justify-self:end"><button class="btn secondary">Bind</button></span></div>`).join('')}</div>`)}</div>`
}
function projSettings(p) {
  const pane = T.pset || 'general'
  const nav = [['general', 'General', 'Name, anchor, company, status, icon'], ['repo', 'Repository', 'Folder, main branch, delivery'], ['tm', 'Task manager', 'Where the tasks are written'], ['checks', 'Checks', 'What a claim has to pass']]
  let panels = ''
  if (pane === 'general') panels = panel('Identity', frow('Title', 'Shown in the navigator and the switcher.', `<div class="fld">${p.title}</div>`) + frow('Label', 'What the anchor carries.', `<div class="fld mono">${p.id}<small>/rk project:${p.id}</small></div>`) + frow('Task key', 'Opens the code of every task.', '<div class="fld mono" style="max-width:120px">VG</div>')) + panel('Placement', frow('Company', 'Changing this moves the project, and its anchor.', `<div class="fld">${co(p.co).name}<small>▾</small></div>`) + frow('Status', '', seg('pstatus', [['a', 'Active'], ['p', 'Paused'], ['d', 'Done'], ['x', 'Archived']], T.pstatus || 'a')) + frow('Icon', 'A glyph, or a picture of your own.', `<span style="display:flex;gap:8px;align-items:center">${glyph(p.hue)}<button class="btn secondary">Upload image</button></span>`)) + panel('Description', frow('Description', 'It travels into every context that loads the project.', `<div class="fld area">${p.desc}</div>`, 'top')) + panel('Delete project', `<div class="pad" style="display:flex;align-items:center;gap:16px"><p class="dim" style="margin:0;flex:1;font-size:var(--t-small);line-height:1.5">Its tasks, pages, components and bindings go with it. This is not recoverable.</p><button class="btn danger">Delete project</button></div>`)
  else if (pane === 'repo') panels = panel('Repository', frow('Folder', 'Where sessions work and checks run.', '<div class="fld mono">/Users/dev/vega<small>Browse</small></div>') + frow('Main branch', 'Where a task’s branch starts and is merged.', '<div class="fld mono" style="max-width:200px">main</div>')) + panel('Delivery', frow('Merge through a pull request', 'Push the branch and open a request when a task is claimed.', sw(true)) + frow('Commit on claim', 'Commit the task’s work when a session claims it.', sw(false)))
  else if (pane === 'tm') panels = panel('Task manager', frow('Binding', 'A plain task creates an issue there with its title alone. At most one.', `<div style="display:grid;gap:8px;width:100%">${[['northwind/vega/api', 'GitLab · api', 1], ['VG', 'Jira · issues · no credential', 0], ['None', '', 0]].map(([n, m, on]) => `<div class="tag" style="height:36px;justify-content:space-between;padding:0 12px;font-size:var(--t-small);${on ? 'border-color:var(--accent);background:var(--accent-soft);color:var(--text)' : ''}"><span>${n}</span><span class="mono dim">${m}</span></div>`).join('')}</div>`, 'top'))
  else panels = panel('Checks', `<div class="ledger" style="--cols:180px minmax(0,1fr) 200px 80px 60px"><div class="lh"><span>Name</span><span>Command</span><span>Folder</span><span style="text-align:right">Timeout</span><span></span></div>${[['Unit tests', 'make test', 'billing/api', 300], ['Lint', 'make lint', 'project folder', 120], ['Migrations apply', './ci/migrate-dry-run.sh', 'ledger', 180]].map(([n, c, f, t]) => `<div class="lr"><span>${n}</span><span class="mono t" style="font-size:var(--t-small)">${c}</span><span class="mono dim" style="font-size:var(--t-caption)">${f}</span><span class="num">${t}s</span><span class="dim" style="text-align:right">↑ ↓</span></div>`).join('')}<div class="lr"><button class="btn quiet" style="margin-left:-10px">${icon('plus')}Add a check</button></div></div>`, { sub: 'run before a step’s claim is accepted', cls: 'span' })
  return `<div class="wb"><nav class="rail" aria-label="Project settings">${nav.map(([k, l, s]) => `<button class="row" data-set="pset:${k}" ${pane === k ? 'aria-current="page"' : ''}><span class="name">${l}</span><span class="sub">${s}</span></button>`).join('')}</nav><div class="scroll"><div class="fgrid">${panels}</div></div></div>`
}
function viewProject() {
  const p = pj(T.proj || 'vega'), tab = T.ptab || 'tasks'
  const tabs = [['tasks', 'Tasks', TASKS.filter((t) => t.project === p.id && t.status !== 'filed').length], ['blueprint', 'Blueprint'], ['pages', 'Pages', PAGES.length], ['components', 'Components', COMPONENTS.length], ['diagrams', 'Diagrams', DIAGRAMS.filter((d) => d.project === p.id).length], ['deployment', 'Deployment', 2], ['connections', 'Connections', CONNECTIONS.filter((c) => c.binds.some((b) => b[0] === p.id)).length], ['settings', 'Settings']]
  const body = { tasks: () => projTasks(p), blueprint: () => projBlueprint(p), pages: projPages, components: projComponents, diagrams: () => `<div class="scroll">${viewDiagrams(DIAGRAMS.filter((d) => d.project === p.id), true)}</div>`, deployment: projDeployment, connections: () => projConnections(p), settings: () => projSettings(p) }[tab]()
  return `<div class="page"><div class="ph">${scopeHead(p)}<span class="anchor">/rk project:${p.id}</span>${pill('ok', 'Active')}<span class="dim mono" style="font-size:var(--t-caption)">○ ${p.tokens}</span><span class="sp"></span><button class="btn secondary">${icon('terminal')}Open in terminal</button><button class="btn primary">${icon('plus')}New task</button></div>${tabsRow(tabs, tab)}<div class="body">${body}<span class="guide-badge"></span></div></div>`
}

/* ---------- shell and router ---------- */
const SCREENS = [['work', 'Work', viewWork], ['projects', 'Projects', viewProjects], ['companies', 'Companies', viewCompanies], ['calendar', 'Calendar', viewCalendar], ['reports', 'Reports', viewReports], ['diagrams', 'Diagrams', () => viewDiagrams()], ['security', 'Security', viewSecurity], ['review', 'Review', viewReview], ['connections', 'Connections', viewConnections], ['claude', 'Claude', viewClaude], ['settings', 'Settings', viewSettings], ['task', 'Task page', viewTask], ['project', 'Project page', viewProject]]
const HASH = location.hash.slice(1).split(',')
const S = { screen: HASH[0] || 'work', guides: HASH[4] === 'guides', menu: null }
const TAB_KEY = { work: 'work', task: 'task', claude: 'claude', project: 'ptab', security: 'sec' }
if (HASH[1] && TAB_KEY[S.screen]) T[TAB_KEY[S.screen]] = HASH[1]
if (HASH[2]) T.ppage = HASH[2]
if (HASH[3]) W.key = HASH[3]

function menus() {
  if (!S.menu) return ''
  if (S.menu === 'width') return `<div class="pop" data-menu="width" role="menu" style="min-width:176px"><div class="ph">Page width</div>${WIDTHS.map(([k, l, px]) => `<button role="menuitemradio" aria-checked="${W.key === k}" data-width="${k}">${W.key === k ? icon('check').replace('<svg', '<svg class="ck"') : '<span class="ck"></span>'}${l}<span class="px">${px ?? 'window'}</span></button>`).join('')}</div>`
  return `<div class="pop" data-menu="screens" role="menu"><div class="ph">Screens</div>${SCREENS.filter((s) => !['task', 'project'].includes(s[0])).map(([k, l]) => `<button data-go="${k}" ${S.screen === k ? 'aria-current="page"' : ''}>${l}</button>`).join('')}</div>`
}
function render() {
  const cur = SCREENS.find((s) => s[0] === S.screen)
  document.getElementById('app').innerHTML = `<div class="mockbar"><b>Mockup</b>${SCREENS.map(([k, l]) => `<button data-go="${k}" aria-pressed="${S.screen === k}">${l}</button>`).join('')}<span class="sep"></span><button data-act="guides" aria-pressed="${S.guides}">Show fill guides</button></div>
    <header class="topbar"><div class="left"><span class="logo"></span><button class="screens" data-act="screens" aria-haspopup="menu" id="sbtn">${icon('menu')}${cur[1].replace(' page', '')}${icon('chevron')}</button></div><div class="search"><span>Search or type an anchor</span><kbd>⌘K</kbd></div><div class="capsule"><span>1 session</span><span>25% · 3h 43m</span><span class="rv">${TASKS.filter(RULE.needs).length} to review</span></div></header>${cur[2]()}${menus()}`
  document.getElementById('app').classList.toggle('guides', S.guides)
  placeMenu(); bindDoc(); badge()
}
function placeMenu() {
  const pop = document.querySelector('.pop'); if (!pop) return
  const b = document.getElementById(S.menu === 'width' ? 'wbtn' : 'sbtn'); if (!b) return
  const r = b.getBoundingClientRect()
  pop.style.top = r.bottom + 6 + 'px'; pop.style.left = (S.menu === 'width' ? Math.max(8, r.right - pop.offsetWidth) : r.left) + 'px'
}
function badge() { const body = document.querySelector('.body'), g = document.querySelector('.guide-badge'); if (body && g) g.textContent = `body fills ${Math.round(body.getBoundingClientRect().width)}px` }
function bindDoc() {
  const doc = document.getElementById('doc'); if (!doc) return
  const links = [...document.querySelectorAll('.toc a')], heads = links.map((a) => document.getElementById(a.dataset.jump))
  doc.addEventListener('scroll', () => { let at = 0; heads.forEach((h, i) => { if (h && h.offsetTop - doc.scrollTop < 80) at = i }); links.forEach((a, i) => a.classList.toggle('on', i === at)) })
}
document.addEventListener('click', (e) => {
  const t = e.target.closest('[data-go],[data-tab],[data-set],[data-act],[data-width],[data-jump],[data-pick]')
  const was = S.menu; S.menu = null
  if (!t) { if (was) render(); return }
  const d = t.dataset
  if (d.pick) T.proj = d.pick
  if (d.go) S.screen = d.go
  else if (d.tab) T[TAB_KEY[S.screen]] = d.tab
  else if (d.set) { const [k, v] = d.set.split(':'); T[k] = v }
  else if (d.width) W.key = d.width
  else if (d.act === 'width') S.menu = was === 'width' ? null : 'width'
  else if (d.act === 'screens') S.menu = was === 'screens' ? null : 'screens'
  else if (d.act === 'guides') S.guides = !S.guides
  else if (d.jump) { const doc = document.getElementById('doc'); doc.scrollTop = document.getElementById(d.jump).offsetTop - 8; S.menu = was; return }
  render()
})
window.addEventListener('resize', badge)
render()
