-- Fills an empty Rekall database with a large, varied catalog: 10 companies, 40 projects, 640
-- tasks (40 epics), step checklists and flows, notes, wrapups, time and token usage, checks,
-- connections, vulnerabilities, deployment graphs and diagrams.
--
-- Run it against a throwaway database that the server has already migrated (start it once with
-- `make run-demo`, stop it, then `make load-sql`). It refuses to touch a database that already
-- holds a company, so a real catalog cannot be polluted by accident.
--
--   sqlite3 data/demo/rekall.db < scripts/seed-demo-data.sql
--
-- Ids are deterministic UUIDs (`<namespace>-0000-4000-8000-<n>`), so a rerun on a fresh database
-- yields the same rows. Timestamps are relative to the moment the script runs.
-- Connections carry no credential (`secret_set_at` stays empty): the secrets live outside the
-- database, so a seeded connection reads as "credential not set".

.bail on
PRAGMA foreign_keys = ON;
BEGIN;
PRAGMA defer_foreign_keys = ON;

CREATE TEMP TABLE guard (database_was_empty INTEGER NOT NULL CHECK (database_was_empty = 1));
INSERT INTO guard SELECT COUNT(*) = 0 FROM company;

CREATE TEMP TABLE seq (n INTEGER PRIMARY KEY);
WITH RECURSIVE counter(n) AS (SELECT 1 UNION ALL SELECT n + 1 FROM counter WHERE n < 800)
INSERT INTO seq SELECT n FROM counter;

------------------------------------------------------------------------------------------------
-- Vocabulary
------------------------------------------------------------------------------------------------

CREATE TEMP TABLE tcompany (n INTEGER PRIMARY KEY, name TEXT, description TEXT);
INSERT INTO tcompany VALUES
 (1, 'Acme Robotics', 'Industrial automation and warehouse robotics.'),
 (2, 'Globex Analytics', 'A data platform for retail demand forecasting.'),
 (3, 'Initech Labs', 'Developer tooling and internal platform engineering.'),
 (4, 'Umbrella Systems', 'Healthcare records and compliance software.'),
 (5, 'Soylent Digital', 'A consumer nutrition and subscription app.'),
 (6, 'Stark Dynamics', 'Energy management and IoT devices.'),
 (7, 'Hooli Cloud', 'Managed hosting, edge delivery and developer APIs.'),
 (8, 'Wayne Logistics', 'Fleet routing, freight tracking and customs paperwork.'),
 (9, 'Wonka Food Tech', 'Production-line scheduling for confectionery plants.'),
 (10, 'Pied Piper Media', 'Streaming, compression and creator tooling.');

CREATE TEMP TABLE tproject_template (n INTEGER PRIMARY KEY, label TEXT, title TEXT, icon TEXT, description TEXT);
INSERT INTO tproject_template VALUES
 (1, 'platform-api', 'Platform API', 'cube', 'The public REST and webhook surface every other product talks to.'),
 (2, 'web-console', 'Web Console', 'globe', 'The browser console customers and operators use every day.'),
 (3, 'mobile-app', 'Mobile App', 'rocket', 'iOS and Android clients sharing one release train.'),
 (4, 'data-pipeline', 'Data Pipeline', 'database', 'Ingestion, validation and the nightly warehouse load.'),
 (5, 'auth-service', 'Auth Service', 'key', 'Sign-in, sessions, tokens and permission checks.'),
 (6, 'billing', 'Billing and Invoicing', 'briefcase', 'Metering, invoices, dunning and tax.'),
 (7, 'design-system', 'Design System', 'palette', 'Tokens and components shared by every front end.'),
 (8, 'observability', 'Observability Stack', 'chart', 'Metrics, traces, logs and the alerts that wake people up.'),
 (9, 'infra', 'Infrastructure', 'cloud', 'Clusters, networking, secrets and the deploy pipeline.'),
 (10, 'ml-models', 'Forecasting Models', 'beaker', 'Training, evaluation and serving of the demand models.'),
 (11, 'docs-site', 'Documentation Site', 'book', 'Public docs, API reference and the changelog.'),
 (12, 'cli-tooling', 'CLI Tooling', 'terminal', 'The command line used by developers and CI.');

CREATE TEMP TABLE ttask_template (n INTEGER PRIMARY KEY, title TEXT);
INSERT INTO ttask_template VALUES
 (1, 'Set up CI pipeline'), (2, 'Fix login redirect bug'), (3, 'Write onboarding docs'),
 (4, 'Refactor auth module'), (5, 'Add rate limiting'), (6, 'Investigate memory leak'),
 (7, 'Migrate database schema'), (8, 'Improve error messages'), (9, 'Add dark mode toggle'),
 (10, 'Write integration tests'), (11, 'Optimize query performance'), (12, 'Update dependencies'),
 (13, 'Design API contract'), (14, 'Implement webhook retries'), (15, 'Add pagination to list endpoint'),
 (16, 'Fix flaky test'), (17, 'Set up monitoring dashboard'), (18, 'Draft RFC for caching layer'),
 (19, 'Clean up dead code'), (20, 'Add feature flag support'), (21, 'Improve build times'),
 (22, 'Write incident postmortem'), (23, 'Add CSV export'), (24, 'Implement search autocomplete'),
 (25, 'Fix timezone bug'), (26, 'Add rate limit headers'), (27, 'Upgrade framework version'),
 (28, 'Harden input validation'), (29, 'Add audit logging'), (30, 'Simplify config loading'),
 (31, 'Reduce bundle size'), (32, 'Add retry with backoff'), (33, 'Fix duplicate query on list'),
 (34, 'Write load test scenarios'), (35, 'Add health check endpoint'), (36, 'Deprecate legacy endpoint'),
 (37, 'Add keyboard shortcuts'), (38, 'Improve empty states'), (39, 'Fix race condition on save'),
 (40, 'Add structured logging');

CREATE TEMP TABLE tepic_template (n INTEGER PRIMARY KEY, title TEXT);
INSERT INTO tepic_template VALUES
 (1, 'Reliability'), (2, 'Performance'), (3, 'Security hardening'), (4, 'Developer experience'),
 (5, 'Accessibility'), (6, 'Migration to the new schema'), (7, 'Observability'), (8, 'Customer requests'),
 (9, 'Cost reduction'), (10, 'Onboarding'), (11, 'Compliance'), (12, 'Release 2.0');

