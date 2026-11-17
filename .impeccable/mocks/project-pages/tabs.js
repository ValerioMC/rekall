/* The project tabs other than Pages. Demonstration data, labelled as such by the mockbar. */
const T = { tf: 'open', comp: 'billing-api', graph: 'release', gstep: 'check', set: 'general', dfilter: 'all', cfilter: 'all' }

const TASKS = [
  ['VG-31', 'Reverse a settled charge from the ledger', 'review', [4, 4], 'an hour ago'],
  ['VG-30', 'Webhook retries back off per endpoint', 'review', [5, 5], '3 hours ago'],
  ['VG-28', 'Invoice rendering moves to its own worker', 'review', [6, 6], 'yesterday'],
  ['VG-33', 'Acquirer timeouts leave a charge pending', 'run', [2, 5], 'now'],
  ['VG-32', 'Reconciler settles pending charges every minute', 'run', [1, 4], '12 minutes ago'],
  ['VG-34', 'Add a currency column to the invoice table', 'todo', [0, 3], 'today'],
  ['VG-29', 'Document the event bus topics', 'todo', [0, 2], 'yesterday'],
  ['VG-27', 'Gateway rejects an expired correlation id', 'todo', [0, 4], '2 days ago'],
  ['VG-26', 'Split billing config from secrets', 'todo', [0, 3], '3 days ago'],
  ['VG-25', 'Pin the acquirer SDK version', 'done', [2, 2], 'last week'],
  ['VG-24', 'Ledger balance endpoint pages its result', 'done', [5, 5], 'last week'],
  ['VG-23', 'Rename the pending state to authorised', 'filed', [3, 3], '2 weeks ago'],
  ['VG-22', 'Migrate the staging cluster to the new ingress', 'filed', [6, 6], '3 weeks ago'],
  ['VG-21', 'Drop the legacy invoice export', 'filed', [2, 2], 'a month ago']
].map(([code, title, status, plan, when]) => ({ code, title, status, plan, when }))
const TASK_RULE = { needs: (t) => t.status === 'review', open: (t) => t.status !== 'filed', filed: (t) => t.status === 'filed' }
const TASK_LABEL = { todo: 'To do', run: 'Running', review: 'In review', done: 'Done', filed: 'Filed' }

const COMPONENTS = [
  { id: 'billing-api', folder: 'billing/api', name: 'billing-api', summary: 'Prices an order, takes the charge through the acquirer and asks the ledger to record it. Owns prices, charges and invoices.', edited: '2 hours ago' },
  { id: 'billing-worker', folder: 'billing/worker', name: 'billing-worker', summary: 'Renders invoices and delivers webhooks from the same image as billing-api, started with --worker.', edited: 'yesterday' },
  { id: 'ledger', folder: 'ledger', name: 'ledger', summary: 'The only source of truth for money. Entries are appended, never updated; a mistake is a second, reversing entry.', edited: '3 days ago' },
  { id: 'gateway', folder: 'gateway', name: 'gateway', summary: 'Authenticates the caller and stamps the correlation id every call and event carries.', edited: 'last week' },
  { id: 'helm-charts', folder: 'deploy/helm', name: 'helm-charts', summary: 'One chart per service and the values for staging and production.', edited: 'last week' },
  { id: 'pipeline', folder: 'ci', name: 'pipeline', summary: 'Builds each image on a tag, applies migrations and rolls the services out one at a time, ledger last.', edited: '2 weeks ago' }
]
const COMP_HEADS = [['l2', 'c-purpose', 'Purpose'], ['l2', 'c-resp', 'Responsibilities'], ['l2', 'c-if', 'Interfaces'], ['l2', 'c-dep', 'Depends on and used by'], ['l2', 'c-run', 'Run and develop'], ['l2', 'c-gotcha', 'Gotchas']]
const COMP_BODY = `
<h2 id="c-purpose">Purpose</h2>
<p>Turns an order into a charge. It is the only service that talks to the acquirer, so everything about how money is asked for lives here.</p>
<h2 id="c-resp">Responsibilities</h2>
<ul><li>Price an order from the catalogue and the customer's currency.</li><li>Create a charge, call the acquirer and keep its reference.</li><li>Ask the ledger to record the result; never write an entry itself.</li><li>Derive the invoice the customer is shown from the charge.</li></ul>
<h2 id="c-if">Interfaces</h2>
<table><thead><tr><th>Endpoint</th><th>Caller</th><th>Does</th></tr></thead><tbody>
<tr><td><code>POST /charges</code></td><td>gateway</td><td>Creates and takes a charge; idempotent on the request key</td></tr>
<tr><td><code>GET /charges/:id</code></td><td>gateway</td><td>Reads a charge with its acquirer reference</td></tr>
<tr><td><code>POST /charges/:id/reverse</code></td><td>support tools</td><td>Asks the ledger for a reversing entry, with a reason</td></tr></tbody></table>
<h2 id="c-dep">Depends on and used by</h2>
<p>Depends on <code>ledger</code> and the acquirer. Used by <code>gateway</code> and <code>billing-worker</code>.</p>
<h2 id="c-run">Run and develop</h2>
<pre>make run SERVICE=billing-api
make test SERVICE=billing-api</pre>
<h2 id="c-gotcha">Gotchas</h2>
<p>A late acquirer answer leaves the charge <code>pending</code>; the reconciler settles it. Do not retry the call from here.</p>`

