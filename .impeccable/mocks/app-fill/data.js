/* Demonstration data for the whole mockup. Synthetic: nothing here is a real customer, advisory or number. */
const COMPANIES = [
  { id: 'nw', name: 'Northwind', hue: 205, projects: 3, connections: 5, tokens: '3.84M', adh: 100 },
  { id: 'hl', name: 'Halden Labs', hue: 42, projects: 2, connections: 3, tokens: '1.21M', adh: 94 },
  { id: 'ob', name: 'Oberon', hue: 262, projects: 2, connections: 2, tokens: '640k', adh: 88 }
]
const PROJECTS = [
  { id: 'vega', co: 'nw', title: 'Vega', status: 'Active', hue: 205, desc: 'The payments backend behind the checkout: prices an order, charges it through the acquirer, records it in the ledger.', open: 11, review: 3, adh: 100, tokens: '1.59M', updated: '12 minutes ago' },
  { id: 'atlas', co: 'nw', title: 'Atlas', status: 'Active', hue: 160, desc: 'Catalogue and inventory service. Owns products, stock levels and the nightly supplier import.', open: 7, review: 1, adh: 100, tokens: '1.31M', updated: '2 hours ago' },
  { id: 'harbor', co: 'nw', title: 'Harbor', status: 'Active', hue: 188, desc: 'Customer-facing web app: storefront, account area and checkout flow.', open: 14, review: 0, adh: 97, tokens: '940k', updated: 'yesterday' },
  { id: 'lumen', co: 'hl', title: 'Lumen', status: 'Active', hue: 42, desc: 'Image pipeline for the lab: ingest, calibrate, stack and publish.', open: 5, review: 1, adh: 94, tokens: '810k', updated: '3 hours ago' },
  { id: 'quill', co: 'hl', title: 'Quill', status: 'Active', hue: 330, desc: 'Internal documentation generator that turns lab notebooks into a searchable handbook.', open: 3, review: 0, adh: 92, tokens: '400k', updated: '3 days ago' },
  { id: 'relay', co: 'ob', title: 'Relay', status: 'Active', hue: 262, desc: 'Message routing between the operations consoles and the field units.', open: 6, review: 2, adh: 90, tokens: '520k', updated: 'today' },
  { id: 'beacon', co: 'ob', title: 'Beacon', status: 'Paused', hue: 8, desc: 'Hosted status page for operations. Paused until the Relay migration lands.', open: 2, review: 0, adh: 85, tokens: '120k', updated: 'a month ago' }
]
const TASKS = [
  ['vega', 'VG-31', 'Reverse a settled charge from the ledger', 'review', [4, 4], '35.1k', 'an hour ago'],
  ['vega', 'VG-30', 'Webhook retries back off per endpoint', 'review', [5, 5], '22.4k', '3 hours ago'],
  ['vega', 'VG-28', 'Invoice rendering moves to its own worker', 'review', [6, 6], '41.0k', 'yesterday'],
  ['vega', 'VG-33', 'Acquirer timeouts leave a charge pending', 'run', [2, 5], '18.7k', 'now'],
  ['vega', 'VG-32', 'Reconciler settles pending charges every minute', 'run', [1, 4], '9.2k', '12 minutes ago'],
  ['vega', 'VG-34', 'Add a currency column to the invoice table', 'todo', [0, 3], '—', 'today'],
  ['vega', 'VG-29', 'Document the event bus topics', 'todo', [0, 2], '—', 'yesterday'],
  ['vega', 'VG-25', 'Pin the acquirer SDK version', 'done', [2, 2], '6.3k', 'last week'],
  ['vega', 'VG-23', 'Rename the pending state to authorised', 'filed', [3, 3], '11.8k', '2 weeks ago'],
  ['atlas', 'AT-18', 'Supplier import skips rows with a blank SKU', 'review', [3, 3], '14.9k', '2 hours ago'],
  ['atlas', 'AT-19', 'Stock level endpoint pages its result', 'run', [3, 6], '27.5k', '40 minutes ago'],
  ['atlas', 'AT-20', 'Add a reorder point to the product model', 'todo', [0, 4], '—', 'yesterday'],
  ['harbor', 'HB-44', 'Checkout keeps the basket after a failed payment', 'todo', [0, 5], '—', 'yesterday'],
  ['harbor', 'HB-43', 'Account area: download invoices as PDF', 'todo', [0, 3], '—', '2 days ago'],
  ['harbor', 'HB-40', 'Storefront search highlights the matched term', 'done', [4, 4], '19.3k', 'last week'],
  ['lumen', 'LM-12', 'Calibration frames are matched by exposure and temperature', 'review', [5, 5], '30.2k', '3 hours ago'],
  ['lumen', 'LM-13', 'Stacking job reports progress per frame', 'todo', [0, 3], '—', 'today'],
  ['relay', 'RL-9', 'Field units reconnect without losing the last message', 'review', [4, 4], '24.8k', 'today'],
  ['relay', 'RL-10', 'Route by unit group, not by console', 'review', [6, 6], '38.6k', '5 hours ago'],
  ['relay', 'RL-8', 'Retire the legacy polling endpoint', 'todo', [0, 2], '—', '3 days ago']
].map(([project, code, title, status, plan, tokens, when]) => ({ project, code, title, status, plan, tokens, when }))
const TASK_LABEL = { todo: 'To do', run: 'Running', review: 'Awaiting review', done: 'Done', filed: 'Filed' }
const RULE = { needs: (t) => t.status === 'review', running: (t) => t.status === 'run', open: (t) => t.status !== 'filed', filed: (t) => t.status === 'filed' }