CREATE TEMP TABLE tstep_template (n INTEGER PRIMARY KEY, title TEXT, body TEXT);
INSERT INTO tstep_template VALUES
 (1, 'Reproduce and pin down the behaviour', 'Write down the exact steps and the expected result before touching code.'),
 (2, 'Write the failing test', 'A test that fails for the reason in the report, and only that reason.'),
 (3, 'Implement the change', 'Smallest change that makes the test pass; no drive-by refactors.'),
 (4, 'Cover the edge cases', 'Empty input, very long input, concurrent calls, a missing permission.'),
 (5, 'Update the docs and changelog', 'Anything a user or an operator would notice goes in the changelog.'),
 (6, 'Run the full suite', 'Unit, integration and lint, all green before the merge request is opened.'),
 (7, 'Open the merge request', 'Link the task, summarise the risk, name the reviewer.'),
 (8, 'Address review feedback', 'Answer every thread; resolve only the ones that were fixed.');

CREATE TEMP TABLE ttag (n INTEGER PRIMARY KEY, name TEXT, icon TEXT, color TEXT);
INSERT INTO ttag VALUES
 (1, 'urgent', 'flame', 'crimson'), (2, 'tech-debt', 'gem', 'tangerine'), (3, 'customer', 'heart', 'magenta'),
 (4, 'security', 'shield', 'violet'), (5, 'research', 'compass', 'cerulean'), (6, 'quick-win', 'bolt', 'citrine'),
 (7, 'ops', 'flag', 'teal'), (8, 'design', 'star', 'moss');

------------------------------------------------------------------------------------------------
-- Derived plan: which project, task and step exists and in what state
------------------------------------------------------------------------------------------------

CREATE TEMP TABLE tproject (pn INTEGER PRIMARY KEY, cn INTEGER, tmpl INTEGER, status TEXT);
INSERT INTO tproject
SELECT p.n,
       (p.n - 1) / 4 + 1,
       (((p.n - 1) / 4 + 1) * 5 + ((p.n - 1) % 4 + 1) * 3) % 12 + 1,
       CASE WHEN p.n % 19 = 0 THEN 'ARCHIVED'
            WHEN p.n % 13 = 0 THEN 'DONE'
            WHEN p.n % 11 = 0 THEN 'PAUSED'
            ELSE 'ACTIVE' END
FROM seq p WHERE p.n <= 40;

CREATE TEMP TABLE ttask (
    tn INTEGER PRIMARY KEY, pn INTEGER, j INTEGER, kind TEXT, tmpl INTEGER, status TEXT,
    review_state TEXT, epic_tn INTEGER, nsteps INTEGER, cur INTEGER, is_flow INTEGER,
    created_min INTEGER, updated_min INTEGER, tag_n INTEGER
);
INSERT INTO ttask (tn, pn, j, kind, tmpl, status)
SELECT (p.pn - 1) * 16 + s.n, p.pn, s.n,
       CASE WHEN s.n = 1 THEN 'EPIC' ELSE 'TASK' END,
       (p.pn * 7 + s.n * 3) % 40 + 1,
       CASE WHEN p.status IN ('DONE', 'ARCHIVED') THEN 'DONE'
            WHEN s.n = 1 THEN 'IN_PROGRESS'
            WHEN (p.pn * 3 + s.n * 7) % 20 < 6 THEN 'DONE'
            WHEN (p.pn * 3 + s.n * 7) % 20 < 12 THEN 'IN_PROGRESS'
            WHEN (p.pn * 3 + s.n * 7) % 20 < 16 THEN 'TODO'
            WHEN (p.pn * 3 + s.n * 7) % 20 < 18 THEN 'BACKLOG'
            ELSE 'BLOCKED' END
FROM tproject p JOIN seq s ON s.n <= 16;

UPDATE ttask SET
    epic_tn = CASE WHEN kind = 'TASK' AND j BETWEEN 2 AND 6 THEN (pn - 1) * 16 + 1 END,
    nsteps = CASE WHEN kind = 'TASK' THEN 3 + tn % 4 ELSE 0 END,
    review_state = CASE WHEN kind = 'EPIC' THEN 'OPEN'
                        WHEN status = 'DONE' THEN 'DONE'
                        WHEN status = 'IN_PROGRESS' AND tn % 3 = 0 THEN 'CLAIMED'
                        ELSE 'OPEN' END,
    created_min = (90 - pn % 30 - j * 3) * 1440 + (pn * 37) % 600,
    tag_n = CASE WHEN tn % 3 = 0 THEN tn % 8 + 1 END;
UPDATE ttask SET
    cur = CASE WHEN nsteps > 0 THEN 1 + tn % nsteps END,
    is_flow = CASE WHEN kind = 'TASK' AND nsteps >= 4 AND tn % 5 = 0 THEN 1 ELSE 0 END,
    updated_min = MAX(60, created_min - (tn % 25 + 1) * 1440);

CREATE TEMP TABLE tstep (sn INTEGER PRIMARY KEY, tn INTEGER, k INTEGER, nsteps INTEGER, template INTEGER, state TEXT);
INSERT INTO tstep
SELECT t.tn * 10 + k.n, t.tn, k.n, t.nsteps, (k.n - 1) * 8 / t.nsteps + 1,
       CASE WHEN t.status = 'DONE' THEN 'DONE'
            WHEN t.status = 'BACKLOG' THEN 'DRAFT'
            WHEN t.status = 'TODO' THEN 'OPEN'
            WHEN t.review_state = 'CLAIMED' THEN CASE WHEN k.n < t.nsteps THEN 'DONE' ELSE 'CLAIMED' END
            WHEN k.n < t.cur THEN 'DONE'
            WHEN k.n = t.cur THEN 'RUNNING'
            ELSE 'OPEN' END
FROM ttask t JOIN seq k ON k.n <= t.nsteps WHERE t.kind = 'TASK';

------------------------------------------------------------------------------------------------
-- Catalog
------------------------------------------------------------------------------------------------

INSERT INTO company (id, name, description, created_at, updated_at)
SELECT printf('%08x-0000-4000-8000-%012x', 1, n), name, description,
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || ((200 - n * 5) * 1440) || ' minutes'),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (n * 1440) || ' minutes')
FROM tcompany;

INSERT INTO tag (id, name, icon, color, created_at, updated_at)
SELECT printf('%08x-0000-4000-8000-%012x', 8, n), name, icon, color,
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-150 days'),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-150 days')
FROM ttag;