const DIAGRAMS = [
  ['Charge request path', 'Sequence', 'project', 'today', 'seq'],
  ['Service dependencies', 'Flowchart', 'project', 'yesterday', 'flow'],
  ['Ledger tables', 'ER', 'project', '3 days ago', 'er'],
  ['Reversal flow', 'Sequence', 'task:VG-31', 'an hour ago', 'seq'],
  ['Reconciler states', 'State', 'task:VG-32', '12 minutes ago', 'flow'],
  ['Release pipeline', 'Flowchart', 'task:VG-22', '3 weeks ago', 'flow']
].map(([title, type, scope, when, shape]) => ({ title, type, scope, when, shape }))

function thumb(shape) {
  const s = 'stroke="#5e6a82" stroke-width="1.4" fill="none"'
  if (shape === 'seq') return `<svg viewBox="0 0 240 120" aria-hidden="true">${[40, 120, 200].map((x) => `<rect x="${x - 26}" y="8" width="52" height="18" rx="3" fill="#1d2330" stroke="#3b4558"/><path d="M${x} 26V112" ${s} stroke-dasharray="3 3"/>`).join('')}<path d="M40 48H116" ${s}/><path d="M116 48l-6-3v6z" fill="#8c96a6"/><path d="M124 68H196" ${s}/><path d="M196 68l-6-3v6z" fill="#8c96a6"/><path d="M196 88H44" stroke="#55c9ea" stroke-width="1.4" stroke-dasharray="4 3" fill="none"/></svg>`
  if (shape === 'er') return `<svg viewBox="0 0 240 120" aria-hidden="true">${[[14, 18], [96, 50], [168, 12]].map(([x, y]) => `<rect x="${x}" y="${y}" width="58" height="46" rx="3" fill="#1d2330" stroke="#3b4558"/><path d="M${x} ${y + 14}H${x + 58}" stroke="#3b4558"/><path d="M${x + 8} ${y + 24}h34M${x + 8} ${y + 34}h26" stroke="#5e6a82" stroke-width="2"/>`).join('')}<path d="M72 40H96M154 70H168V40" ${s}/></svg>`
  return `<svg viewBox="0 0 240 120" aria-hidden="true">${[[12, 46], [88, 14], [88, 78], [164, 46]].map(([x, y]) => `<rect x="${x}" y="${y}" width="64" height="26" rx="4" fill="#1d2330" stroke="#3b4558"/>`).join('')}<path d="M76 54L88 28M76 64L88 90M152 28L164 52M152 90L164 68" ${s}/></svg>`
}