const COMPONENTS = [
  { id: 'billing-api', folder: 'billing/api', name: 'billing-api', summary: 'Prices an order, takes the charge through the acquirer and asks the ledger to record it. Owns prices, charges and invoices.', edited: '2 hours ago' },
  { id: 'billing-worker', folder: 'billing/worker', name: 'billing-worker', summary: 'Renders invoices and delivers webhooks from the same image as billing-api, started with --worker.', edited: 'yesterday' },
  { id: 'ledger', folder: 'ledger', name: 'ledger', summary: 'The only source of truth for money. Entries are appended, never updated; a mistake is a second, reversing entry.', edited: '3 days ago' },
  { id: 'gateway', folder: 'gateway', name: 'gateway', summary: 'Authenticates the caller and stamps the correlation id every call and event carries.', edited: 'last week' },
  { id: 'helm-charts', folder: 'deploy/helm', name: 'helm-charts', summary: 'One chart per service and the values for staging and production.', edited: 'last week' },
  { id: 'pipeline', folder: 'ci', name: 'pipeline', summary: 'Builds each image on a tag, applies migrations and rolls the services out one at a time, ledger last.', edited: '2 weeks ago' }
]
const PAGES = [
  { id: 'ref', title: 'Platform reference', chars: 62410, edited: '2 hours ago' },
  { id: 'dep', title: 'Deployment', chars: 3180, edited: 'yesterday' },
  { id: 'con', title: 'Contributing', chars: 4720, edited: '3 days ago' },
  { id: 'glo', title: 'Glossary', chars: 1260, edited: 'last week' }
]
const DIAGRAMS = [
  ['vega', 'Charge request path', 'Sequence', 'project', 'today', 'seq'],
  ['vega', 'Service dependencies', 'Flowchart', 'project', 'yesterday', 'flow'],
  ['vega', 'Ledger tables', 'ER', 'project', '3 days ago', 'er'],
  ['vega', 'Reversal flow', 'Sequence', 'task:VG-31', 'an hour ago', 'seq'],
  ['atlas', 'Supplier import stages', 'Flowchart', 'task:AT-18', '2 hours ago', 'flow'],
  ['atlas', 'Stock level model', 'ER', 'project', 'last week', 'er'],
  ['relay', 'Reconnect handshake', 'Sequence', 'task:RL-9', 'today', 'seq'],
  ['lumen', 'Calibration pipeline', 'Flowchart', 'project', '3 days ago', 'flow']
].map(([project, title, type, scope, when, shape]) => ({ project, title, type, scope, when, shape }))

