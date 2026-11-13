/* Demo data, taken from the seeded console. Mock only: in memory, nothing persists. */
window.R = window.R || {}
R.TODAY = { y: 2026, m: 10, d: 13 } // 13 Nov 2026

R.companies = [
  { id: 'acme', name: 'Acme', hue: '#e8575f', initial: 'A' },
  { id: 'art', name: 'Artcraft', hue: '#f0f2f5', initial: 'A' },
  { id: 'esa', name: 'ESA', hue: '#4ea1e0', initial: 'E' },
  { id: 'vf', name: 'VFORGE', hue: '#9f78e6', initial: 'V' }
]

R.projects = [
  { id: 'mal', co: 'acme', name: 'Malwarebytes', label: 'malwarebytes', hue: '#a78bfa', tokens: '99.3k', updated: 'yesterday', status: 'Active' },
  { id: 'lig', co: 'art', name: 'Lightcraft', label: 'lightcraft', hue: '#f0955a', tokens: '6.84k', updated: '11 days ago', status: 'Active' },
  { id: 'cos', co: 'esa', name: 'COSMIC', label: 'cosmic', hue: '#6c8cff', tokens: '1.59M', updated: '28 days ago', status: 'Active' },
  { id: 'mae', co: 'esa', name: 'MAESTRO', label: 'maestro', hue: '#b8a98a', tokens: '0', updated: '31/08/2026', status: 'Active' },
  { id: 'sai', co: 'esa', name: 'SAIGO', label: 'saigo', hue: '#46c9b0', tokens: '57.8k', updated: '4 days ago', status: 'Active' },
  { id: 'stv', co: 'esa', name: 'STVW', label: 'stvv', hue: '#e263b0', tokens: '7.7M', updated: '12/10/2026', status: 'Active' },
  { id: 'rvr', co: 'vf', name: 'Rekall vR', label: 'rekall-vrust', hue: '#eaa43c', tokens: '21.4M', updated: 'today', status: 'Active' },
  { id: 'hyp', co: 'vf', name: 'HyperCV', label: 'hypercv', hue: '#7fd66a', tokens: '296k', updated: '26/09/2026', status: 'Active' },
  { id: 'mlx', co: 'vf', name: 'MLX Studio', label: 'mlx-studio', hue: '#4ea1e0', tokens: '1.1M', updated: '26/09/2026', status: 'Active' },
  { id: 'osx', co: 'vf', name: 'OSX Tools', label: 'osx-tools', hue: '#f0f2f5', tokens: '6.99k', updated: '11 days ago', status: 'Active' },
  { id: 'pen', co: 'vf', name: 'Penna', label: 'penna', hue: '#46c9b0', tokens: '179k', updated: '2 days ago', status: 'Active' },
  { id: 'pfa', co: 'vf', name: 'Psifatture', label: 'psifatture', hue: '#b8a98a', tokens: '88k', updated: '5 days ago', status: 'Active' }
]