INSERT INTO project (id, label, status, description, created_at, updated_at, company_id, title,
                     blueprint_markdown, repo_folder, auto_commit, icon)
SELECT printf('%08x-0000-4000-8000-%012x', 2, p.pn), m.label, p.status, m.description,
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || ((120 - p.pn % 30) * 1440) || ' minutes'),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (p.pn % 9 * 1440 + 30) || ' minutes'),
       printf('%08x-0000-4000-8000-%012x', 1, p.cn), m.title,
       printf('# %s' || char(10) || char(10) || '## Shape' || char(10) || 'Three deployables share one repository: the **api**, the **web** front end and a **worker** that drains the queue.' || char(10) || char(10) || '## Conventions' || char(10) || '- Trunk-based, short-lived branches.' || char(10) || '- Every change ships behind a flag until the owner signs it off.' || char(10) || '- Migrations are forward-only.' || char(10) || char(10) || '| Environment | Cluster | Release cadence |' || char(10) || '|---|---|---|' || char(10) || '| staging | shared | on merge |' || char(10) || '| production | dedicated | weekly |',
              m.title),
       NULL, 0, m.icon
FROM tproject p JOIN tproject_template m ON m.n = p.tmpl;

INSERT INTO task (id, label, status, description, project_id, created_at, updated_at, title, review_state,
                  claimed_at, accepted_at, tag_id, graph_laid_at, release_skipped, kind, epic_id, quick,
                  auto_accepted, worktree_folder)
SELECT printf('%08x-0000-4000-8000-%012x', 3, t.tn),
       CASE WHEN t.kind = 'EPIC' THEN 'epic-' || lower(replace(e.title, ' ', '-')) ELSE lower(replace(k.title, ' ', '-')) END,
       t.status,
       CASE WHEN t.tn % 7 = 0 THEN NULL
            WHEN t.tn % 4 = 0 THEN printf('## Approach' || char(10) || 'Start from the failing path, add a test, then change the code. Keep the diff reviewable.' || char(10) || char(10) || '```sql' || char(10) || 'SELECT id, status FROM orders WHERE updated_at > now() - interval ''1 day'';' || char(10) || '```')
            WHEN t.tn % 4 = 1 THEN printf('## Checklist' || char(10) || '- [x] Reproduce' || char(10) || '- [ ] Fix' || char(10) || '- [ ] Add a regression test' || char(10) || '- [ ] Tell support')
            WHEN t.tn % 4 = 2 THEN printf('## Constraints' || char(10) || char(10) || '| Limit | Value |' || char(10) || '|---|---|' || char(10) || '| p95 latency | 300 ms |' || char(10) || '| error budget | 0.1%% |' || char(10) || char(10) || '> Do not change the public response shape.')
            ELSE printf('## Pointers' || char(10) || '- Start in the service that owns the record.' || char(10) || '- See the incident notes attached to this task.' || char(10) || '- [Runbook](https://example.com/runbook)') END,
       printf('%08x-0000-4000-8000-%012x', 2, t.pn),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || t.created_min || ' minutes'),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || t.updated_min || ' minutes'),
       CASE WHEN t.kind = 'EPIC' THEN e.title || ' — ' || m.title ELSE k.title END,
       t.review_state,
       CASE WHEN t.review_state IN ('CLAIMED', 'DONE') THEN strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (t.updated_min + 120) || ' minutes') END,
       CASE WHEN t.review_state = 'DONE' THEN strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || t.updated_min || ' minutes') END,
       CASE WHEN t.tag_n IS NOT NULL THEN printf('%08x-0000-4000-8000-%012x', 8, t.tag_n) END,
       NULL, 0, t.kind,
       CASE WHEN t.epic_tn IS NOT NULL THEN printf('%08x-0000-4000-8000-%012x', 3, t.epic_tn) END,
       0, CASE WHEN t.status = 'DONE' AND t.tn % 8 = 0 THEN 1 ELSE 0 END, NULL
FROM ttask t
JOIN tproject p ON p.pn = t.pn
JOIN tproject_template m ON m.n = p.tmpl
JOIN ttask_template k ON k.n = t.tmpl
JOIN tepic_template e ON e.n = (t.pn - 1) % 12 + 1;

INSERT INTO project_section (id, project_id, title, body_markdown, position, created_at, updated_at)
SELECT printf('%08x-0000-4000-8000-%012x', 12, p.pn * 10 + k.n),
       printf('%08x-0000-4000-8000-%012x', 2, p.pn),
       CASE k.n WHEN 1 THEN 'Deployment' ELSE 'Runbook' END,
       CASE k.n WHEN 1 THEN '## Deploying' || char(10) || 'Merge to trunk; the pipeline builds, deploys to staging and waits for a person before production.' || char(10) || char(10) || '1. Check the dashboards.' || char(10) || '2. Approve the production stage.' || char(10) || '3. Watch the error rate for ten minutes.'
                ELSE '## When it pages' || char(10) || '- Look at the last deploy first.' || char(10) || '- Roll back before debugging.' || char(10) || '- Write the postmortem within two days.' END,
       k.n - 1,
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-60 days'),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-10 days')
FROM tproject p JOIN seq k ON k.n <= 2;

INSERT INTO project_component (id, project_id, name, folder, description, position, created_at, updated_at)
SELECT printf('%08x-0000-4000-8000-%012x', 13, p.pn * 10 + k.n),
       printf('%08x-0000-4000-8000-%012x', 2, p.pn),
       CASE k.n WHEN 1 THEN 'api' WHEN 2 THEN 'web' ELSE 'worker' END,
       CASE k.n WHEN 1 THEN 'services/api' WHEN 2 THEN 'apps/web' ELSE 'services/worker' END,
       CASE k.n WHEN 1 THEN 'HTTP handlers, validation and the service layer.'
                WHEN 2 THEN 'The browser front end and its view models.'
                ELSE 'Queue consumers and scheduled jobs.' END,
       k.n - 1,
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-60 days'),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-10 days')
FROM tproject p JOIN seq k ON k.n <= 3;

------------------------------------------------------------------------------------------------
-- Steps, flows and dependencies between tasks
------------------------------------------------------------------------------------------------

INSERT INTO task_step (id, task_id, title, body_markdown, done_at, position, created_at, updated_at, state,
                       running_at, claimed_at, passes_json, flow_x, flow_y, max_visits, connection_id, starts_session)