const STEPS = {
  push: { title: 'Push commit', conn: 'gitlab · vega/api', x: 30, y: 40, text: 'Push the task branch and open the merge request with the task code in its title.', budget: 1, routes: [['pushed', 'build']] },
  build: { title: 'Pipeline build', conn: 'gitlab · vega/api', x: 310, y: 40, text: 'Wait for the pipeline of the branch. Read the failing job log if it is red.', budget: 3, routes: [['green', 'deploy'], ['red', 'fix']] },
  deploy: { title: 'Deploy', conn: 'kubernetes · vega-prod', x: 590, y: 40, text: 'Apply the chart for the built tag to the cluster and wait for the rollout.', budget: 2, routes: [['ready', 'check']] },
  check: { title: 'Check', conn: 'grafana · vega', x: 870, y: 40, text: 'Open the Vega dashboard. Confirm error rate and charge latency are inside their usual band for ten minutes.', budget: 2, routes: [['healthy', 'done'], ['failing', 'fix']] },
  fix: { title: 'Fix', conn: 'gitlab · vega/api', x: 310, y: 220, text: 'Change the code, commit and push again. Name what failed in the commit message.', budget: 3, routes: [['fixed', 'build']] },
  done: { title: 'Released', conn: '', x: 960, y: 220, text: 'The flow ends here.', budget: 0, routes: [] }
}

const KINDS = { gitlab: ['GL', '#f0a05a'], kubernetes: ['K8', '#7aa2f7'], jira: ['JR', '#6cb3f0'], grafana: ['GF', '#f08a4b'] }
const CONNECTIONS = [
  { kind: 'gitlab', title: 'gitlab.fintech.co', meta: 'root path fintech/', cred: true, binds: [['api', 'fintech/vega/api', ''], ['web', 'fintech/vega/web', ''], ['infra', 'platform/vega-infra', 'Expected to start with fintech/']] },
  { kind: 'kubernetes', title: 'vega-prod', meta: 'context vega-prod · https://k8s.fintech.co', cred: true, binds: [['prod', 'vega', ''], ['stg', 'vega-stg', '']] },
  { kind: 'jira', title: 'fintech.atlassian.net', meta: 'Jira Cloud', cred: false, binds: [['issues', 'VG', '']] },
  { kind: 'grafana', title: 'grafana.fintech.co', meta: 'Grafana', cred: true, binds: [['dash', 'vega-overview', '']] }
]
const SHARED = [['github', 'github.com', 'owner vmcforge', true], ['gitlab', 'gitlab.partner.io', 'root path partner/', false]]
KINDS.github = ['GH', '#b4b9ff']

const CHECKS = [['Unit tests', 'make test', 'billing/api', 300], ['Lint', 'make lint', '', 120], ['Migrations apply', './ci/migrate-dry-run.sh', 'ledger', 180]]

const dots = (n) => '<span class="count">' + n + '</span>'
const row = (a, b) => `<div class="srow"><div><div class="lbl">${a[0]}</div><p class="hlp">${a[1]}</p></div><div>${b}</div></div>`

function taskCounts() { return { needs: TASKS.filter(TASK_RULE.needs).length, open: TASKS.filter(TASK_RULE.open).length, filed: TASKS.filter(TASK_RULE.filed).length } }

function viewTasks() {
  const c = taskCounts(), list = TASKS.filter(TASK_RULE[T.tf])
  const f = [['needs', 'Needs me'], ['open', 'Open'], ['filed', 'Filed']]
  const planBar = ([d, n], status) => `<span class="steps"><span class="seg-run">${Array.from({ length: n }, (_, i) => `<i class="${i < d ? 'on' : ''} ${status === 'run' && i === d ? 'cur' : ''}"></i>`).join('')}</span><span class="n num">${d}/${n}</span></span>`
  return `<div class="wb col">
    <div class="bar"><div class="seg" role="group" aria-label="Tasks">${f.map(([k, l]) => `<button data-set="tf:${k}" aria-pressed="${T.tf === k}">${l}<span class="count">${c[k]}</span></button>`).join('')}</div>
      <label class="finder"><svg><use href="#i-search"/></svg>Filter this project's tasks</label><span class="sp"></span>
      <button class="btn secondary">${icon('terminal')}Open in terminal</button><button class="btn primary">${icon('plus')}New task</button></div>
    <section class="panel"><div class="panel-b"><div class="ledger"><div class="lh"><span>Status</span><span>Task</span><span>Title</span><span>Plan</span><span>Updated</span><span></span></div>
      ${list.map((t) => `<div class="lr"><span><span class="pill ${t.status}">${TASK_LABEL[t.status]}</span></span><span class="code">${t.code}</span><span class="title">${t.title}</span>${planBar(t.plan, t.status)}<span class="when">${t.when}</span><span class="go" title="Open">${icon('open')}</span></div>`).join('')}</div></div></section>
    <div class="adh"><span>Plan adherence</span><span class="well" title="128 of 128 steps accepted first time"><i></i></span><span><b>128 of 128</b> accepted first time</span><span><b>0</b> sent back</span><span><b>5</b> deviations</span></div></div>`
}