/* kind: epic marker only. status: TODO | IN_PROGRESS | DONE.  taskReview: a task handed in whole (no steps). */
R.tasks = [
  { id: 't-doc', p: 'mal', title: 'Create the requested documents', label: 'create-the-requested-documents', status: 'IN_PROGRESS', ago: '2d', tokens: '0', desc: 'Prepare the documents the client asked for after the first meeting.' },
  { id: 't-int', p: 'mal', title: 'Review based on the interview', label: 'review-based-on-the-interview', status: 'IN_PROGRESS', ago: '16h', tokens: '5.54k', taskReview: true, desc: 'Consider this is a project that is used in the evaluation for the interview. Do you have some feedback about the code and the structure?', wrapup: { by: 'Claude', ago: '16 hours ago', text: 'The interview review is delivered as one short document, INTERVIEW.md at the project root. It condenses the findings into five points: structure, tests, naming, error handling and the build.' } },
  { id: 't-sol', p: 'mal', title: 'Solution improvements', label: 'solution-improvements', status: 'IN_PROGRESS', ago: '1d', tokens: '93.7k', taskReview: true, desc: 'Propose and apply improvements to the current solution.' },
  { id: 't-img', p: 'lig', title: 'Delete images on mac', label: 'delete-images-on-mac', status: 'IN_PROGRESS', ago: '3d', tokens: '6.84k', desc: 'Remove the unused images shipped in the macOS bundle.' },
  { id: 't-prom', p: 'cos', title: 'Prometheus configuration in AINABLER', label: 'metrics-in-ainabler', status: 'IN_PROGRESS', ago: '4d', tokens: '33.6k', notes: 2, desc: 'Add the Prometheus scrape configuration to the AINABLER chart.' },
  { id: 't-ref', p: 'cos', title: 'Refactoring applicatio', label: 'refactoring-applicatio', status: 'IN_PROGRESS', ago: '5d', tokens: '95.1k', notes: 1, desc: 'Split the application module and remove the shared state.' },
  { id: 't-cv', p: 'cos', title: 'Work experience', label: 'voice-in-the-cv', status: 'IN_PROGRESS', ago: '6d', tokens: '15.8k', notes: 1, desc: 'Rewrite the work experience section in a direct voice.' },
  { id: 't-aira1', p: 'cos', title: 'AIRA for XMM: development and deployment environment on c3h', label: 'aira-xmm-environment-c3h', status: 'TODO', ago: '6d', tokens: '0', desc: 'Set up the development and deployment environment on the c3h cluster.' },
  { id: 't-aira2', p: 'cos', title: 'AIRA for XMM: ingest XMM data and run similarity searches', label: 'aira-xmm-data-ingestion', status: 'TODO', ago: '6d', tokens: '0', desc: 'Ingest the XMM data and run the first similarity searches.' },
  { id: 't-conn', p: 'cos', title: 'Data connector: one access layer to the operations data', label: 'data-connector', status: 'TODO', ago: '9d', tokens: '0', desc: 'One access layer in front of the operations databases.' },
  { id: 't-sai', p: 'sai', title: 'Define many information for SAIGO', label: 'define-many-informatio-for-saigo', status: 'IN_PROGRESS', ago: '2d', tokens: '57.8k', taskReview: true, notes: 2, desc: 'Define the information model the schedule and the milestones need.' },
  { id: 't-sch', p: 'sai', title: 'Schedule and milestones', label: 'schedule-and-milestones', status: 'TODO', ago: '4d', tokens: '0', desc: 'Lay out the project schedule and its milestones.' },
  { id: 't-pypi', p: 'stv', title: 'access to pypi repository in local', label: 'access-to-pypi-repository-in-local', status: 'IN_PROGRESS', ago: '8d', tokens: '27.8k', notes: 1, desc: 'Reach the internal pypi mirror from the local environment.' },
  { id: 't-metric', p: 'stv', title: 'check metricbridge integration situation', label: 'check-metricbridge-integration-situation', status: 'IN_PROGRESS', ago: '1d', tokens: '731k', time: '5h', sessions: 10, desc: 'Context: in the project folder we have the following repositories. I want to know which services already carry metricbridge, which still miss it, and what stops each one from being integrated.', wrapup: { by: 'Claude', ago: 'yesterday', text: 'Metricbridge is integrated in 13 services on branch feat/remaining project integration (codev), and all 13 run. Nine merge requests are open on codev, none has conflicts, nothing is merged. The reference pattern is mcps/weaviate-mcp; the services that remain are listed in the note on merge requests.' }, deviations: [{ step: 22, text: 'The SSE finding changed the order of the two MCP upgrades; the plan was re-ordered by the person.' }] },
  { id: 't-front', p: 'stv', title: 'frontend notification', label: 'frontend-notification', status: 'IN_PROGRESS', ago: '3d', tokens: '23.6k', notes: 6, desc: 'Show a notification in the frontend when an integration run ends.' },
  { id: 't-open', p: 'stv', title: 'Metricbridge integration: open points', label: 'metricbridge-open-points', status: 'IN_PROGRESS', ago: '2d', tokens: '162k', desc: 'List the open points left by the integration and settle each.' },
  { id: 't-mcp', p: 'stv', title: 'MCP servers: spans and trace ids in logs', label: 'mcp-spans-and-trace-logs', status: 'IN_PROGRESS', ago: '9d', tokens: '0', desc: 'Carry the span and trace ids into the MCP servers logs.' },
  { id: 't-rep', p: 'stv', title: 'Report work done', label: 'report-work-done', status: 'IN_PROGRESS', ago: '2d', tokens: '6.95k', desc: 'Write the weekly report of the work done.' },
  { id: 't-span', p: 'stv', title: 'Testing the current span id in h5 services', label: 'testing-span-id-h5', status: 'IN_PROGRESS', ago: '4d', tokens: '41.2k', taskReview: true, desc: 'Check that the current span id reaches the h5 services.' },
  { id: 't-co', p: 'stv', title: 'Checkout all the projects', label: 'checkout-all-the-projects', status: 'TODO', ago: '7d', tokens: '32.9k', desc: 'Clone every project of the group in one pass.' },
  { id: 't-impr', p: 'rvr', title: 'Improved task implementation', label: 'change-task-label', status: 'IN_PROGRESS', ago: '17m', tokens: '1.2M', time: '1h', sessions: 3, desc: 'Change how a task label is derived and keep the old anchors working.', wrapup: null },
  { id: 't-pr', p: 'rvr', title: 'Pull request workflow: push, open the PR, merge it on acceptance', label: 'pull-request-workflow', status: 'DONE', ago: '2d', tokens: '2.4M', filed: true, desc: 'Push, open the pull request and merge it on acceptance.' },
  { id: 't-back', p: 'rvr', title: 'Backend review', label: 'backend-review', status: 'DONE', ago: '3d', tokens: '1.1M', filed: true, desc: 'Review the backend for schema and index gaps.' },
  { id: 't-hy', p: 'hyp', title: 'Rebuild commit_reference so a deleted step keeps its commits', label: 'commit-reference-rebuild', status: 'DONE', ago: '26/09', tokens: '88k', filed: true, desc: '' },
  { id: 't-ml', p: 'mlx', title: 'Model cache that survives a restart', label: 'model-cache', status: 'DONE', ago: '26/09', tokens: '412k', filed: true, desc: '' },
  { id: 't-os', p: 'osx', title: 'Menu bar status item', label: 'menu-bar-status-item', status: 'DONE', ago: '11d', tokens: '6.99k', filed: true, desc: '' },
  { id: 't-pe', p: 'pen', title: 'Ink pressure curve', label: 'ink-pressure-curve', status: 'IN_PROGRESS', ago: '2d', tokens: '179k', desc: 'Tune the pressure curve of the ink engine.' },
  { id: 't-fs1', p: 'stv', title: 'Flyway integration for the ingestion registry', label: 'flyway-integration', status: 'DONE', ago: '9d', tokens: '210k', filed: true, desc: '' },
  { id: 't-fs2', p: 'cos', title: 'Usage collector: how it runs and how it is designed', label: 'usage-collector-design', status: 'DONE', ago: '3w', tokens: '640k', filed: true, desc: '' }
]
R.filedTotals = { mal: 42, lig: 9, cos: 61, sai: 18, stv: 157, rvr: 96, hyp: 12, mlx: 7, osx: 4, pen: 3, mae: 0 }