SELECT printf('%08x-0000-4000-8000-%012x', 4, s.sn),
       printf('%08x-0000-4000-8000-%012x', 3, s.tn),
       m.title,
       CASE WHEN s.state = 'DRAFT' THEN NULL ELSE m.body END,
       CASE WHEN s.state = 'DONE' THEN strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (t.updated_min + (s.nsteps - s.k) * 90) || ' minutes') END,
       s.k - 1,
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || t.created_min || ' minutes'),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || t.updated_min || ' minutes'),
       s.state,
       CASE WHEN s.state IN ('RUNNING', 'CLAIMED', 'DONE') THEN strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (t.updated_min + (s.nsteps - s.k) * 90 + 60) || ' minutes') END,
       CASE WHEN s.state IN ('CLAIMED', 'DONE') THEN strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (t.updated_min + (s.nsteps - s.k) * 90 + 20) || ' minutes') END,
       CASE WHEN s.state = 'DONE' AND s.sn % 9 = 0
            THEN json_array(json_object('detailMarkdown', m.body, 'sentBackAt', strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (t.updated_min + 400) || ' minutes')))
            ELSE '[]' END,
       NULL, NULL,
       CASE WHEN t.is_flow = 1 AND s.k = 2 THEN 3 END,
       NULL, 0
FROM tstep s
JOIN ttask t ON t.tn = s.tn
JOIN tstep_template m ON m.n = s.template;

INSERT INTO step_link (id, task_id, from_step_id, to_step_id, outcome, position, created_at, updated_at)
SELECT printf('%08x-0000-4000-8000-%012x', 21, s.sn * 10 + 1),
       printf('%08x-0000-4000-8000-%012x', 3, s.tn),
       printf('%08x-0000-4000-8000-%012x', 4, s.sn),
       CASE WHEN s.k < s.nsteps THEN printf('%08x-0000-4000-8000-%012x', 4, s.sn + 1) END,
       CASE WHEN s.k < s.nsteps THEN 'next' ELSE 'done' END,
       0,
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || t.created_min || ' minutes'),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || t.created_min || ' minutes')
FROM tstep s JOIN ttask t ON t.tn = s.tn WHERE t.is_flow = 1;

INSERT INTO step_link (id, task_id, from_step_id, to_step_id, outcome, position, created_at, updated_at)
SELECT printf('%08x-0000-4000-8000-%012x', 21, s.sn * 10 + 2),
       printf('%08x-0000-4000-8000-%012x', 3, s.tn),
       printf('%08x-0000-4000-8000-%012x', 4, s.sn),
       printf('%08x-0000-4000-8000-%012x', 4, s.sn - 1),
       'fail', 1,
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || t.created_min || ' minutes'),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || t.created_min || ' minutes')
FROM tstep s JOIN ttask t ON t.tn = s.tn WHERE t.is_flow = 1 AND s.k = 3;

INSERT INTO step_visit (id, task_id, step_id, ordinal, entered_at, left_at, outcome, report)
SELECT printf('%08x-0000-4000-8000-%012x', 22, s.sn),
       printf('%08x-0000-4000-8000-%012x', 3, s.tn),
       printf('%08x-0000-4000-8000-%012x', 4, s.sn),
       s.k,
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (t.updated_min + (s.nsteps - s.k) * 90 + 60) || ' minutes'),
       CASE WHEN s.state = 'DONE' THEN strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (t.updated_min + (s.nsteps - s.k) * 90 + 20) || ' minutes') END,
       CASE WHEN s.state = 'DONE' THEN CASE WHEN s.k < s.nsteps THEN 'next' ELSE 'done' END END,
       NULL
FROM tstep s JOIN ttask t ON t.tn = s.tn
WHERE t.is_flow = 1 AND s.state IN ('DONE', 'RUNNING', 'CLAIMED');

INSERT INTO task_dependency (id, task_id, depends_on_task_id, created_at)
SELECT printf('%08x-0000-4000-8000-%012x', 11, t.tn),
       printf('%08x-0000-4000-8000-%012x', 3, t.tn),
       printf('%08x-0000-4000-8000-%012x', 3, t.tn - CASE t.j WHEN 8 THEN 5 ELSE 4 END),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || t.created_min || ' minutes')
FROM ttask t WHERE t.j IN (8, 12);

------------------------------------------------------------------------------------------------
-- Notes
------------------------------------------------------------------------------------------------

INSERT INTO document (id, title, kind, body_markdown, source_path, created_at, updated_at, context_mode,
                      scope_company_id, scope_project_id)
SELECT printf('%08x-0000-4000-8000-%012x', 31, p.pn * 10 + d.n),
       m.title || ' — ' || CASE d.n WHEN 1 THEN 'Operating context' WHEN 2 THEN 'Design notes' ELSE 'Architecture overview' END,
       CASE d.n WHEN 1 THEN 'context' WHEN 2 THEN 'notes' ELSE 'architecture' END,
       CASE d.n
         WHEN 1 THEN '## Operating context' || char(10) || 'Owned by one small team. Production is a dedicated cluster; staging is shared.' || char(10) || char(10) || '- On call: weekly rotation.' || char(10) || '- Change freeze: the last Friday of the month.'
         WHEN 2 THEN '## Design notes' || char(10) || 'Talked this through with the team; the approach below is what we settled on.' || char(10) || char(10) || '- Keep the handler thin.' || char(10) || '- One writer per table.' || char(10) || '- Emit an event after commit, never inside it.'
         ELSE '## Architecture' || char(10) || char(10) || '```mermaid' || char(10) || 'flowchart LR' || char(10) || '  Client --> Api' || char(10) || '  Api --> Db[(Database)]' || char(10) || '  Api --> Queue' || char(10) || '  Queue --> Worker' || char(10) || '```' END,
       NULL,
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || ((80 - p.pn % 30) * 1440) || ' minutes'),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || ((p.pn % 20 + d.n) * 1440) || ' minutes'),
       CASE WHEN d.n = 3 THEN 'REFERENCE' ELSE 'FULL' END,
       NULL,
       CASE WHEN d.n = 1 THEN printf('%08x-0000-4000-8000-%012x', 2, p.pn) END
FROM tproject p JOIN tproject_template m ON m.n = p.tmpl JOIN seq d ON d.n <= 3;

INSERT INTO document (id, title, kind, body_markdown, source_path, created_at, updated_at, context_mode,
                      scope_company_id, scope_project_id)