function viewBlueprint() {
  const parts = [['Blueprint', 'Read in every context of the project', 2100, 'var(--accent)'], ['Components', '6 summaries; a description only when a task names it', 1400, 'var(--anchor)'], ['Connections', '4 bindings, labels and targets', 300, 'var(--filed)'], ['Pages', '4 titles, one line each', 60, 'var(--session)']]
  const total = parts.reduce((s, p) => s + p[2], 0)
  return `<div class="wb bp"><section class="panel"><div class="panel-h"><div><h2>Blueprint</h2><p class="sub">What <span class="anchor">/rk project:vega</span> hands to Claude</p></div><span class="sp"></span><button class="btn quiet">Replace from file</button><div class="seg"><button aria-pressed="false">Write</button><button aria-pressed="true">Read</button></div></div>
    <div class="reader" style="grid-template-columns:minmax(0,1fr)"><div class="doc"><div class="doc-col"><h1>Vega</h1><p class="lead">The payments backend behind the checkout: it prices an order, charges it through the acquirer and records the result in the ledger.</p>
    <h2>Stack</h2><p>Go services on Kubernetes, PostgreSQL per service, an event bus between them. One repository, one release train.</p>
    <h2>How to work here</h2><ul><li>Money is an integer count of minor units with a currency code.</li><li>A migration ships with the code that needs it and is safe with the old code still live.</li><li>Every change goes through a merge request linked to its task.</li></ul>
    <h2>Where things are</h2><p>The service pages list each part; the reference page holds the architecture, the data model and the release procedure.</p></div></div></div></section>
    <aside class="panel" aria-label="What a session receives"><div class="panel-h"><h2>Handed to a session</h2></div><div class="handed">
      <div class="total"><b>≈ ${(total / 1000).toFixed(1)}k</b><span>tokens, in every context of this project</span></div>
      <div class="mix" aria-hidden="true">${parts.map((p) => `<i style="flex:${p[2]};background:${p[3]}"></i>`).join('')}</div>
      <ul class="parts">${parts.map((p) => `<li><span class="k" style="background:${p[3]}"></span><b>${p[0]}</b><span class="n num">${fmt(p[2])}</span><span class="d">${p[1]}</span></li>`).join('')}</ul>
      <div class="holdback">Kept out of context: the text of every page, 62,410 + 9,160 characters. A session reads one with <code>rekall_read_sections</code> when the work touches it.</div></div></aside></div>`
}

function viewComponents() {
  const c = COMPONENTS.find((x) => x.id === T.comp), folders = [...new Set(COMPONENTS.map((x) => x.folder.split('/')[0]))]
  const body = c.id === 'billing-api' ? COMP_BODY : `<h2 id="c-purpose">Purpose</h2><p>${c.summary}</p><h2 id="c-gotcha">Gotchas</h2><p>Nothing written yet beyond the summary. A scan of the folder, or Write, adds the rest.</p>`
  const heads = c.id === 'billing-api' ? COMP_HEADS : [COMP_HEADS[0], COMP_HEADS[5]]
  return `<div class="wb cp" style="grid-template-columns:288px minmax(0,1fr)"><nav class="rail" aria-label="Components"><div class="rail-h"><span class="t">Components<span class="n">${COMPONENTS.length}</span></span><button class="iconbtn" title="Scan folder: a session reads the folder and writes the components">${icon('spark')}</button><button class="iconbtn" title="New component">${icon('plus')}</button></div>
    <div class="tree">${folders.map((f) => `<div class="folder">${f}/</div>${COMPONENTS.filter((x) => x.folder.split('/')[0] === f).map((x) => `<button class="row" data-set="comp:${x.id}" ${x.id === T.comp ? 'aria-current="page"' : ''}><span class="name">${x.name}</span><span class="sum">${x.summary}</span></button>`).join('')}`).join('')}</div>
    <p class="rail-note">Every context carries each component's summary. A description is read with <code>rekall_read_components</code>.</p></nav>
    <section class="panel"><div class="panel-h"><div style="min-width:0"><h2>${c.name}</h2><p class="sub mono" style="font-family:var(--font-mono);font-size:var(--t-caption)">${c.folder}</p></div><span class="sp"></span><span class="sub">Edited ${c.edited}</span><div class="seg"><button aria-pressed="false">Write</button><button aria-pressed="true">Read</button></div></div>
    <div class="reader"><div class="doc" id="doc"><div class="doc-col"><p class="lead" style="font-size:var(--t-title);color:var(--text);margin-top:0">${c.summary}</p>${body}</div></div><aside class="toc"><div class="toc-h"><span>On this page</span><span class="mono num">${heads.length}</span></div>${heads.map(([l, id, t], i) => `<a class="${l} ${i === 0 ? 'on' : ''}" data-jump="${id}">${t}</a>`).join('')}</aside></div></section></div>`
}

