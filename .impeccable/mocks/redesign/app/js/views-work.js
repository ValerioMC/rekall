/* Work: ledger, board, task page, review queue. */
;(function () {
  const S = R.state

  R.tube = (done, claimed, total, live, cls) => {
    const t = total || 1
    return `<div class="tube ${cls || ''} ${live ? 'live' : ''}" role="img" aria-label="${done} done, ${claimed} claimed of ${total}"><i class="g" style="width:${(done / t) * 100}%"></i><i class="a" style="width:${(claimed / t) * 100}%"></i></div>`
  }
  R.md = (src) => {
    const inl = (s) => R.esc(s).replace(/`([^`]+)`/g, '<code>$1</code>').replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
    const lines = src.replace(/^---[\s\S]*?---\n/, '').split('\n'); let out = '', i = 0
    while (i < lines.length) {
      const l = lines[i]
      if (/^\|/.test(l) && /^\|[-| ]+\|?$/.test(lines[i + 1] || '')) {
        const row = (x, tag) => '<tr>' + x.replace(/^\||\|$/g, '').split('|').map((c) => `<${tag}>${inl(c.trim())}</${tag}>`).join('') + '</tr>'
        let t = `<table><thead>${row(l, 'th')}</thead><tbody>`; i += 2
        while (i < lines.length && /^\|/.test(lines[i])) t += row(lines[i++], 'td')
        out += t + '</tbody></table>'; continue
      }
      let m
      if ((m = /^(#{1,3}) (.*)/.exec(l))) out += `<h${m[1].length}>${inl(m[2])}</h${m[1].length}>`
      else if (/^(- |\d+\. )/.test(l)) { const ol = /^\d/.test(l); let t = ol ? '<ol>' : '<ul>'; while (i < lines.length && /^(- |\d+\. )/.test(lines[i])) t += `<li>${inl(lines[i++].replace(/^(- |\d+\. )/, ''))}</li>`; out += t + (ol ? '</ol>' : '</ul>'); continue }
      else if (l.trim()) out += `<p>${inl(l)}</p>`
      i++
    }
    return out
  }

  /* ---------- filtering ---------- */
  R.visibleTasks = (scope, view, q) => R.tasks.filter((t) => {
    if (scope.p && t.p !== scope.p) return false
    if (scope.co && R.project(t.p).co !== scope.co) return false
    if (q && !(t.title + ' ' + t.label).toLowerCase().includes(q.toLowerCase())) return false
    const k = R.kind(t)
    return view === 'needs' ? k === 'review' : view === 'running' ? k === 'run' : view === 'filed' ? k === 'filed' : k !== 'filed'
  })
  R.viewCounts = () => ({
    needs: R.reviewCount(),
    running: R.tasks.filter((t) => R.kind(t) === 'run').length,
    open: R.tasks.filter((t) => R.kind(t) !== 'filed').length + 41,
    filed: Object.values(R.filedTotals).reduce((a, b) => a + b, 0) + R.tasks.filter((t) => t.filedNow).length
  })
  const sorters = { default: () => 0, updated: (a, b) => R.agoDays(a.ago) - R.agoDays(b.ago), tokens: (a, b) => R.tokenNum(b.tokens) - R.tokenNum(a.tokens), need: (a, b) => R.needsCount(b) - R.needsCount(a) }

  /* ---------- the peek: what a row opens to, also the board sheet ---------- */
  R.peekBody = (t) => {
    const st = R.stepsOf(t.id), c = R.counts(t), claimed = st.filter((s) => s.state === 'CLAIMED')
    let rows = ''
    claimed.slice(0, 3).forEach((s) => (rows += `<div class="irow">${R.seal('CLAIMED')}<span class="trunc">${s.title}</span><span class="w">${s.ago}</span><span style="display:flex;gap:6px"><button class="btn sm" data-act="sendback" data-id="${s.id}">Send back</button><button class="btn sm primary" data-act="accept" data-id="${s.id}">Accept</button></span></div>`))
    if (t.taskReview) rows += `<div class="irow">${R.seal('CLAIMED')}<span class="trunc">Work handed in for your review</span><span class="w">${t.ago}</span><span style="display:flex;gap:6px"><button class="btn sm" data-act="sendbackTask" data-id="${t.id}">Send back</button><button class="btn sm primary" data-act="acceptTask" data-id="${t.id}">Accept</button></span></div>`
    if (!rows) {
      const next = st.filter((s) => s.state === 'RUNNING' || s.state === 'OPEN').slice(0, 2)
      rows = next.length ? next.map((s) => `<div class="irow">${R.seal(s.state)}<span class="trunc">${s.title}</span><span class="w">${s.state === 'RUNNING' ? 'working' : 'next'}</span><span></span></div>`).join('') : `<div class="irow" style="grid-template-columns:1fr"><span class="mut">${R.esc(t.desc || 'No description yet.')}</span></div>`
    }
    const extra = Math.max(0, c.claimed - 3)
    return `<div class="anc"><button class="anchor" data-act="copy" data-v="/rk ${R.anchorOf(t)}" title="Copy">/rk ${R.anchorOf(t)} ${R.icon('copy', 12)}</button><span>${[t.wrapup ? '1 wrapup' : '', t.deviations ? R.plural(t.deviations.length, 'deviation') : '', R.notesOf(t.id).length ? R.plural(R.notesOf(t.id).length, 'note') : ''].filter(Boolean).join(' · ') || 'No wrapup yet'}</span></div>${rows}
      <div class="foot"><a class="btn sm" href="#/task/${t.id}">Open task</a>${S.running === t.id ? `<span class="pill run"><i class="orb"></i> Session working</span>` : t.status === 'DONE' ? '' : `<button class="btn quiet sm" data-act="resume" data-id="${t.id}">${R.icon('play', 11)} Resume session</button>`}<span class="sp"></span>${extra ? `<span class="dim mono" style="font-size:var(--t-caption)">${extra} more to review</span>` : ''}</div>`
  }

  /* ---------- ledger ---------- */
  R.lrow = (t, dense) => {
    const k = R.kind(t), [lab, cls] = R.KIND[k], c = R.counts(t), open = S.open === t.id
    const plan = c.total ? `<span class="plan">${R.tube(c.done, c.claimed, c.total, k === 'run')}${c.done}/${c.total}</span>` : '<span class="dim mono" style="font-size:var(--t-caption)">no steps</span>'
    const need = k === 'run' ? '<span class="need" style="color:var(--session-strong)">working</span>' : R.needsCount(t) ? `<span class="need">${c.claimed ? c.claimed + ' to review' : 'Review'}</span>` : '<span class="dim">·</span>'
    const notes = R.notesOf(t.id).length || t.notes || ''
    return `<div class="lrow ${open ? 'open' : ''} ${dense ? 'dense' : ''} ${k === 'filed' ? 'filedt' : ''} ${t.fresh ? 'arrive' : ''}" data-act="rowopen" data-id="${t.id}" tabindex="0" role="button" aria-expanded="${open}">
      ${R.mk(R.markOf(k))}<span class="t"><span class="trunc">${R.esc(t.title)}</span><button class="cp" data-act="copy" data-v="/rk ${R.anchorOf(t)}" title="Copy /rk ${R.anchorOf(t)}" aria-label="Copy anchor">${R.icon('copy', 13)}</button></span>
      <span><span class="pill ${cls}">${lab}</span></span>${plan}${need}<span class="dim num r mono" style="font-size:var(--t-caption)">${notes}</span><span class="dim r mono" style="font-size:var(--t-caption)">${t.tokens === '0' ? '' : t.tokens}</span><span class="dim r mono" style="font-size:var(--t-caption)">${t.ago}</span></div>` + (open ? `<div class="inline">${R.peekBody(t)}</div>` : '')
  }
  R.ledgerTable = (scope, view, q, o) => {
    o = o || {}
    const list = R.visibleTasks(scope, view, q).sort(sorters[S.sort] || sorters.default)
    const byP = R.projects.map((p) => ({ p, rows: list.filter((t) => t.p === p.id) })).filter((g) => g.rows.length)
    const sortBtn = (k, l, cls) => `<button class="${cls || ''} ${S.sort === k ? 'on' : ''}" data-act="sort" data-k="${k}">${l}${S.sort === k ? R.icon('sort', 11) : ''}</button>`
    let html = `<div class="cols" role="row"><span></span>${sortBtn('default', 'Task')}<span>Status</span><span>Plan</span>${sortBtn('need', 'Needs you')}<span class="r">Notes</span>${sortBtn('tokens', 'Tokens', 'r')}${sortBtn('updated', 'Updated', 'r')}</div>`
    if (!byP.length) html += view === 'needs' ? `<div style="padding:72px 0;display:grid;justify-items:center;gap:12px;text-align:center"><div style="width:56px;height:56px">${R.seal('DONE', { complete: true, arrive: 'DONE' })}</div><div class="hero-line" style="font-size:var(--t-heading);font-weight:500">Nothing waits for you.</div><p class="mut">Every claimed step is accepted. Resume a session to get more.</p></div>` : '<div class="empty" style="margin-top:32px"><b>No task matches.</b>Clear the filter or change the scope.</div>'
    byP.forEach(({ p, rows }) => {
      const co = R.company(p.co), nd = rows.reduce((a, t) => a + R.needsCount(t), 0)
      html += `<div class="grp">${R.pAvatar(p, 'sm')}<a href="#/projects/${p.id}"><b>${p.name}</b></a><span class="co">${co.name}</span><span class="cnt">${R.plural(rows.length, 'task')}${nd ? ` · <span class="hot">${nd} to review</span>` : ''}</span><span class="tok">${p.tokens} tokens</span></div>`
      html += rows.map((t) => R.lrow(t, S.dense)).join('')
      if (view !== 'filed' && R.filedTotals[p.id]) {
        const open = S.filedOpen[p.id], ft = R.tasks.filter((t) => t.p === p.id && R.kind(t) === 'filed')
        html += `<button class="filedrow ${open ? 'open' : ''}" data-act="filed" data-id="${p.id}">${R.icon('right', 12)} ${R.filedTotals[p.id] + R.tasks.filter((t) => t.p === p.id && t.filedNow).length} filed</button>` + (open ? ft.map((t) => R.lrow(t, S.dense)).join('') : '')
      }
    })
    return html
  }
  R.scopeLabel = () => S.scope.p ? R.project(S.scope.p).name : S.scope.co ? R.company(S.scope.co).name : 'All work'
  R.workHead = (mode) => {
    const vc = R.viewCounts()
    return `<div class="page-h" style="padding-bottom:14px"><h1>Work</h1><button class="select" data-act="scope">${R.scopeLabel()} <span class="c">${R.tasks.filter((t) => (!S.scope.p || t.p === S.scope.p) && (!S.scope.co || R.project(t.p).co === S.scope.co)).length + 300} tasks</span>${R.icon('chevron', 14)}</button>
      <label class="field" style="width:280px">${R.icon('search', 15)}<input id="wq" data-input="wq" placeholder="Filter by title or anchor" value="${R.esc(S.q)}" aria-label="Filter"></label><span class="sp"></span>
      <div class="seg" role="group" aria-label="Layout"><a class="${mode === 'ledger' ? 'on' : ''}" href="#/work" style="display:flex"><button class="${mode === 'ledger' ? 'on' : ''}" tabindex="-1">${R.icon('list', 15)}Ledger</button></a><a href="#/work/board" style="display:flex"><button class="${mode === 'board' ? 'on' : ''}" tabindex="-1">${R.icon('board', 15)}Board</button></a></div>
      ${mode === 'ledger' ? `<div class="seg" role="group" aria-label="Density"><button class="${S.dense ? '' : 'on'}" data-act="dense" data-v="0">Comfortable</button><button class="${S.dense ? 'on' : ''}" data-act="dense" data-v="1">Compact</button></div>` : ''}
      <button class="btn primary" data-act="newtask">${R.icon('plus', 14)}New task</button></div>
      ${mode === 'ledger' ? `<div class="subnav" role="tablist">${[['needs', 'Needs me', vc.needs, 1], ['running', 'Running', vc.running], ['open', 'Open', vc.open], ['filed', 'Filed', vc.filed]].map(([k, l, n, h]) => `<button role="tab" class="${S.view === k ? 'on' : ''}" data-act="view" data-k="${k}">${l}<span class="n ${h && n ? 'hot' : ''}">${n}</span></button>`).join('')}</div>` : ''}`
  }
  R.routes.work = (r) => {
    const board = r.a === 'board'
    if (board) return R.board()
    return `<div class="view"><div class="scroll" id="wscroll"><div class="page">${R.workHead('ledger')}${R.ledgerTable(S.scope, S.view, S.q)}</div></div></div>`
  }

  /* ---------- board ---------- */
  R.board = () => {
    const all = R.visibleTasks(S.scope, 'open', S.q), kind = (t) => R.kind(t)
    const lanes = [['To do', 'todo', all.filter((t) => kind(t) === 'todo'), 35], ['In progress', 'prog', all.filter((t) => kind(t) === 'prog' || kind(t) === 'run'), 39], ['Awaiting your review', 'claimed', all.filter((t) => kind(t) === 'review'), 0]]
    const card = (t) => {
      const p = R.project(t.p), c = R.counts(t), k = kind(t)
      const foot = [R.needsCount(t) ? `<span class="need">${c.claimed ? c.claimed + ' to review' : 'Review'}</span>` : '', k === 'run' ? '<span class="need" style="color:var(--session-strong)">working</span>' : '', c.total ? `<span>${c.done}/${c.total}</span>` : '', R.notesOf(t.id).length || t.notes ? `<span>${R.notesOf(t.id).length || t.notes} notes</span>` : '', t.tokens !== '0' ? `<span>${t.tokens}</span>` : ''].filter(Boolean).join('')
      return `<button class="card ${t.fresh ? 'arrive' : ''}" data-act="sheet" data-id="${t.id}"><span class="top2">${R.pAvatar(p, 'sm')}<span class="trunc">${p.name}</span><span class="sp"></span><span>${t.ago}</span></span><h3>${R.esc(t.title)}</h3>${c.total ? R.tube(c.done, c.claimed, c.total, k === 'run') : ''}<span class="foot">${foot}</span></button>`
    }
    const n = R.viewCounts()
    const sheet = S.sheet ? R.task(S.sheet) : null
    return `<div class="view"><div class="page" style="max-width:none;padding-bottom:0">${R.workHead('board')}</div><div class="lanes">${lanes.map(([name, mk, list, base]) => `<section class="lane ${name.startsWith('Awaiting') ? 'review' : ''}"><div class="lane-h">${R.mk(mk)}${name}<span class="n ${name.startsWith('Awaiting') ? 'hot' : ''}">${list.length + base}</span><span class="sp"></span><button class="ibtn" data-act="newtask" aria-label="Add to ${name}">${R.icon('plus', 15)}</button></div><div class="cards scroll">${list.map(card).join('') || '<div class="empty" style="margin:6px 0">Empty</div>'}${base ? `<div class="dim" style="font-size:var(--t-caption);padding:6px 4px">and ${base} more, oldest first</div>` : ''}</div></section>`).join('')}
      <section class="lane filed"><div class="lane-h">${R.mk('filed')}Filed</div><div style="padding:6px 14px"><div class="num" style="font:300 28px var(--font-sans);color:var(--filed)">${n.filed}</div><p class="mut" style="font-size:var(--t-small);line-height:1.5;margin-top:6px">Finished tasks stay out of the way.</p><a class="btn sm quiet" style="margin-top:8px;padding:0" href="#/work" data-act="filedview">Open the shelf</a></div></section></div>
      ${sheet ? `<div class="scrim" style="inset:48px 0 0 0;padding:0;display:block;z-index:30" data-act="sheetclose"></div><aside class="decide" style="position:fixed;top:48px;right:0;bottom:0;width:600px;z-index:31;background:var(--canvas);border-left:1px solid var(--border-strong);box-shadow:-24px 0 60px #0009;animation:rise var(--m-slow) var(--ease)"><div style="display:flex;align-items:center;gap:8px;color:var(--muted);font-size:var(--t-small)">${R.pAvatar(R.project(sheet.p), 'sm')}<span>${R.company(R.project(sheet.p).co).name} / ${R.project(sheet.p).name}</span><span class="sp"></span><a class="btn quiet sm" href="#/task/${sheet.id}">Open as page</a><button class="ibtn" data-act="sheetclose" aria-label="Close">${R.icon('close')}</button></div><h2 style="font-size:var(--t-heading);font-weight:500">${R.esc(sheet.title)}</h2><div class="inline" style="background:none;box-shadow:none;padding:0;animation:none">${R.peekBody(sheet)}</div></aside>` : ''}</div>`
  }

  /* ---------- task page ---------- */
  R.selStep = (t) => {
    const st = R.stepsOf(t.id), cur = S.sel[t.id] && R.step(S.sel[t.id])
    if (cur) return cur
    return st.find((s) => s.state === 'CLAIMED') || st.find((s) => s.state === 'RUNNING') || st.find((s) => s.state === 'OPEN') || st[0] || null
  }
  R.stepBody = (s) => `Step brief: ${R.esc(s.title)}. The session reads the repository, makes the change named here and runs the checks configured on the project before it claims the step.`
  R.decide = (t, s) => {
    if (!s) {
      if (t.taskReview) return `<div class="k">${R.seal('CLAIMED')}Work handed in · ${t.ago} ago</div><h2>${R.esc(t.title)}</h2><p class="claim">${t.wrapup ? R.esc(t.wrapup.text) : 'The session handed in the whole task without a checklist. Read the wrapup and the diff, then decide.'}</p><div class="actions"><button class="btn" data-act="sendbackTask" data-id="${t.id}">Send back</button><button class="btn primary" data-act="acceptTask" data-id="${t.id}">Accept task</button></div>`
      return `<div class="empty"><b>Pick a step</b>Its brief and what the session claimed appear here.</div>`
    }
    const cl = R.claims[s.n] && s.task === 't-metric' ? R.claims[s.n] : R.defaultClaim
    const steps = R.stepsOf(t.id), idx = steps.indexOf(s)
    const nexts = steps.filter((x) => x.state === 'CLAIMED' && x !== s).slice(0, 3)
    let head = `<div class="k">${R.seal(s.state, { arrive: S.arrivals[s.id] })}Step ${s.n} of ${steps.filter((x) => x.state !== 'DRAFT').length}${s.state === 'CLAIMED' ? ` · claimed ${s.ago}` : s.state === 'RUNNING' ? ' · a session is working on it' : s.state === 'DONE' ? ' · done' : s.state === 'DRAFT' ? ' · draft' : ' · open'}</div><h2>${R.esc(s.title)}</h2>`
    if (s.state === 'CLAIMED') return head + `<p class="claim">${cl.summary}</p><div class="checks">${cl.checks.map((c) => `<div><span>${c[0]}</span><span class="${c[1]}">${c[2]}</span></div>`).join('')}</div><div class="actions"><button class="btn" data-act="sendback" data-id="${s.id}">Send back <kbd>S</kbd></button><button class="btn primary" data-act="accept" data-id="${s.id}">Accept <kbd style="color:var(--accent-ink);border-color:var(--accent-shade);background:transparent">A</kbd></button></div><div class="quiet-row" style="display:flex;justify-content:space-between;font-size:var(--t-caption);color:var(--subtle)"><button class="btn quiet sm" style="padding:0" data-act="toast" data-v="Opening the diff">Open the diff</button><span>Pass 1 · 2 files</span></div>` + (nexts.length ? `<div class="upnext"><div class="k" style="margin-bottom:6px">Up next to review</div>${nexts.map((x) => `<div class="un" data-act="selstep" data-id="${x.id}"><span class="no">${x.n}</span><span class="trunc">${R.esc(x.title)}</span></div>`).join('')}</div>` : '')
    if (s.state === 'RUNNING') return head + `<p class="claim">The session is editing the files named in this step. When it finishes it claims the step and it appears here for your review. Claude can move a step only up to claimed.</p><div class="checks"><div><span>Session</span><span style="color:var(--session-strong)">working</span></div><div><span>Started</span><span>just now</span></div></div><button class="btn" data-act="sessions">${R.icon('terminal', 14)} Watch the session</button>`
    if (s.state === 'OPEN') return head + `<p class="claim">${R.stepBody(s)}</p><div class="actions"><button class="btn" data-act="markdone" data-id="${s.id}">Mark done myself</button>${S.running ? '' : `<button class="btn primary" data-act="resume" data-id="${t.id}">${R.icon('play', 12)} Give to a session</button>`}</div>`
    if (s.state === 'DRAFT') return head + `<p class="claim">A draft is not on the checklist yet. Promote it when its detail is right.</p><div class="actions"><button class="btn primary" data-act="promote" data-id="${s.id}">Promote to the checklist</button></div>`
    return head + `<p class="claim">Accepted. ${R.stepBody(s)}</p><div class="actions"><button class="btn" data-act="reopen" data-id="${s.id}">Reopen this step</button></div>`
  }
  R.routes.task = (r) => {
    const t = R.task(r.a); if (!t) return '<div class="view"><div class="page"><div class="empty" style="margin-top:60px"><b>No such task.</b><a class="btn" href="#/work">Back to work</a></div></div></div>'
    const tab = r.b || 'plan', p = R.project(t.p), co = R.company(p.co), c = R.counts(t), k = R.kind(t), [lab, cls] = R.KIND[k]
    const notes = R.notesOf(t.id), devs = t.deviations || []
    const tabs = [['plan', 'Plan', c.total || '', c.claimed], ['wrapup', 'Wrapup', '', 0], ['notes', 'Notes', notes.length || '', 0], ['description', 'Description', '', 0], ['time', 'Time', '', 0], ['deviations', 'Deviations', devs.length || '', 0]]
    const same = R.reviewItems().filter((i) => i.task.id !== t.id)
    const head = `<div class="thead"><div class="crumbs"><a href="#/work">Work</a><span class="dim">/</span><a href="#/projects/${p.id}">${co.name} · ${p.name}</a><span class="sp"></span><button class="btn sm" data-act="switch">Switch task <kbd>⌘K</kbd></button></div>
      <h1 class="ttitle">${R.esc(t.title)}</h1>
      <div class="tmeta"><button data-act="status" data-id="${t.id}" class="pill ${cls}" title="Change status">${k === 'run' ? '<i class="orb"></i>' : ''}${lab}${R.icon('chevron', 12)}</button><button class="anchor" data-act="copy" data-v="/rk ${R.anchorOf(t)}">/rk ${R.anchorOf(t)} ${R.icon('copy', 12)}</button><span class="sp"></span>${t.time ? `<span>${t.time} across <b>${R.plural(t.sessions || 1, 'session')}</b></span>` : ''}<span><b class="num">${t.tokens}</b> tokens</span><button class="btn quiet" data-act="term" data-v="${p.label}">${R.icon('terminal', 15)}Open in terminal</button>${S.running === t.id ? `<button class="btn" data-act="sessions"><i class="orb"></i> Watch the session</button>` : t.status === 'DONE' ? '' : `<button class="btn primary" data-act="resume" data-id="${t.id}">${R.icon('play', 13)}Resume session</button>`}</div>
      <div class="subnav" style="border:0;margin:0" role="tablist">${tabs.map(([id, n, cnt, hot]) => `<a class="${tab === id ? 'on' : ''}" href="#/task/${t.id}/${id}" role="tab">${n}${cnt !== '' ? `<span class="n ${hot ? 'hot' : ''}">${cnt}</span>` : ''}</a>`).join('')}</div></div>`
    let body = ''
    if (tab === 'plan') {
      const st = R.stepsOf(t.id), main = st.filter((s) => s.state !== 'DRAFT'), drafts = st.filter((s) => s.state === 'DRAFT'), sel = R.selStep(t)
      const nextOpen = main.find((s) => s.state === 'OPEN'), all = main.length && main.every((s) => s.state === 'DONE')
      const visible = S.hideDone ? main.filter((s) => s.state !== 'DONE') : main
      const line = c.claimed ? `<b class="hot">${R.plural(c.claimed, 'step')} ${c.claimed === 1 ? 'is' : 'are'} claimed and ${c.claimed === 1 ? 'waits' : 'wait'} for you.</b> ${c.done} of ${c.total} done` : c.running ? `A session is working on step ${main.find((s) => s.state === 'RUNNING').n}. ${c.done} of ${c.total} done` : c.total ? `${c.done} of ${c.total} done` : ''
      const stepRow = (s) => `<div class="step ${s.state} seal-hover ${sel && sel.id === s.id ? 'sel' : ''}" data-act="selstep" data-id="${s.id}" tabindex="0">${R.seal(s.state, { next: nextOpen === s, complete: all, arrive: S.arrivals[s.id] })}<span class="no">${s.n}</span><span class="t trunc">${R.esc(s.title)}</span><span class="when"><span class="hov">${s.state === 'CLAIMED' ? `<button class="btn sm primary" data-act="accept" data-id="${s.id}">Accept</button>` : s.state === 'OPEN' ? `<button class="btn sm" data-act="markdone" data-id="${s.id}">Done</button>` : s.state === 'DRAFT' ? `<button class="btn sm" data-act="promote" data-id="${s.id}">Promote</button>` : ''}</span>${s.state === 'CLAIMED' ? s.ago : s.state === 'RUNNING' ? '<span style="color:var(--session-strong)">working</span>' : s.state === 'DONE' ? '' : ''}</span></div>`
      body = `<div class="stage ${sel || t.taskReview ? '' : 'solo'}"><section class="plan-col scroll">${c.total ? `<div class="sumline"><div>${R.tube(c.done, c.claimed, c.total, c.running > 0, 'lg')}<p>${line}</p></div><div style="display:flex;gap:8px"><button class="btn quiet sm" data-act="hidedone">${S.hideDone ? 'Show done' : 'Hide done'}</button>${c.claimed ? '<button class="btn sm" data-act="reviewnext">Review next</button>' : ''}</div></div>` : ''}
        ${visible.map(stepRow).join('') || (c.total ? '<p class="mut" style="padding:20px 12px">Every step is done. Show them again to correct one.</p>' : `<div class="empty"><b>No steps yet</b>${t.taskReview ? 'This task was handed in whole. Review it on the right.' : 'Draft the first step: a session picks up the checklist from there.'}</div>`)}
        ${drafts.length ? `<div class="sec-h" style="margin:22px 0 6px"><h2 style="font-size:var(--t-small);color:var(--muted)">Drafts</h2><span class="c">${drafts.length}</span></div>${drafts.map(stepRow).join('')}` : ''}
        <form class="addstep" data-form="addstep" data-id="${t.id}"><label class="field">${R.icon('plus', 14)}<input id="ns" placeholder="Draft a step" aria-label="New step"></label><button class="btn" type="submit">Add draft</button></form></section>
        ${sel || t.taskReview ? `<aside class="decide scroll">${R.decide(t, sel)}</aside>` : ''}</div>`
    } else if (tab === 'wrapup') body = `<div class="scroll"><section class="pad"><h3>Wrapup ${t.wrapup ? `<span class="dim" style="font-weight:400;font-size:var(--t-caption)">· ${t.wrapup.by} · ${t.wrapup.ago}</span>` : ''}</h3>${t.wrapup ? `<p>${R.esc(t.wrapup.text)}</p><p style="margin-top:14px" class="dim">A wrapup is a state, overwritten each time the implementation changes. It is not a log.</p>` : '<div class="empty"><b>No wrapup yet</b>A session writes one when it hands work in.</div>'}</section></div>`
    else if (tab === 'notes') {
      const cur = notes.find((n) => n.id === r.c) || notes[0]
      body = notes.length ? `<div class="notes-wrap"><div class="nlist">${notes.map((n) => `<a class="nitem ${cur && n.id === cur.id ? 'on' : ''}" href="#/task/${t.id}/notes/${n.id}">${R.icon('note', 15)}<span class="trunc">${n.title}</span></a>`).join('')}<button class="btn quiet sm" style="margin-top:8px" data-act="addnote" data-id="${t.id}">${R.icon('plus', 13)} Add information</button></div><div class="scroll"><div class="nbar"><span class="anchor">/rk ${R.anchorOf(t)}</span><span class="sp"></span><div class="seg"><button class="on">In full</button><button data-act="toast" data-v="Shared by reference">By reference</button></div></div><div class="md">${cur ? R.md(cur.body) : ''}</div></div></div>` : `<div class="scroll"><section class="pad"><div class="empty"><b>Nothing here yet</b>Notes attach context to this task: a reference, a decision, a finding.<br><button class="btn" style="margin-top:14px" data-act="addnote" data-id="${t.id}">${R.icon('plus', 14)}Add information</button></div></section></div>`
    } else if (tab === 'description') body = `<div class="scroll"><section class="pad"><h3>Description</h3>${S.editDesc === t.id ? `<textarea class="editor" id="descbox">${R.esc(t.desc || '')}</textarea><div style="display:flex;gap:8px;margin-top:12px"><button class="btn primary" data-act="descsave" data-id="${t.id}">Save</button><button class="btn quiet" data-act="descedit" data-id="">Cancel</button></div>` : `<p>${R.esc(t.desc || 'No description yet.')}</p><button class="btn" style="margin-top:16px" data-act="descedit" data-id="${t.id}">${R.icon('edit', 14)}Edit</button>`}</section></div>`
    else if (tab === 'time') body = `<div class="scroll"><section class="pad"><h3>Time</h3><div class="grid g3" style="margin-bottom:22px"><div class="box stat"><span>Tracked</span><b>${t.time || '0m'}</b><span>${R.plural(t.sessions || 0, 'session')}</span></div><div class="box stat"><span>Tokens</span><b>${t.tokens}</b><span>generated</span></div><div class="box stat"><span>Last session</span><b>${t.ago}</b><span>ago</span></div></div>${(t.sessions ? Array.from({ length: Math.min(t.sessions, 5) }, (_, i) => `<div class="list-row" style="grid-template-columns:90px 1fr auto"><span class="mono dim">Session ${t.sessions - i}</span><span class="mut">${['claimed 2 steps', 'claimed 1 step', 'no claim', 'claimed 3 steps', 'sent back'][i]}</span><span class="mono mut">${[38, 52, 17, 64, 29][i]}m</span></div>`).join('') : '<div class="empty"><b>No session yet</b></div>')}</section></div>`
    else body = `<div class="scroll"><section class="pad"><h3>Deviations <span class="dim" style="font-weight:400">${devs.length}</span></h3>${devs.length ? devs.map((d) => `<div class="box" style="padding:14px 16px;margin-bottom:10px"><div class="dim" style="font-size:var(--t-caption);margin-bottom:4px">Step ${d.step}</div><p>${R.esc(d.text)}</p></div>`).join('') : '<div class="empty"><b>The plan held</b>No step left the plan.</div>'}</section></div>`
    return `<div class="view">${head}${body}</div>`
  }

  /* ---------- review queue ---------- */
  R.routes.review = (r) => {
    const items = R.reviewItems()
    if (!items.length) return `<div class="view"><div class="page" style="display:grid;place-items:center;height:100%;text-align:center;gap:14px"><div style="width:84px;height:84px;margin:auto">${R.seal('DONE', { complete: true, arrive: 'DONE' })}</div><div class="hero-line">Nothing <b style="color:var(--safe)">waits</b> for you.</div><p class="mut">Every claimed step is accepted. Resume a session to get more.</p><a class="btn" href="#/work" style="justify-self:center">Back to work</a></div></div>`
    const cur = items.find((i) => i.key === (r.a || S.rsel)) || items[0]; S.rsel = cur.key
    const t = cur.task, p = R.project(t.p)
    const done = R.steps.filter((s) => s.state === 'DONE').length
    const list = items.map((i) => `<a class="rq-item ${i.key === cur.key ? 'sel' : ''}" href="#/review/${i.key}">${R.seal('CLAIMED')}<span><span class="l1">${R.esc(i.kind === 'step' ? i.step.title : i.task.title)}</span><span class="l2"><span class="trunc" style="max-width:190px">${R.esc(R.project(i.task.p).name)} · ${i.kind === 'step' ? 'step ' + i.step.n : 'whole task'}</span><span class="sp"></span><span>${i.kind === 'step' ? i.step.ago : i.task.ago}</span></span></span></a>`).join('')
    const detail = cur.kind === 'step' ? R.decide(t, cur.step) : R.decide(t, null)
    return `<div class="view"><div class="rq"><div class="rq-list scroll"><div style="padding:14px 10px 8px;display:flex;align-items:center;gap:10px"><h1 style="font-size:var(--t-heading);font-weight:500">Review</h1><span class="badge">${items.length}</span><span class="sp"></span><span class="dim" style="font-size:var(--t-caption)">longest waiting first</span></div>${list}</div>
      <div class="rq-detail scroll"><div class="crumbs" style="margin-bottom:14px">${R.pAvatar(p, 'sm')}<span>${R.company(p.co).name} / ${p.name}</span><span class="dim">·</span><a href="#/task/${t.id}" class="hot">${R.esc(t.title)}</a><span class="sp"></span><span class="dim" style="font-size:var(--t-caption)"><kbd>J</kbd> <kbd>K</kbd> walk · <kbd>A</kbd> accept · <kbd>S</kbd> send back</span></div>
      <div class="decide" style="background:none;border:0;padding:0;overflow:visible">${detail}</div>
      <div class="adh"><span><b>${done}</b>steps accepted</span><span><b>${R.state.sentBack || 0}</b>sent back this session</span><span><b>${R.tasks.filter((x) => x.deviations).length}</b>deviation</span></div></div></div></div>`
  }
  R.nextReview = () => { const items = R.reviewItems(), i = items.findIndex((x) => x.key === S.rsel); return (items[i + 1] || items[i - 1] || {}).key }
  R.onKey = (e) => {
    const r = R.route(), k = e.key.toLowerCase()
    if (r.name === 'review') {
      const items = R.reviewItems(), i = items.findIndex((x) => x.key === S.rsel)
      if (k === 'j' && items[i + 1]) location.hash = '#/review/' + items[i + 1].key
      if (k === 'k' && items[i - 1]) location.hash = '#/review/' + items[i - 1].key
      const cur = items[i]; if (!cur) return
      if (k === 'a') { const nx = R.nextReview(); cur.kind === 'step' ? R.accept(cur.key) : R.acceptTask(cur.key); S.rsel = nx; if (nx) location.hash = '#/review/' + nx }
      if (k === 's') { const nx = R.nextReview(); S.sentBack = (S.sentBack || 0) + 1; cur.kind === 'step' ? R.sendBack(cur.key) : R.sendBackTask(cur.key); S.rsel = nx; if (nx) location.hash = '#/review/' + nx }
    } else if (r.name === 'task' && (r.b || 'plan') === 'plan') {
      const t = R.task(r.a), s = t && R.selStep(t)
      if (s && s.state === 'CLAIMED' && k === 'a') R.accept(s.id)
      if (s && s.state === 'CLAIMED' && k === 's') R.sendBack(s.id)
    }
  }

  /* ---------- acts ---------- */
  R.inputs.wq = (v) => { S.q = v; R.render() }
  Object.assign(R.acts, {
    rowopen: (el, e) => { if (e.target.closest('.cp')) return; S.open = S.open === el.dataset.id ? null : el.dataset.id; R.render() },
    view: (el) => { S.view = el.dataset.k; S.open = null; R.render() },
    sort: (el) => { S.sort = el.dataset.k; R.render() },
    dense: (el) => { S.dense = el.dataset.v === '1'; R.render() },
    filed: (el) => { S.filedOpen[el.dataset.id] = !S.filedOpen[el.dataset.id]; R.render() },
    filedview: () => { S.view = 'filed'; location.hash = '#/work' },
    sheet: (el) => { S.sheet = el.dataset.id; R.render() },
    sheetclose: () => { S.sheet = null; R.render() },
    toast: (el) => R.toast(el.dataset.v),
    selstep: (el, e) => { if (e.target.closest('.btn')) return; const s = R.step(el.dataset.id); S.sel[s.task] = s.id; R.render() },
    hidedone: () => { S.hideDone = !S.hideDone; R.render() },
    reviewnext: () => { const t = R.task(R.route().a), s = R.stepsOf(t.id).find((x) => x.state === 'CLAIMED'); if (s) { S.sel[t.id] = s.id; R.render() } },
    switch: () => R.openPalette(),
    scope: (el) => { const items = [{ html: 'All work', on: !S.scope.co && !S.scope.p, run: () => { S.scope = { co: null, p: null }; R.render() } }, '-']; R.companies.forEach((c) => { items.push({ html: `<b style="font-weight:500">${c.name}</b>`, on: S.scope.co === c.id && !S.scope.p, run: () => { S.scope = { co: c.id, p: null }; R.render() } }); R.projects.filter((p) => p.co === c.id).forEach((p) => items.push({ html: `<span style="padding-left:14px;color:var(--muted)">${p.name}</span>`, on: S.scope.p === p.id, run: () => { S.scope = { co: null, p: p.id }; R.render() } })) }); R.pop(el, items) },
    status: (el) => { const t = R.task(el.dataset.id); R.pop(el, [['TODO', 'To do'], ['IN_PROGRESS', 'In progress'], ['DONE', 'Done']].map(([v, l]) => ({ html: l, on: t.status === v, run: () => { t.status = v; if (v === 'DONE') t.filedNow = true; R.toast('Status set to ' + l.toLowerCase()); R.render() } }))) },
    newtask: () => R.dialog({ title: 'New task', ok: 'Create task', body: `<label class="lbl">Project<select class="field" id="nt-p" style="width:100%;background:var(--surface)">${R.projects.map((p) => `<option value="${p.id}" ${p.id === (S.scope.p || 'stv') ? 'selected' : ''}>${R.company(p.co).name} · ${p.name}</option>`).join('')}</select></label><label class="lbl">Title<span class="field"><input id="nt-t" placeholder="What has to be done"></span></label><label class="lbl">Description<textarea class="editor" id="nt-d" style="min-height:90px" placeholder="Context for the session"></textarea></label>`, run: (d) => { const t = R.$('#nt-t', d).value.trim(); if (!t) { R.$('#nt-t', d).focus(); return false } R.newTask(R.$('#nt-p', d).value, t, R.$('#nt-d', d).value) } }),
    addnote: (el) => R.dialog({ title: 'Add information', ok: 'Add', body: `<label class="lbl">Title<span class="field"><input id="nn-t" placeholder="name-of-the-note.md"></span></label><label class="lbl">Content<textarea class="editor" id="nn-b" style="min-height:140px" placeholder="# Heading"></textarea></label>`, run: (d) => { const ti = R.$('#nn-t', d).value.trim(); if (!ti) return false; const n = { id: 'nx' + Date.now(), task: el.dataset.id, title: ti.endsWith('.md') ? ti : ti + '.md', body: R.$('#nn-b', d).value || '# ' + ti }; R.notes.push(n); location.hash = '#/task/' + el.dataset.id + '/notes/' + n.id; R.render() } }),
    descedit: (el) => { S.editDesc = el.dataset.id || null; R.render() },
    descsave: (el) => { R.task(el.dataset.id).desc = R.$('#descbox').value; S.editDesc = null; R.toast('Description saved'); R.render() }
  })
  document.addEventListener('submit', (e) => { const f = e.target.closest('[data-form=addstep]'); if (!f) return; e.preventDefault(); const v = R.$('#ns').value.trim(); if (v) R.addStep(f.dataset.id, v) })
})()