SELECT printf('%08x-0000-4000-8000-%012x', 31, 1000 + c.n),
       c.name || ' — House rules', 'context',
       '## House rules' || char(10) || '- Conventional commits.' || char(10) || '- Two reviewers on anything that touches money or permissions.' || char(10) || '- Write the wrapup before you close the session.',
       NULL,
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-120 days'),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (c.n * 3) || ' days'),
       'FULL',
       printf('%08x-0000-4000-8000-%012x', 1, c.n), NULL
FROM tcompany c;

INSERT INTO document_task (document_id, task_id, position)
SELECT printf('%08x-0000-4000-8000-%012x', 31, t.pn * 10 + CASE WHEN t.j BETWEEN 2 AND 5 THEN 2 ELSE 3 END),
       printf('%08x-0000-4000-8000-%012x', 3, t.tn), 0
FROM ttask t WHERE t.kind = 'TASK' AND t.j BETWEEN 2 AND 8;

------------------------------------------------------------------------------------------------
-- Work done: wrapups, commits, time, tokens, checks, deviations, feedback
------------------------------------------------------------------------------------------------

INSERT INTO wrapup (id, task_id, body_markdown, written_by, created_at, updated_at)
SELECT printf('%08x-0000-4000-8000-%012x', 5, t.tn),
       printf('%08x-0000-4000-8000-%012x', 3, t.tn),
       '## What it is now' || char(10) || k.title || ' is in place. The handler validates its input, the service owns the rule and the repository takes the transaction it is given.' || char(10) || char(10) || '## Where to look' || char(10) || '- `services/api` for the handler' || char(10) || '- the matching test file for the behaviour that was asked for',
       CASE WHEN t.tn % 3 = 0 THEN 'HAND' ELSE 'CLAUDE' END,
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (t.updated_min + 40) || ' minutes'),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || t.updated_min || ' minutes')
FROM ttask t JOIN ttask_template k ON k.n = t.tmpl
WHERE t.kind = 'TASK' AND (t.status = 'DONE' OR (t.status = 'IN_PROGRESS' AND t.tn % 2 = 0));

INSERT INTO wrapup_version (id, task_id, number, body_markdown, written_by, written_at, commit_hash)
SELECT printf('%08x-0000-4000-8000-%012x', 6, t.tn * 10 + v.n),
       w.task_id, v.n, w.body_markdown, w.written_by,
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (t.updated_min + 40 + (2 - v.n) * 600) || ' minutes'),
       NULL
FROM wrapup w
JOIN ttask t ON printf('%08x-0000-4000-8000-%012x', 3, t.tn) = w.task_id
JOIN seq v ON v.n <= 1 + (t.tn % 4 = 0);

INSERT INTO commit_reference (id, task_id, step_id, commit_hash, comment, created_at, diff, in_context)
SELECT printf('%08x-0000-4000-8000-%012x', 23, t.tn * 10 + c.n),
       printf('%08x-0000-4000-8000-%012x', 3, t.tn),
       NULL,
       substr(lower(hex(randomblob(20))), 1, 40),
       CASE c.n WHEN 1 THEN 'feat: ' ELSE 'test: ' END || lower(k.title),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (t.updated_min + 200 - c.n * 60) || ' minutes'),
       NULL, 0
FROM ttask t JOIN ttask_template k ON k.n = t.tmpl JOIN seq c ON c.n <= 2
WHERE t.kind = 'TASK' AND t.status = 'DONE';

INSERT INTO time_entry (id, task_id, started_at, stopped_at, created_at, updated_at)
SELECT printf('%08x-0000-4000-8000-%012x', 24, t.tn * 10 + e.n),
       printf('%08x-0000-4000-8000-%012x', 3, t.tn),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || w.start_min || ' minutes'),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (w.start_min - 20 - (t.tn + e.n * 13) % 130) || ' minutes'),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || w.start_min || ' minutes'),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || w.start_min || ' minutes')
FROM ttask t
JOIN seq e ON e.n <= CASE WHEN t.status = 'DONE' THEN 3 ELSE 2 END
JOIN (SELECT t2.tn AS tn, e2.n AS en, ((t2.tn * 5 + e2.n * 11) % 28) * 1440 + 240 + (t2.tn * 37 + e2.n * 53) % 400 AS start_min
      FROM ttask t2 JOIN seq e2 ON e2.n <= 3) w ON w.tn = t.tn AND w.en = e.n
WHERE t.kind = 'TASK' AND t.status IN ('IN_PROGRESS', 'DONE', 'BLOCKED');

UPDATE time_entry
SET started_at = strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-25 minutes'),
    stopped_at = NULL,
    created_at = strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-25 minutes'),
    updated_at = strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-25 minutes')
WHERE id = (SELECT printf('%08x-0000-4000-8000-%012x', 24, MIN(tn) * 10 + 1) FROM ttask WHERE kind = 'TASK' AND status = 'IN_PROGRESS');

INSERT INTO token_usage (id, session_id, task_id, step_id, input_tokens, output_tokens, cache_creation_tokens,
                         cache_read_tokens, recorded_at, spent_on)
SELECT printf('%08x-0000-4000-8000-%012x', 25, t.tn * 10 + r.n),
       printf('seed-session-%d-%d', t.tn, r.n),
       printf('%08x-0000-4000-8000-%012x', 3, t.tn),
       NULL,
       3000 + (t.tn * 977) % 40000,
       1500 + (t.tn * 613) % 20000,
       20000 + (t.tn * 3571) % 90000,
       100000 + (t.tn * 7919) % 900000,
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (((t.tn * 3 + r.n * 7) % 28) * 1440 + t.tn % 600) || ' minutes'),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (((t.tn * 3 + r.n * 7) % 28) * 1440 + t.tn % 600) || ' minutes')
FROM ttask t JOIN seq r ON r.n <= 2
WHERE t.kind = 'TASK' AND t.status IN ('IN_PROGRESS', 'DONE', 'BLOCKED');

INSERT INTO done_check (id, project_id, step_id, label, command, folder, timeout_seconds, position, created_at, updated_at)
SELECT printf('%08x-0000-4000-8000-%012x', 9, p.pn * 10 + c.n),
       printf('%08x-0000-4000-8000-%012x', 2, p.pn),
       NULL,
       CASE c.n WHEN 1 THEN 'Unit tests' ELSE 'Lint' END,
       CASE c.n WHEN 1 THEN 'make test' ELSE 'make lint' END,
       NULL,
       CASE c.n WHEN 1 THEN 300 ELSE 120 END,
       c.n - 1,
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-60 days'),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-60 days')
FROM tproject p JOIN seq c ON c.n <= 2;