function viewDiagrams() {
  const list = DIAGRAMS.filter((d) => T.dfilter === 'all' || (T.dfilter === 'project' ? d.scope === 'project' : d.scope !== 'project'))
  const f = [['all', 'All', DIAGRAMS.length], ['project', 'Project', DIAGRAMS.filter((d) => d.scope === 'project').length], ['task', 'Tasks', DIAGRAMS.filter((d) => d.scope !== 'project').length]]
  return `<div class="wb" style="grid-template-rows:auto minmax(0,1fr);gap:12px"><div class="bar"><div class="seg">${f.map(([k, l, n]) => `<button data-set="dfilter:${k}" aria-pressed="${T.dfilter === k}">${l}<span class="count">${n}</span></button>`).join('')}</div><span class="sp"></span><button class="btn primary">${icon('spark')}Generate one</button></div>
    <section class="panel"><div class="panel-b"><div class="cards">${list.map((d) => `<button class="card"><span class="thumb">${thumb(d.shape)}</span><span class="cb"><span class="ct">${d.title}</span><span class="cm"><span class="tag">${d.type}</span>${d.scope === 'project' ? '<span>Project</span>' : `<span class="anchor">${d.scope}</span>`}<span style="margin-left:auto">${d.when}</span></span></span></button>`).join('')}</div></div></section></div>`
}

const EDGES = [
  ['M230 72H310', 270, 62, 'pushed', ''], ['M510 72H590', 550, 62, 'green', ''], ['M790 72H870', 830, 62, 'ready', ''],
  ['M360 104V220', 370, 168, 'red', 'bad'], ['M460 220V104', 470, 168, 'fixed', 'loop'],
  ['M900 104V252H510', 700, 244, 'failing', 'bad'], ['M1030 104V220', 1040, 168, 'healthy', '']
]
const edgeSvg = ([d, x, y, label, cls]) => `<path class="edge ${cls}" d="${d}" marker-end="url(#ah)"/><text class="elabel" x="${x}" y="${y}" ${y < 100 || y > 240 ? 'text-anchor="middle"' : ''}>${label}</text>`