/* step state: DRAFT | OPEN | RUNNING | CLAIMED | DONE */
const metricTitles = [
  'Read solution-overview.md and the merge-requests note',
  'Integrate metricbridge in stvv-core-backend (log-preprocessing, knowledge-manager, ingestion-registry) and push',
  'Verify stvv-core-backend integration after propagation',
  'Add the metricbridge deploy settings to the stvv-core-backend apps',
  'Verify the metricbridge deploy settings of the stvv-core-backend apps',
  'Check the stvv-core-backend apps in Grafana',
  'Integrate metricbridge in stvv-knowledge-loader and push',
  'Verify stvv-knowledge-loader integration after propagation',
  'Integrate metricbridge in the registry-mcp service (FastAPI)',
  'Integrate metricbridge in the email-mcp and jira-zephyr-mcp services (FastMCP)',
  'Upgrade gitlab-mcp to Spring Boot 4.1.0 and add metricbridge-lib-java 0.3.0',
  'Integrate metricbridge in the sftp-mos-mcp service (FastMCP)',
  'Integrate metricbridge in the postgresql-mcp service (FastMCP)',
  'Integrate metricbridge in the neo4j-mcp service (FastMCP)',
  'Verify the MCP services after propagation',
  'Check the MCP services in Grafana',
  'Add the deploy settings to the MCP services',
  'Verify the deploy settings of the MCP services',
  'Open the merge requests on codev',
  'Link each merge request in the merge-requests note',
  'Re-run the tempo query for every service',
  'Redeploy when another branch changes the shared library',
  'Write the redeploy procedure in the note',
  'Report the integration situation'
]
R.steps = []
let sid = 0
const mkStep = (task, title, state, extra) => R.steps.push(Object.assign({ id: 's' + ++sid, task, title, state, ago: state === 'CLAIMED' ? '5 days ago' : '' }, extra || {}))
metricTitles.forEach((t, i) => mkStep('t-metric', t, i === 0 ? 'DONE' : 'CLAIMED', { ago: i === 0 ? '' : ['7 days ago', '7 days ago', '7 days ago', '7 days ago', '6 days ago', '6 days ago', '6 days ago', '5 days ago', '5 days ago', '4 days ago'][i - 1] || '3 days ago' }))
;[['Fix the open points of the bridge configuration', 'CLAIMED', '2 days ago'], ['Write down the decisions taken on each open point', 'CLAIMED', '2 days ago'], ['Settle the last two points with the team', 'OPEN'], ['Update the note with the final state', 'OPEN']].forEach((s) => mkStep('t-open', s[0], s[1], { ago: s[2] || '' }))
mkStep('t-front', 'Show the run result in the notification area', 'OPEN')
;[['Read the label derivation in the catalog', 'DONE'], ['Keep the old anchors resolving to the same task', 'DONE'], ['Change the label derivation and add the migration', 'RUNNING'], ['Cover the old and the new anchor with a test', 'OPEN'], ['Update the documentation', 'OPEN'], ['Draft: rename the label field in the API', 'DRAFT']].forEach((s) => mkStep('t-impr', s[0], s[1]))
;[['Add the pull request flow as a project setting', 'DONE'], ['Add push to the worktree port', 'DONE'], ['Add the pull request port and its GitHub implementation', 'DONE'], ['Store the pull request on the task', 'DONE'], ['Open the pull request when the task\'s work is claimed', 'DONE'], ['Merge the pull request when the task is accepted', 'DONE'], ['Resolve a conflicting pull request with the merge session', 'DONE'], ['Add GitLab merge requests', 'DONE'], ['Show the pull request on the task in the console', 'DONE'], ['Update the documentation', 'DONE']].forEach((s) => mkStep('t-pr', s[0], s[1]))
;[['Rebuild commit_reference so a deleted step keeps its commits', 'DONE'], ['Index the remaining foreign-key columns', 'DONE'], ['Guard foreign-key indexes with a schema test', 'DONE']].forEach((s) => mkStep('t-back', s[0], s[1]))
R.steps.forEach((s) => (s.n = R.steps.filter((x) => x.task === s.task && R.steps.indexOf(x) <= R.steps.indexOf(s)).length))