INSERT INTO done_check_result (id, check_id, task_id, step_id, passed, exit_code, output_tail, ran_at, duration_ms)
SELECT printf('%08x-0000-4000-8000-%012x', 10, t.tn * 10 + c.n),
       printf('%08x-0000-4000-8000-%012x', 9, t.pn * 10 + c.n),
       printf('%08x-0000-4000-8000-%012x', 3, t.tn),
       NULL,
       (t.tn + c.n) % 9 <> 0,
       CASE WHEN (t.tn + c.n) % 9 <> 0 THEN 0 ELSE 1 END,
       CASE WHEN (t.tn + c.n) % 9 <> 0 THEN 'test result: ok. 142 passed; 0 failed; 0 ignored'
            ELSE 'test result: FAILED. 141 passed; 1 failed' || char(10) || 'assertion failed: expected 200, got 500' END,
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (t.updated_min + 10) || ' minutes'),
       800 + (t.tn * 131) % 9000
FROM ttask t JOIN seq c ON c.n <= 2
WHERE t.kind = 'TASK' AND t.status = 'DONE';

INSERT INTO task_deviation (id, task_id, step_number, body_markdown, created_at, updated_at)
SELECT printf('%08x-0000-4000-8000-%012x', 26, t.tn),
       printf('%08x-0000-4000-8000-%012x', 3, t.tn),
       2,
       'The plan assumed one table; the data lives in two. Added a join and a test for the case where the second row is missing.',
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (t.updated_min + 90) || ' minutes'),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (t.updated_min + 90) || ' minutes')
FROM ttask t WHERE t.kind = 'TASK' AND t.status IN ('IN_PROGRESS', 'DONE') AND t.tn % 6 = 0;

INSERT INTO task_feedback (id, task_id, seq, kind, state, quoted_markdown, body_markdown, reply_markdown,
                           created_at, updated_at, applied_at, pass, taken_at)
SELECT printf('%08x-0000-4000-8000-%012x', 27, t.tn * 10 + f.n),
       printf('%08x-0000-4000-8000-%012x', 3, t.tn),
       f.n,
       CASE f.n WHEN 1 THEN 'PLAN' ELSE 'IMPLEMENTATION' END,
       CASE f.n WHEN 1 THEN 'APPLIED' ELSE 'PENDING' END,
       CASE f.n WHEN 1 THEN 'Run the full suite' ELSE NULL END,
       CASE f.n WHEN 1 THEN 'Split the suite so the slow integration tests run last.'
                ELSE 'The error message still says "something went wrong"; name the field.' END,
       CASE f.n WHEN 1 THEN 'Reordered the steps; integration tests now run after lint and unit tests.' END,
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (t.updated_min + 300 - f.n * 100) || ' minutes'),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (t.updated_min + 200 - f.n * 100) || ' minutes'),
       CASE f.n WHEN 1 THEN strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (t.updated_min + 150) || ' minutes') END,
       NULL, NULL
FROM ttask t JOIN seq f ON f.n <= 1 + (t.tn % 14 = 0)
WHERE t.kind = 'TASK' AND t.status IN ('IN_PROGRESS', 'DONE') AND t.tn % 7 = 0;

------------------------------------------------------------------------------------------------
-- Connections
------------------------------------------------------------------------------------------------

INSERT INTO external_connection (id, label, title, kind, base_url, kube_context, proxy_url, root_path, secret_ref,
                                 secret_set_at, created_at, updated_at, accept_self_signed)
SELECT printf('%08x-0000-4000-8000-%012x', 41, n), label, title, kind, base_url, kube_context, NULL, root_path,
       printf('seed-secret-%02d', n), NULL,
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-100 days'),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-5 days'),
       accept_self_signed
FROM (SELECT 1 AS n, 'gitlab-main' AS label, 'GitLab (company)' AS title, 'GITLAB' AS kind, 'https://gitlab.example.com' AS base_url, NULL AS kube_context, 'platform' AS root_path, 0 AS accept_self_signed
      UNION ALL SELECT 2, 'gitlab-selfhosted', 'GitLab (self-hosted)', 'GITLAB', 'https://git.internal.example.com', NULL, 'eng', 1
      UNION ALL SELECT 3, 'github-com', 'GitHub', 'GITHUB', 'https://api.github.com', NULL, 'example-org', 0
      UNION ALL SELECT 4, 'github-enterprise', 'GitHub Enterprise', 'GITHUB', 'https://github.example.com/api/v3', NULL, 'labs', 0
      UNION ALL SELECT 5, 'k8s-prod', 'Kubernetes production', 'KUBERNETES', NULL, 'prod-eu-west-1', NULL, 0
      UNION ALL SELECT 6, 'k8s-staging', 'Kubernetes staging', 'KUBERNETES', NULL, 'staging-eu-west-1', NULL, 0
      UNION ALL SELECT 7, 'k8s-dev', 'Kubernetes development', 'KUBERNETES', NULL, 'dev-local', NULL, 0
      UNION ALL SELECT 8, 'jira-cloud', 'Jira Cloud', 'JIRA', 'https://example.atlassian.net', NULL, NULL, 0
      UNION ALL SELECT 9, 'jira-server', 'Jira Data Center', 'JIRA', 'https://jira.internal.example.com', NULL, NULL, 1
      UNION ALL SELECT 10, 'grafana-obs', 'Grafana observability', 'GRAFANA', 'https://grafana.example.com', NULL, NULL, 0
      UNION ALL SELECT 11, 'grafana-edge', 'Grafana edge sites', 'GRAFANA', 'https://edge-grafana.example.com', NULL, NULL, 0
      UNION ALL SELECT 12, 'gitlab-legacy', 'GitLab (legacy, unshared)', 'GITLAB', 'https://old-git.example.com', NULL, 'archive', 1);