function viewDeployment() {
  const s = STEPS[T.gstep]
  const nodes = Object.entries(STEPS).map(([id, n]) => `<g class="node ${id === T.gstep ? 'sel' : ''} ${id === 'done' ? 'end' : ''}" data-set="gstep:${id}" tabindex="0" role="button" aria-label="${n.title}"><rect class="box" x="${n.x}" y="${n.y}" width="200" height="64" rx="8"/><text class="t" x="${n.x + 16}" y="${n.y + 27}">${n.title}</text>${n.conn ? `<text class="c" x="${n.x + 16}" y="${n.y + 50}">${n.conn}</text>` : ''}</g>`).join('')
  const edges = EDGES.map(edgeSvg).join('')
  return `<div class="wb dp"><nav class="rail" aria-label="Deployment graphs"><div class="rail-h"><span class="t">Graphs<span class="n">2</span></span><button class="iconbtn" title="New graph">${icon('plus')}</button></div>
    <button class="row" data-set="graph:release" ${T.graph === 'release' ? 'aria-current="page"' : ''}><span class="name">Release loop <span class="badge">release</span></span><span class="sub"><span>6 steps · 7 routes</span></span></button>
    <button class="row" data-set="graph:hotfix" ${T.graph === 'hotfix' ? 'aria-current="page"' : ''}><span class="name">Hotfix</span><span class="sub"><span>3 steps · 3 routes</span></span></button>
    <p class="rail-note">A graph is the route a commit takes to a running application, written once and run on any task.</p></nav>
    <section class="panel"><div class="panel-h"><div><h2>Release loop</h2><p class="sub">From a pushed commit to a checked release; a failure routes back to a fix</p></div><span class="sp"></span><button class="btn quiet">Tidy layout</button><button class="btn secondary">${icon('plus')}Add step</button></div>
      <div class="canvas"><svg viewBox="0 0 1190 310" preserveAspectRatio="xMidYMid meet" role="group" aria-label="Release loop graph"><defs><marker id="ah" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0 0L10 5L0 10z" fill="#8c96a6"/></marker></defs>${edges}${nodes}</svg></div></section>
    <aside class="panel" aria-label="Step"><div class="panel-h"><h2>${s.title}</h2></div><div class="inspect"><dl class="kvs">
      <div><dt>Instructions</dt><dd>${s.text}</dd></div>${s.conn ? `<div><dt>Reaches</dt><dd><span class="anchor">${s.conn}</span></dd></div><div><dt>Visit budget</dt><dd>${s.budget} ${s.budget === 1 ? 'visit' : 'visits'} before the flow stops and asks</dd></div>` : ''}</dl>
      ${s.routes.length ? `<div><div class="field-l">Routes by outcome</div><ul class="routes">${s.routes.map(([o, to]) => `<li><b>${o}</b><i>→</i>${STEPS[to].title}</li>`).join('')}</ul></div>` : ''}</div></aside></div>`
}

function viewConnections() {
  const miss = CONNECTIONS.filter((c) => !c.cred).length
  const block = (c) => { const [tag, hue] = KINDS[c.kind]; return `<section class="panel conn"><div class="conn-h"><span class="kind" style="background:${hue}">${tag}</span><div class="host">${c.title}<small>${c.meta}</small></div><span class="cred ${c.cred ? '' : 'miss'}"><i></i>${c.cred ? 'Credential stored' : 'No credential'}</span><button class="btn quiet">Open in Connections</button></div>
    <div class="bind"><div class="bh"><span>Label</span><span>Target</span><span>Note</span><span></span></div>${c.binds.map(([l, t, n]) => `<div class="br"><span class="lab">${l}</span><span class="tgt">${t}</span><span class="hint">${n}</span><button class="btn quiet unbind" style="justify-self:end">Unbind</button></div>`).join('')}<div class="br"><button class="btn quiet" style="margin-left:-10px">${icon('plus')}Target</button></div></div></section>` }
  return `<div class="wb" style="grid-template-rows:auto minmax(0,1fr);gap:12px"><div class="bar"><div class="seg"><button data-set="cfilter:all" aria-pressed="${T.cfilter === 'all'}">All<span class="count">${CONNECTIONS.length}</span></button><button data-set="cfilter:miss" aria-pressed="${T.cfilter === 'miss'}">Needs a credential<span class="count">${miss}</span></button></div><span class="sp"></span>
    <button class="btn quiet">Manage connections</button><button class="btn secondary">${icon('spark')}Find repositories</button><button class="btn primary">${icon('plus')}New connection</button></div>
    <div style="overflow-y:auto;min-height:0">${CONNECTIONS.filter((c) => T.cfilter === 'all' || !c.cred).map(block).join('')}
    ${T.cfilter === 'all' ? `<section class="panel conn"><div class="panel-h"><h2>Shared with Fintech Co, not used by Vega</h2></div><div class="bind"><div class="bh"><span>Connection</span><span>Scope</span><span>Credential</span><span></span></div>${SHARED.map(([k, t, s, cr]) => `<div class="br bindable"><span class="lab">${t}</span><span class="tgt" style="font-family:var(--font-sans);color:var(--muted)">${s}</span><span class="cred ${cr ? '' : 'miss'}"><i></i>${cr ? 'Stored' : 'No credential'}</span><button class="btn secondary" style="justify-self:end">Bind</button></div>`).join('')}</div></section>` : ''}</div></div>`
}