const claim = (summary, checks) => ({ summary, checks })
R.claims = {
  1: claim('Branch <code>feat/remainig-project-integration</code> created from <code>develop</code>. Metricbridge added to the three apps following <code>apps/process-admin</code>; deploy settings left for step 4. Unpushed local work kept in <code>git stash -u</code>.', [['Build', 'ok', 'passed'], ['Lint', 'ok', 'passed'], ['Tests', 'ok', '14 / 14']]),
  2: claim('Ran the three apps against the staging collector for ten minutes. Spans and trace ids reach tempo with the service name set; no gap in the sequence.', [['Build', 'ok', 'passed'], ['Tests', 'ok', '9 / 9'], ['Grafana panel', 'ok', 'present']]),
  3: claim('Added <code>METRICBRIDGE_ENDPOINT</code> and the sampling rate to the helm values of the three apps, one values file per environment.', [['Build', 'ok', 'passed'], ['Helm lint', 'ok', 'passed']]),
  4: claim('Rendered the chart for each environment and compared the output with the reference app. One difference left on purpose: the staging sampling rate.', [['Helm template', 'ok', 'matches reference'], ['Diff to reference', 'bad', '1 difference']])
}
R.defaultClaim = claim('The session changed the files named in the step and ran the checks configured on the project. Open the diff to read what changed.', [['Build', 'ok', 'passed'], ['Lint', 'ok', 'passed'], ['Tests', 'ok', 'passed']])