INSERT INTO connection_company (connection_id, company_id)
SELECT printf('%08x-0000-4000-8000-%012x', 41, CASE WHEN c.n % 2 = 0 THEN 1 ELSE 2 END), printf('%08x-0000-4000-8000-%012x', 1, c.n) FROM tcompany c
UNION ALL SELECT printf('%08x-0000-4000-8000-%012x', 41, CASE WHEN c.n % 3 = 0 THEN 4 ELSE 3 END), printf('%08x-0000-4000-8000-%012x', 1, c.n) FROM tcompany c
UNION ALL SELECT printf('%08x-0000-4000-8000-%012x', 41, c.n % 3 + 5), printf('%08x-0000-4000-8000-%012x', 1, c.n) FROM tcompany c
UNION ALL SELECT printf('%08x-0000-4000-8000-%012x', 41, CASE WHEN c.n % 2 = 0 THEN 8 ELSE 9 END), printf('%08x-0000-4000-8000-%012x', 1, c.n) FROM tcompany c
UNION ALL SELECT printf('%08x-0000-4000-8000-%012x', 41, CASE WHEN c.n <= 5 THEN 10 ELSE 11 END), printf('%08x-0000-4000-8000-%012x', 1, c.n) FROM tcompany c;

INSERT INTO project_connection (id, project_id, connection_id, label, target, created_at, updated_at)
SELECT printf('%08x-0000-4000-8000-%012x', 42, p.pn * 10000 + cc.rowid),
       printf('%08x-0000-4000-8000-%012x', 2, p.pn),
       ec.id,
       lower(ec.kind),
       CASE ec.kind WHEN 'GITLAB' THEN 'platform/' || m.label
                    WHEN 'GITHUB' THEN 'example-org/' || m.label
                    WHEN 'KUBERNETES' THEN 'namespace/' || m.label
                    WHEN 'JIRA' THEN upper(substr(m.label, 1, 3))
                    ELSE 'dashboard/' || m.label END,
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-60 days'),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-60 days')
FROM tproject p
JOIN tproject_template m ON m.n = p.tmpl
JOIN connection_company cc ON cc.company_id = printf('%08x-0000-4000-8000-%012x', 1, p.cn)
JOIN external_connection ec ON ec.id = cc.connection_id
WHERE (ec.kind = 'GITLAB' AND p.pn % 2 = 0)
   OR (ec.kind = 'GITHUB' AND p.pn % 2 = 1)
   OR ec.kind = 'KUBERNETES'
   OR (ec.kind = 'JIRA' AND p.pn % 3 < 2)
   OR (ec.kind = 'GRAFANA' AND p.pn % 3 = 0);

INSERT INTO connection_access (id, connection_id, task_id, terminal_id, accessed_at, granted, reason)
SELECT printf('%08x-0000-4000-8000-%012x', 43, n.n),
       printf('%08x-0000-4000-8000-%012x', 41, (n.n * 5) % 11 + 1),
       printf('%08x-0000-4000-8000-%012x', 3, (n.n * 29) % 640 + 1),
       printf('term-%04x', n.n * 977 % 65535),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (n.n * 53 % 40000) || ' minutes'),
       n.n % 6 <> 0,
       CASE WHEN n.n % 6 = 0 THEN 'The connection is not bound to this task''s project.' END
FROM seq n WHERE n.n <= 80;

------------------------------------------------------------------------------------------------
-- Vulnerabilities
------------------------------------------------------------------------------------------------

CREATE TEMP TABLE tdependency (n INTEGER PRIMARY KEY, ecosystem TEXT, name TEXT, version TEXT, manifest TEXT);
INSERT INTO tdependency VALUES
 (1, 'NPM', 'lodash', '4.17.15', 'package.json'),
 (2, 'NPM', 'axios', '0.21.1', 'package.json'),
 (3, 'RUST', 'regex', '1.5.4', 'Cargo.lock'),
 (4, 'PIP', 'pillow', '8.2.0', 'requirements.txt'),
 (5, 'MAVEN', 'org.apache.logging.log4j:log4j-core', '2.14.1', 'pom.xml'),
 (6, 'GO', 'golang.org/x/net', '0.7.0', 'go.mod'),
 (7, 'NPM', 'vue', '3.4.21', 'package.json'),
 (8, 'RUST', 'tokio', '1.38.0', 'Cargo.lock'),
 (9, 'PIP', 'requests', '2.31.0', 'requirements.txt'),
 (10, 'NPM', 'typescript', '5.4.5', 'package.json');

INSERT INTO project_dependency (id, project_id, ecosystem, name, version, manifest, created_at)
SELECT printf('%08x-0000-4000-8000-%012x', 14, p.pn * 100 + d.n),
       printf('%08x-0000-4000-8000-%012x', 2, p.pn),
       d.ecosystem, d.name, d.version, d.manifest,
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-3 days')
FROM tproject p JOIN tdependency d ON (d.n + p.pn) % 5 <> 0;

INSERT INTO security_advisory (id, ghsa_id, cve_id, summary, description, url, published_at, affected_json,
                               created_at, updated_at, severity, malware)
SELECT printf('%08x-0000-4000-8000-%012x', 15, a.n),
       printf('GHSA-demo-%04d-seed', a.n),
       printf('CVE-2025-%05d', 40000 + a.n),
       a.summary, a.summary || '. Demo advisory: not a real vulnerability.',
       printf('https://example.com/advisories/GHSA-demo-%04d-seed', a.n),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (a.n * 9) || ' days'),
       json_array(json_object('ecosystem', d.ecosystem, 'name', d.name, 'vulnerableRange', '<= ' || d.version,
                              'firstPatched', d.version || '-patched', 'vulnerableFunctions', json_array())),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (a.n * 9) || ' days'),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-' || (a.n * 9) || ' days'),
       a.severity, a.malware
FROM (SELECT 1 AS n, 'Prototype pollution in a merge helper' AS summary, 'CRITICAL' AS severity, 0 AS malware
      UNION ALL SELECT 2, 'Server-side request forgery through redirects', 'HIGH', 0
      UNION ALL SELECT 3, 'Denial of service through a crafted pattern', 'HIGH', 0
      UNION ALL SELECT 4, 'Out-of-bounds read when decoding an image', 'CRITICAL', 0
      UNION ALL SELECT 5, 'Remote code execution through a lookup string', 'CRITICAL', 0
      UNION ALL SELECT 6, 'HTTP/2 rapid reset exhausts the server', 'MEDIUM', 0
      UNION ALL SELECT 7, 'Package published with an install-time script that exfiltrates environment variables', 'CRITICAL', 1) a
JOIN tdependency d ON d.n = a.n;

INSERT INTO vulnerability_exposure (id, project_id, advisory_id, ecosystem, package_name, package_version, verdict,
                                    reasoning, conclusion, assessed_at, assessed_commit, fix_task_id, created_at, updated_at)