function viewSettings() {
  const panes = [['general', 'General'], ['repo', 'Repository'], ['tm', 'Task manager'], ['checks', 'Checks']]
  const sw = (on) => `<span class="switch ${on ? 'on' : ''}" role="switch" aria-checked="${on}"></span>`
  let body = ''
  if (T.set === 'general') body = row(['Title', 'Shown in the navigator and the switcher.'], '<div class="fld">Vega</div>') + row(['Label and anchor', 'A new label moves the anchor sessions use.'], '<div class="fld mono">vega<small>/rk project:vega</small></div>') + row(['Task key', 'Opens the code of every task: VG-12.'], '<div class="fld mono">VG</div>') + row(['Company', ''], '<div class="fld">Fintech Co<small>change</small></div>') + row(['Status', ''], '<div class="fld">Active<small>▾</small></div>') + row(['Description', 'Every context of the project carries it.'], '<div class="fld area">The payments backend behind the checkout.</div>') + `<div class="danger-zone"><p><b style="color:var(--text);font-weight:500">Delete project.</b> Removes Vega with its tasks, pages, components and bindings. Nothing keeps them.</p><button class="btn danger secondary">Delete project</button></div>`
  else if (T.set === 'repo') body = row(['Folder', 'Where sessions work and checks run.'], '<div class="fld mono">/Users/dev/vega<small>Browse</small></div>') + row(['Main branch', 'Where a task\'s branch starts and is merged. Empty takes the folder\'s current branch.'], '<div class="fld mono">main</div>') + row(['Merge through a pull request', 'Push the branch and open a request when a task is claimed; accepting merges it.'], sw(true)) + row(['Commit on claim', 'Commit the task\'s work when a session claims it.'], sw(false))
  else if (T.set === 'tm') body = row(['Task manager', 'A plain task creates an issue there with its title alone. At most one binding.'], `<div class="choice2"><div class="opt on"><span class="r"></span><span>fintech/vega/api</span><span class="m">GitLab · api</span></div><div class="opt"><span class="r"></span><span>VG</span><span class="m">Jira · issues · no credential</span></div><div class="opt"><span class="r"></span><span>None</span><span class="m"></span></div></div>`)
  else body = `<div class="panel-b" style="flex:none"><div class="checks"><div class="bh" style="--cols:var(--cols);padding:0 24px;display:grid;grid-template-columns:160px minmax(0,1fr) 200px 80px 72px;column-gap:16px"><span>Name</span><span>Command</span><span>Folder</span><span>Timeout</span><span></span></div>${CHECKS.map(([n, c, f, t]) => `<div class="br" style="grid-template-columns:160px minmax(0,1fr) 200px 80px 72px;padding:0 24px"><span>${n}</span><span class="tgt">${c}</span><span class="hint mono" style="font-family:var(--font-mono)">${f || 'project folder'}</span><span class="num">${t}s</span><span style="justify-self:end;color:var(--subtle)">↑ ↓</span></div>`).join('')}<div class="br" style="padding:0 24px"><button class="btn quiet" style="margin-left:-10px">${icon('plus')}Add a check</button></div></div></div>`
  return `<div class="wb st"><nav class="rail" aria-label="Settings">${panes.map(([k, l]) => `<button class="row" data-set="set:${k}" ${T.set === k ? 'aria-current="page"' : ''}><span class="name">${l}</span></button>`).join('')}</nav>
    <section class="panel"><div class="panel-h"><h2>${panes.find((p) => p[0] === T.set)[1]}</h2>${T.set === 'checks' ? '<p class="sub">Run before a step\'s claim is accepted</p>' : ''}</div><div class="panel-b">${body}</div>${T.set === 'checks' ? '' : `<div class="sbar"><button class="btn primary">Save</button><button class="btn quiet">Revert</button><span class="note">Saved together: a new label moves the anchor.</span></div>`}</section></div>`
}

const VIEWS = { tasks: viewTasks, blueprint: viewBlueprint, components: viewComponents, diagrams: viewDiagrams, deployment: viewDeployment, connections: viewConnections, settings: viewSettings }
function tabCounts() { return { tasks: taskCounts().open, pages: null, components: COMPONENTS.length, diagrams: DIAGRAMS.length, deployment: 2, connections: CONNECTIONS.length } }