R.notes = [
  { id: 'n1', task: 't-metric', title: 'merge-requests.md', body: '# Merge requests of the metricbridge integration\n\nAll nine are drafts on `codev`, from `feat/remainig-project-integration` to `develop`, assigned to `vmcasale`. Opened on 2026-11-09. None has conflicts. Nothing is merged.\n\n| Repository | Merge request | What it carries |\n|---|---|---|\n| stvv-core-backend | !195 | metricbridge integration and deploy settings, plus the KPI route of `test-quality-api` |\n| stvv-knowledge-loader | !52 | metricbridge integration and deploy settings |\n| registry-mcp | !7 | metricbridge integration (FastAPI) |\n| email-mcp | !11 | metricbridge integration (FastMCP) |\n| jira-zephyr-mcp | !10 | metricbridge integration (FastMCP) |\n| gitlab-mcp | !21 | upgrade to Spring Boot 4.1.0, then `metricbridge-lib-java` 0.3.0 |' },
  { id: 'n2', task: 't-metric', title: 'mcp-sse-findings.md', body: '# MCP SSE findings\n\nThe SSE transport keeps the connection open, so the span of a request never ends until the client disconnects.\n\n## What we do\n\n- Close the span on the first event, not on the stream end.\n- Record the stream length as an attribute.\n- Keep the trace id on every event.' },
  { id: 'n3', task: 't-metric', title: 'redeploy-when-another-branch.md', body: '# Redeploy when another branch changes the shared library\n\nA service that depends on `metricbridge-lib` must be rebuilt when the library branch moves.\n\n1. Bump the version in the lock file.\n2. Rebuild the image.\n3. Redeploy and check the Grafana panel.' },
  { id: 'n4', task: 't-metric', title: 'verify-method.md', body: '# Verify method\n\nAfter each propagation, open the service dashboard and check three things: spans arrive, the service name is set, and the sampling rate matches the environment.' },
  { id: 'n5', task: 't-metric', title: 'tempo-query-caution.md', body: '# Tempo query caution\n\nA query over more than **six hours** is cut by the gateway. Narrow the range first, then widen it.' },
  { id: 'n6', task: 't-metric', title: 'solution-overview.md', body: '# Solution overview\n\nThe reference pattern is `mcps/weaviate-mcp`. Every integrated service sets the same four attributes, reads the endpoint from one variable, and exposes the sampling rate in its helm values.' },
  { id: 'n7', task: 't-front', title: 'notification-design.md', body: '# Notification design\n\nA run that ends raises one notification with the run result and a link to the task.' },
  { id: 'n8', task: 't-sai', title: 'information-proposal.md', body: '# Information proposal\n\nThe model carries the project, the milestone and the schedule line, each with an owner and a date.' },
  { id: 'n9', task: 't-sai', title: 'saigo-schedule-and-milestones.md', body: '# Schedule and milestones\n\nThree milestones: kickoff, first delivery, acceptance.' }
]