SELECT printf('%08x-0000-4000-8000-%012x', 16, p.pn * 100 + d.n),
       printf('%08x-0000-4000-8000-%012x', 2, p.pn),
       printf('%08x-0000-4000-8000-%012x', 15, d.n),
       pd.ecosystem, pd.name, pd.version,
       CASE p.pn % 3 WHEN 0 THEN 'AFFECTED' WHEN 1 THEN 'NOT_AFFECTED' END,
       CASE p.pn % 3 WHEN 0 THEN 'The vulnerable function is called from the request handler with user input.'
                      WHEN 1 THEN 'The package is a transitive dependency of a build tool and is never loaded at runtime.' END,
       CASE p.pn % 3 WHEN 0 THEN 'Reachable: upgrade before the next release.'
                      WHEN 1 THEN 'Not reachable from any entry point.' END,
       CASE WHEN p.pn % 3 < 2 THEN strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-2 days') END,
       CASE WHEN p.pn % 3 < 2 THEN substr(lower(hex(randomblob(20))), 1, 40) END,
       CASE WHEN p.pn % 3 = 0 AND p.pn % 2 = 0 THEN printf('%08x-0000-4000-8000-%012x', 3, (p.pn - 1) * 16 + 7) END,
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-3 days'),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-2 days')
FROM project_dependency pd
JOIN tproject p ON printf('%08x-0000-4000-8000-%012x', 2, p.pn) = pd.project_id
JOIN tdependency d ON d.ecosystem = pd.ecosystem AND d.name = pd.name AND d.version = pd.version
JOIN security_advisory a ON a.id = printf('%08x-0000-4000-8000-%012x', 15, d.n) AND d.n <= 7;

------------------------------------------------------------------------------------------------
-- Deployment graphs and diagrams
------------------------------------------------------------------------------------------------

INSERT INTO deployment_graph (id, project_id, title, description, created_at, updated_at, release_default)
SELECT printf('%08x-0000-4000-8000-%012x', 17, p.pn),
       printf('%08x-0000-4000-8000-%012x', 2, p.pn),
       'Release', 'Build, stage, smoke test and promote to production.',
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-70 days'),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-20 days'), 1
FROM tproject p WHERE p.pn % 2 = 0;

INSERT INTO graph_step (id, graph_id, position, title, body_markdown, connection_id, max_visits, created_at, flow_x, flow_y)
SELECT printf('%08x-0000-4000-8000-%012x', 18, p.pn * 10 + k.n),
       printf('%08x-0000-4000-8000-%012x', 17, p.pn),
       k.n - 1,
       CASE k.n WHEN 1 THEN 'Build artefacts' WHEN 2 THEN 'Deploy to staging' WHEN 3 THEN 'Smoke test staging' ELSE 'Deploy to production' END,
       CASE k.n WHEN 1 THEN 'Run the pipeline and publish the images.'
                WHEN 2 THEN 'Roll the new images out to the staging namespace.'
                WHEN 3 THEN 'Hit the health endpoints and the three critical journeys.'
                ELSE 'Promote the same images; watch the error rate.' END,
       NULL, CASE WHEN k.n = 2 THEN 3 END,
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-70 days'), NULL, NULL
FROM tproject p JOIN seq k ON k.n <= 4 WHERE p.pn % 2 = 0;

INSERT INTO graph_route (id, graph_id, from_step_id, to_step_id, outcome, position)
SELECT printf('%08x-0000-4000-8000-%012x', 19, p.pn * 10 + k.n),
       printf('%08x-0000-4000-8000-%012x', 17, p.pn),
       printf('%08x-0000-4000-8000-%012x', 18, p.pn * 10 + k.n),
       CASE WHEN k.n < 4 THEN printf('%08x-0000-4000-8000-%012x', 18, p.pn * 10 + k.n + 1) END,
       CASE WHEN k.n < 4 THEN 'next' ELSE 'done' END, 0
FROM tproject p JOIN seq k ON k.n <= 4 WHERE p.pn % 2 = 0;

INSERT INTO graph_route (id, graph_id, from_step_id, to_step_id, outcome, position)
SELECT printf('%08x-0000-4000-8000-%012x', 19, p.pn * 10 + 5),
       printf('%08x-0000-4000-8000-%012x', 17, p.pn),
       printf('%08x-0000-4000-8000-%012x', 18, p.pn * 10 + 3),
       printf('%08x-0000-4000-8000-%012x', 18, p.pn * 10 + 2),
       'rollback', 1
FROM tproject p WHERE p.pn % 2 = 0;

INSERT INTO diagram (id, project_id, task_id, title, question, kinds, document_json, created_at, updated_at, walkthrough)
SELECT printf('%08x-0000-4000-8000-%012x', 20, p.pn),
       printf('%08x-0000-4000-8000-%012x', 2, p.pn),
       NULL,
       'How a request is handled',
       'How does ' || m.title || ' handle a request from the client to the database?',
       'sequence',
       json_object('sheets', json_array(json_object(
           'title', 'Request path',
           'mermaid', 'sequenceDiagram' || char(10) || '  participant Client' || char(10) || '  participant Api' || char(10) || '  participant Db' || char(10) || '  Client->>Api: POST /orders' || char(10) || '  Api->>Db: insert order' || char(10) || '  Db-->>Api: ok' || char(10) || '  Api-->>Client: 201 Created'))),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-12 days'),
       strftime('%Y-%m-%dT%H:%M:%S.000000Z', 'now', '-12 days'),
       0
FROM tproject p JOIN tproject_template m ON m.n = p.tmpl WHERE p.pn % 2 = 1;

COMMIT;

SELECT 'companies' AS what, COUNT(*) AS rows FROM company
UNION ALL SELECT 'projects', COUNT(*) FROM project
UNION ALL SELECT 'tasks', COUNT(*) FROM task
UNION ALL SELECT 'steps', COUNT(*) FROM task_step
UNION ALL SELECT 'notes', COUNT(*) FROM document
UNION ALL SELECT 'wrapups', COUNT(*) FROM wrapup
UNION ALL SELECT 'time entries', COUNT(*) FROM time_entry
UNION ALL SELECT 'connections', COUNT(*) FROM external_connection
UNION ALL SELECT 'project bindings', COUNT(*) FROM project_connection
UNION ALL SELECT 'exposures', COUNT(*) FROM vulnerability_exposure;