const KINDS = { gitlab: ['GL', '#f0a05a', 'GitLab'], github: ['GH', '#b4b9ff', 'GitHub'], kubernetes: ['K8', '#7aa2f7', 'Kubernetes'], jira: ['JR', '#6cb3f0', 'Jira'], grafana: ['GF', '#f08a4b', 'Grafana'] }
const CONNECTIONS = [
  { id: 'c1', kind: 'gitlab', title: 'gitlab.northwind.dev', meta: 'root path northwind/', cred: true, cos: ['Northwind'], used: 4, binds: [['vega', 'vega', 'northwind/vega/api'], ['vega', 'web', 'northwind/vega/web'], ['atlas', 'api', 'northwind/atlas/api'], ['harbor', 'app', 'northwind/harbor/app']] },
  { id: 'c2', kind: 'github', title: 'github.com', meta: 'owner haldenlabs', cred: true, cos: ['Halden Labs'], used: 2, binds: [['lumen', 'pipeline', 'haldenlabs/lumen'], ['quill', 'docs', 'haldenlabs/quill']] },
  { id: 'c3', kind: 'kubernetes', title: 'vega-prod', meta: 'context vega-prod · https://k8s.northwind.dev', cred: true, cos: ['Northwind'], used: 2, binds: [['vega', 'prod', 'vega'], ['vega', 'stg', 'vega-stg']] },
  { id: 'c4', kind: 'jira', title: 'northwind.atlassian.net', meta: 'Jira Cloud', cred: false, cos: ['Northwind', 'Oberon'], used: 1, binds: [['vega', 'issues', 'VG']] },
  { id: 'c5', kind: 'grafana', title: 'grafana.northwind.dev', meta: 'Grafana', cred: true, cos: ['Northwind'], used: 1, binds: [['vega', 'dash', 'vega-overview']] }
]
const ADVISORIES = [
  ['critical', 'ADV-2026-0412', 'openssl-sys', '0.9.101', ['Vega', 'Atlas'], 'Reaches', 'Vega calls TLS through it on every acquirer request'],
  ['critical', 'ADV-2026-0398', 'lodash', '4.17.20', ['Harbor'], 'Not reachable', 'Only the build tooling imports it'],
  ['high', 'ADV-2026-0377', 'tokio', '1.38.0', ['Vega', 'Relay'], 'Reaches', 'The worker pool is configured with the affected flag'],
  ['high', 'ADV-2026-0351', 'serde_yaml', '0.9.30', ['Atlas'], 'Unassessed', ''],
  ['high', 'ADV-2026-0344', 'jsonwebtoken', '8.3.0', ['Harbor', 'Relay'], 'Unassessed', ''],
  ['medium', 'ADV-2026-0310', 'axios', '1.6.2', ['Harbor'], 'Not reachable', 'Server-side only; the affected adapter is not used'],
  ['medium', 'ADV-2026-0287', 'chrono', '0.4.31', ['Vega', 'Atlas', 'Lumen'], 'Unassessed', ''],
  ['low', 'ADV-2026-0260', 'minimist', '1.2.5', ['Harbor'], 'Not reachable', 'Dev dependency of the test runner']
]
const MONTH_HOURS = { 2: 5.5, 3: 7.2, 4: 6.8, 5: 8.1, 6: 4.0, 9: 7.0, 10: 6.4, 11: 7.5, 12: 6.9, 13: 3.5, 16: 8.4, 17: 5.2, 18: 0, 19: 0, 20: 0, 23: 0, 24: 0, 25: 0, 26: 0, 27: 0, 30: 0 }
const CLAUDE = {
  skills: [['impeccable', 'user', 'Design, redesign, critique and polish a frontend', '18 KB', 'last week'], ['writing-style', 'user', 'Apply professional writing style rules to technical prose', '3 KB', '2 weeks ago'], ['rk', 'project', 'Load a Rekall working context by anchor', '6 KB', 'yesterday'], ['simplify', 'plugin', 'Review changed code for reuse and simplification', '4 KB', 'last month']],
  commands: [['/rk', 'project', 'Load a Rekall working context by anchor', '6 KB', 'yesterday'], ['/review', 'user', 'Review the current diff', '2 KB', '2 weeks ago']],
  hooks: [['guard-write', 'user', 'Refuses writes outside the project folder', '1 KB', '3 weeks ago'], ['format-on-edit', 'project', 'Runs the formatter after an edit', '1 KB', 'last month']],
  memory: [['MEMORY.md', 'project', 'Index of what Claude remembers about this project', '2 KB', 'today'], ['feedback_commit_style.md', 'project', 'Conventional commits, no trailers', '1 KB', 'last week']]
}