R.sessionsLog = [
  'claude --resume  project:rekall-vrust task:change-task-label',
  'Reading the task: 6 steps, 2 done, 1 running.',
  'Step 3 · Change the label derivation and add the migration',
  'Reading crates/rekall-core/src/catalog/label.rs',
  'Editing crates/rekall-core/src/catalog/label.rs',
  'Editing crates/rekall-store/migrations/0042_label_alias.sql',
  'cargo test -p rekall-core label',
  'test label::keeps_old_anchor ... ok',
  'test label::derives_from_title ... ok',
  'Claiming step 3 for your review.'
]

/* Time: minutes tracked by day (Nov 9..15, Mon..Sun) and company */
R.week = { start: '09/11/2026', end: '15/11/2026', days: ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'], byCompany: { vf: [100, 300, 400, 520, 57, 0, 0], esa: [28, 100, 36, 224, 51, 0, 0], acme: [0, 0, 0, 0, 30, 0, 0] } }
R.reportData = [
  { co: 'vf', project: 'Rekall vR', label: 'rekall-vrust', tasks: [
    { title: 'Pull request workflow: push, open the PR, merge it on acceptance', label: 'pull-request-workflow', time: '4h 40m', spark: [0, 0, 1, 3, 0, 0, 0], steps: ['Add the pull request flow as a project setting', 'Add push to the worktree port', 'Add the pull request port and its GitHub implementation', 'Store the pull request on the task', 'Open the pull request when the task\'s work is claimed', 'Merge the pull request when the task is accepted'] },
    { title: 'Backend review', label: 'backend-review', time: '4h 34m', spark: [0, 1, 1, 2, 0, 0, 0], steps: ['Rebuild commit_reference so a deleted step keeps its commits', 'Index the remaining foreign-key columns', 'Guard foreign-key indexes with a schema test', 'Delete the unused pub items'] },
    { title: 'Improved task implementation', label: 'change-task-label', time: '1h 02m', spark: [0, 0, 0, 0, 1, 0, 0], steps: ['Read the label derivation in the catalog', 'Keep the old anchors resolving to the same task'] }
  ] },
  { co: 'esa', project: 'STVW', label: 'stvv', tasks: [
    { title: 'check metricbridge integration situation', label: 'check-metricbridge-integration-situation', time: '2h 54m', spark: [1, 2, 0, 0, 1, 0, 0], steps: ['Read solution-overview.md and the merge-requests note'] },
    { title: 'frontend notification', label: 'frontend-notification', time: '46m', spark: [0, 1, 0, 0, 0, 0, 0], steps: [] }
  ] },
  { co: 'acme', project: 'Malwarebytes', label: 'malwarebytes', tasks: [{ title: 'Review based on the interview', label: 'review-based-on-the-interview', time: '30m', spark: [0, 0, 0, 0, 1, 0, 0], steps: [] }] }
]
R.calendarTotals = { 26: '3h 44m', 27: '2h 28m', 28: '8h 47m', 29: '7h 47m', 30: '5h 22m', 1: '22m', 2: '7h 40m', 3: '3h 34m', 4: '2h 59m', 5: '7h 13m', 6: '5h 15m', 9: '2h 53m', 10: '6h 50m', 11: '7h 36m', 12: '10h 44m', 13: '2h 41m' }
R.calendarPool = ['Complete the documents', 'Frontend system notes', 'Task reference', 'Backend review', 'Hearbeat logs', 'Metricbridge lib install', 'Refactor the workflow', 'Check service in grafana', 'Session in multi-step', 'Tasks steps session', 'Updated show...', 'Refactoring of the revision', 'Idea for a better flow', 'Create penna project', 'Pull request workflow', 'Workflow fixes']

R.diagrams = [
  { id: 'd1', p: 'mal', title: 'Lock Screen Scam Guard sequences', desc: 'Create sequence diagrams based on the implementation on Main assignment', kinds: ['Sequence', 'Sequence', 'Sequence', 'Sequence', 'Sequence'], when: 'yesterday', walk: true },
  { id: 'd2', p: 'cos', title: 'ainabler-usage-collector: Kubernetes watch and KFP API workflow', desc: 'What happens in the whole workflow of ainabler metrics. In particular I want to know how the watch reaches the store.', kinds: ['Sequence', 'Sequence', 'Sequence', 'Sequence', 'Sequence', 'Sequence'], when: '23 days ago', task: 'ainabler-metrics', walk: true },
  { id: 'd3', p: 'cos', title: 'kube resource', desc: '', kinds: ['Sequence'], when: '11/10/2026' },
  { id: 'd4', p: 'cos', title: 'Watch pattern and event storage per Kubeflow kind', desc: 'Details of the watch pattern and all the triggered actions when an event occurs for each kind.', kinds: ['Sequence', 'Sequence', 'Sequence', 'Entity–relationship'], when: '10/10/2026' },
  { id: 'd5', p: 'cos', title: 'Usage collector: how it runs and how it is designed', desc: 'I want to understand how the application works and how the app has been designed.', kinds: ['Sequence', 'Sequence', 'Sequence', 'Class', 'Class'], when: '10/10/2026' },
  { id: 'd6', p: 'sai', title: 'Schedule data flow', desc: 'How the schedule and the milestones travel from the editor to the store.', kinds: ['Flow'], when: '4 days ago' },
  { id: 'd7', p: 'rvr', title: 'Step claim and acceptance', desc: 'A session claims a step, a person accepts it, and the plan adherence is measured.', kinds: ['Sequence'], when: 'today', task: 'change-task-label', walk: true }
]
R.sequence = {
  actors: ['Collector', 'Kubernetes API', 'KFP API', 'Store'],
  msgs: [
    ['Collector', 'Kubernetes API', 'Open a watch on the Kubeflow kinds', false],
    ['Kubernetes API', 'Collector', 'Event: run created', true],
    ['Collector', 'KFP API', 'Get the run and its pipeline', false],
    ['KFP API', 'Collector', 'Run detail', true],
    ['Collector', 'Store', 'Write the usage row', false],
    ['Store', 'Collector', 'Row id', true],
    ['Kubernetes API', 'Collector', 'Event: run finished', true],
    ['Collector', 'Store', 'Close the usage row with the duration', false]
  ]
}

R.advisories = [
  { id: 'GHSA-q7v9-x93c-xqv3', cve: 'CVE-2026-115909', sev: 'High', pub: '14 days ago', p: 'pfa', proj: 'Psifatture', title: 'zip: Unbounded Symlink-Target Handling Can Abort Or Exhaust Memory During Extraction', pkg: 'zip', from: '2.4.2', to: '9.0.0', range: '>= 1.3.0, < 9.0.0' },
  { id: 'GHSA-q7v9-x93c-xqv3', cve: 'CVE-2026-115909', sev: 'High', pub: '10 days ago', p: 'rvr', proj: 'Rekall vR', title: 'zip: Unbounded Symlink-Target Handling Can Abort Or Exhaust Memory During Extraction', pkg: 'zip', from: '2.4.2', to: '9.0.0', range: '>= 1.3.0, < 9.0.0' },
  { id: 'GHSA-m665-v7pm-7grj', cve: 'CVE-2026-115905', sev: 'Critical', pub: '15 days ago', p: 'pfa', proj: 'Psifatture', title: 'zip: Critical Parser Differential & Security Control Bypass via Dual-Header Inconsistencies (Zip Cloaking) Leading to Arbitrary File Overwrite & RCE', pkg: 'zip', from: '2.4.2', to: '9.0.5', range: '<= 9.0.4' }
]
R.secProjects = [['Rekall vR', 'VFORGE', 19], ['Psifatture', 'VFORGE', 18], ['COSMIC', 'ESA', 0], ['HyperCV', 'VFORGE', 0], ['Lightcraft', 'Artcraft', 0], ['Malwarebytes', 'Acme', 0], ['MLX Studio', 'VFORGE', 0], ['OSX Tools', 'VFORGE', 0], ['Penna', 'VFORGE', 0], ['SAIGO', 'ESA', 0], ['STVW', 'ESA', 0]]

R.connections = [
  { id: 'c1', kind: 'Kubernetes', name: 'c3h', host: 'keystone-context · 10.33.244.3', shared: '1 company · 1 project' },
  { id: 'c2', kind: 'Kubernetes', name: 'kmaster12 aistranova', host: 'kubernetes-admin@kubernetes · 10.36.97.126:6443', shared: '1 company · 0 projects' },
  { id: 'c3', kind: 'Kubernetes', name: 'kmaster14', host: 'kubernetes-admin@kubernetes · 10.36.97.201:6443', shared: '1 company · 1 project' },
  { id: 'c4', kind: 'Kubernetes', name: 'kmaster2', host: 'kubernetes-admin@kubernetes · 10.36.97.50:6443', shared: '1 company · 0 projects' },
  { id: 'c5', kind: 'GitLab', name: 'codev', host: 'gitlab.space-codev.org', shared: '1 company · 1 project' },
  { id: 'c6', kind: 'GitLab', name: 'gitlab.esa.int', host: 'gitlab.esa.int', shared: '1 company · 2 projects' },
  { id: 'c7', kind: 'GitLab', name: 'gitlabesaadm', host: 'gitlab.esa.int', shared: '1 company · 1 project' },
  { id: 'c8', kind: 'GitHub', name: 'Gitub', host: 'github.com/ValerioMC', shared: '1 company · 6 projects' },
  { id: 'c9', kind: 'Jira', name: 'ESA Jira', host: 'jira.esa.int', shared: '1 company · 2 projects' }
]

R.claudeFiles = {
  Memories: [
    { g: 'Everywhere', path: 'Memories that apply to every project', files: [['lean-visual-iteration.md', '---\nname: lean-visual-iteration\ndescription: design work: few render rounds, no long polish on an unvalidated concept\n---\n\nKeep render rounds few. Do not polish a concept the user has not validated.'], ['mermaid-for-sequences.md', '---\nname: mermaid-for-sequences\ndescription: wrapups and notes draw flows as a mermaid sequenceDiagram\n---\n\nDraw a flow as a mermaid sequenceDiagram. Parse the block first.']] },
    { g: 'COSMIC', path: '/Users/demo/Projects/ESA', files: [['kubeflow-kinds.md', '---\nname: kubeflow-kinds\n---\n\nThe watch covers Run, Pipeline and Experiment.']] },
    { g: 'STVW', path: '/Users/demo/Projects/ESA', files: [['metricbridge-reference.md', '---\nname: metricbridge-reference\n---\n\nReference pattern is mcps/weaviate-mcp.']] }
  ],
  Skills: [{ g: 'Global', path: '~/.claude/skills', files: [['impeccable', '# impeccable\n\nDesign skill for frontends.'], ['rk', '# rk\n\nLoad a Rekall working context by anchor.']] }],
  Commands: [{ g: 'Global', path: '~/.claude/commands', files: [['review.md', 'Review the current diff.']] }],
  Agents: [{ g: 'Global', path: '~/.claude/agents', files: [['explore.md', 'Read-only search agent.']] }]
}
R.tags = [{ n: 'Simulator', c: '#9f78e6' }, { n: 'Framework (core)', c: '#46c9b0' }, { n: 'Diagnostic', c: '#f0955a' }, { n: 'Frontend', c: '#4ea1e0' }, { n: 'Docs', c: '#7fd66a' }]
