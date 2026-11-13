/* Core: icons, marks, store, actions, router, shell, overlays. Mock only. */
;(function () {
  const ICONS = {
    copy: '<rect x="8" y="8" width="11" height="11" rx="2"/><path d="M5 15V6a2 2 0 0 1 2-2h8"/>', search: '<circle cx="11" cy="11" r="6"/><path d="m20 20-4-4"/>',
    settings: '<circle cx="12" cy="12" r="3"/><path d="M12 3v3M12 18v3M3 12h3M18 12h3M5.6 5.6l2.1 2.1M16.3 16.3l2.1 2.1M18.4 5.6l-2.1 2.1M7.7 16.3l-2.1 2.1"/>',
    tag: '<path d="M3 12V4h8l10 10-8 8z"/><circle cx="7.5" cy="8.5" r="1"/>', chevron: '<path d="m7 10 5 5 5-5"/>', right: '<path d="m10 7 5 5-5 5"/>', left: '<path d="m14 7-5 5 5 5"/>',
    plus: '<path d="M12 5v14M5 12h14"/>', play: '<path d="M8 5v14l11-7z"/>', stop: '<rect x="6" y="6" width="12" height="12" rx="2"/>', terminal: '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="m7 9 3 3-3 3M13 15h4"/>',
    note: '<path d="M6 3h9l4 4v14H6z"/><path d="M14 3v5h5"/>', clock: '<circle cx="12" cy="12" r="8"/><path d="M12 8v4l3 2"/>', check: '<path d="m5 12 5 5 9-10"/>',
    grid: '<rect x="4" y="4" width="7" height="7" rx="1.5"/><rect x="13" y="4" width="7" height="7" rx="1.5"/><rect x="4" y="13" width="7" height="7" rx="1.5"/><rect x="13" y="13" width="7" height="7" rx="1.5"/>',
    list: '<path d="M8 6h12M8 12h12M8 18h12M4 6h.01M4 12h.01M4 18h.01"/>', board: '<rect x="3" y="4" width="5" height="16" rx="1.5"/><rect x="10" y="4" width="5" height="10" rx="1.5"/><rect x="17" y="4" width="4" height="13" rx="1.5"/>',
    close: '<path d="m6 6 12 12M18 6 6 18"/>', sort: '<path d="M8 6v12m0 0-3-3m3 3 3-3M16 18V6m0 0-3 3m3-3 3 3"/>', shield: '<path d="M12 3 5 6v6c0 4 3 7 7 9 4-2 7-5 7-9V6z"/><path d="M12 8v4M12 15h.01"/>',
    calendar: '<rect x="4" y="5" width="16" height="15" rx="2"/><path d="M8 3v4M16 3v4M4 10h16"/>', folder: '<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>',
    link: '<path d="M10 14a4 4 0 0 0 6 0l3-3a4 4 0 0 0-6-6l-1 1M14 10a4 4 0 0 0-6 0l-3 3a4 4 0 0 0 6 6l1-1"/>', key: '<circle cx="8" cy="15" r="4"/><path d="m11 12 8-8M16 7l3 3"/>',
    refresh: '<path d="M20 11a8 8 0 0 0-14-4M4 5v4h4M4 13a8 8 0 0 0 14 4M20 19v-4h-4"/>', edit: '<path d="M4 20h4L20 8l-4-4L4 16z"/>', trash: '<path d="M5 7h14M10 4h4M7 7l1 13h8l1-13"/>',
    book: '<path d="M5 4h11a3 3 0 0 1 3 3v13H8a3 3 0 0 1-3-3z"/><path d="M5 17a3 3 0 0 1 3-3h11"/>', flow: '<rect x="3" y="4" width="6" height="5" rx="1"/><rect x="15" y="15" width="6" height="5" rx="1"/><path d="M6 9v4a3 3 0 0 0 3 3h6"/>',
    eye: '<path d="M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12z"/><circle cx="12" cy="12" r="3"/>', code: '<path d="m8 8-4 4 4 4M16 8l4 4-4 4M14 5l-4 14"/>', building: '<rect x="5" y="3" width="14" height="18" rx="1"/><path d="M9 7h2M13 7h2M9 11h2M13 11h2M9 15h2M13 15h2"/>',
    more: '<path d="M5 12h.01M12 12h.01M19 12h.01"/>', bolt: '<path d="M13 3 5 14h6l-1 7 8-11h-6z"/>', download: '<path d="M12 4v12m0 0-4-4m4 4 4-4M5 20h14"/>', warn: '<path d="M12 4 3 20h18z"/><path d="M12 10v4M12 17h.01"/>',
    wand: '<path d="m5 19 10-10M14 4l1 2 2 1-2 1-1 2-1-2-2-1 2-1zM18 12l.7 1.3L20 14l-1.3.7L18 16l-.7-1.3L16 14l1.3-.7z"/>', globe: '<circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3c3 3 3 15 0 18M12 3c-3 3-3 15 0 18"/>'
  }
  R.icon = (n, s) => `<svg class="ic" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"${s ? ` style="width:${s}px;height:${s}px"` : ''}>${ICONS[n] || ''}</svg>`
  R.mk = (state) => {
    const r = '<circle cx="7" cy="7" r="5.2" '
    const ck = '<path d="m4.6 7.2 1.7 1.7 3.1-3.6" fill="none" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" '
    const map = {
      todo: r + 'fill="none" stroke="#8c96a6" stroke-width="1.4"/>', prog: r + 'fill="none" stroke="#eaa43c" stroke-width="1.4"/><path d="M7 4.2a2.8 2.8 0 0 1 0 5.6z" fill="#eaa43c"/>',
      claimed: r + 'fill="#eaa43c22" stroke="#eaa43c" stroke-width="1.4"/>' + ck + 'stroke="#ffd88e"/>', done: r + 'fill="#3ddc97"/>' + ck + 'stroke="#08301f"/>', filed: r + 'fill="none" stroke="#8fa89d" stroke-width="1.4"/>' + ck + 'stroke="#8fa89d"/>',
      run: r + 'fill="none" stroke="#7c83f7" stroke-width="1.4"/><circle cx="7" cy="7" r="2" fill="#7c83f7"/>'
    }
    return `<span class="mk" aria-hidden="true"><svg viewBox="0 0 14 14">${map[state] || map.todo}</svg></span>`
  }

  /* ---------- the step seal ---------- */
  R.seal = (state, o) => {
    o = o || {}
    const ticks = Array.from({ length: 12 }, (_, i) => {
      const a = ((i * 30 - 90) * Math.PI) / 180, p = (r) => `${(12 + r * Math.cos(a)).toFixed(2)} ${(12 + r * Math.sin(a)).toFixed(2)}`
      return `<path d="M${p(6.7)}L${p(8.3)}" style="animation-delay:${(((i * 30 - 80) / 360 % 1 + 1) % 1 * 3.4 - 3.4).toFixed(2)}s"/>`
    }).join('')
    const cardinal = [0, 3, 6, 9].map((i) => ticks.split('/>').filter(Boolean)[i] + '/>').join('')
    return `<span class="seal-host"><svg class="seal" viewBox="-6 -6 36 36" data-state="${state}" ${o.next ? 'data-next="true"' : ''} ${o.complete ? 'data-complete="true"' : ''} ${o.arrive ? `data-arrive="${o.arrive}"` : ''} aria-hidden="true">
      <circle class="ripple" cx="12" cy="12" r="10.6"/><circle class="disc" cx="12" cy="12" r="10.6"/><circle class="rim" cx="12" cy="12" r="10.1"/>
      <circle class="ring" cx="12" cy="12" r="10.25" pathLength="100" transform="rotate(${state === 'CLAIMED' ? 25 : -90} 12 12)"/>
      <g class="ticks">${state === 'RUNNING' ? ticks : o.next && state === 'OPEN' ? cardinal : ''}</g>
      <g class="comet"><circle class="tail" cx="12" cy="12" r="10.25" pathLength="100"/><circle class="head" cx="12" cy="12" r="10.25" pathLength="100"/></g>
      <circle class="core" cx="12" cy="12" r="2.3"/><path class="check" pathLength="1" d="M7.5 12.4 10.4 15.3 16.4 9.1"/></svg></span>`
  }

  /* ---------- helpers ---------- */
  R.$ = (s, r) => (r || document).querySelector(s)
  R.$$ = (s, r) => [...(r || document).querySelectorAll(s)]
  R.project = (id) => R.projects.find((p) => p.id === id)
  R.company = (id) => R.companies.find((c) => c.id === id)
  R.task = (id) => R.tasks.find((t) => t.id === id)
  R.step = (id) => R.steps.find((s) => s.id === id)
  R.stepsOf = (tid) => R.steps.filter((s) => s.task === tid)
  R.notesOf = (tid) => R.notes.filter((n) => n.task === tid)
  R.anchorOf = (t) => `project:${R.project(t.p).label} task:${t.label}`
  R.avatar = (p, cls) => `<span class="avatar ${cls || ''}" style="background:${p.hue}">${(p.initial || p.name[0]).toUpperCase()}</span>`
  R.pAvatar = (p, cls) => R.avatar({ hue: p.hue, initial: p.name[0] }, cls)
  R.agoDays = (a) => { const m = /^(\d+)(m|h|d|w)/.exec(String(a)); if (!m) return 20; const n = +m[1]; return m[2] === 'm' ? n / 1440 : m[2] === 'h' ? n / 24 : m[2] === 'd' ? n : n * 7 }
  R.stepAgo = (s) => { const m = /^(\d+) (day|days)/.exec(s.ago || ''); return m ? +m[1] : 0 }
  R.tokenNum = (s) => { const m = /^([\d.]+)([kM]?)/.exec(s); return m ? +m[1] * (m[2] === 'M' ? 1e6 : m[2] === 'k' ? 1e3 : 1) : 0 }
  R.plural = (n, w) => `${n} ${w}${n === 1 ? '' : 's'}`
  R.esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]))

  R.counts = (t) => {
    const st = R.stepsOf(t.id).filter((s) => s.state !== 'DRAFT')
    return { total: st.length, done: st.filter((s) => s.state === 'DONE').length, claimed: st.filter((s) => s.state === 'CLAIMED').length, running: st.filter((s) => s.state === 'RUNNING').length, drafts: R.stepsOf(t.id).filter((s) => s.state === 'DRAFT').length }
  }
  R.kind = (t) => { const c = R.counts(t); return t.status === 'DONE' ? 'filed' : c.running || R.state.running === t.id ? 'run' : c.claimed || t.taskReview ? 'review' : t.status === 'TODO' ? 'todo' : 'prog' }
  R.KIND = { filed: ['Filed', 'filed'], run: ['Session working', 'run'], review: ['Awaiting review', 'review'], todo: ['To do', ''], prog: ['In progress', 'prog'] }
  R.markOf = (k) => ({ filed: 'filed', run: 'run', review: 'claimed', todo: 'todo', prog: 'prog' }[k])
  R.needsCount = (t) => { const c = R.counts(t); return c.claimed + (t.taskReview ? 1 : 0) }
  R.reviewItems = () => {
    const items = []
    R.steps.filter((s) => s.state === 'CLAIMED').forEach((s) => items.push({ key: s.id, kind: 'step', step: s, task: R.task(s.task), age: R.stepAgo(s) }))
    R.tasks.filter((t) => t.taskReview && t.status !== 'DONE').forEach((t) => items.push({ key: t.id, kind: 'task', task: t, age: R.agoDays(t.ago) }))
    return items.sort((a, b) => b.age - a.age)
  }
  R.reviewCount = () => R.reviewItems().length

  /* ---------- state ---------- */
  R.state = { queue: [{ id: 't-impr', state: 'running' }, { id: 't-open', state: 'queued' }, { id: 't-front', state: 'queued' }], running: 't-impr', open: 't-metric', view: 'needs', mode: 'ledger', scope: { co: null, p: null }, q: '', sort: 'default', dense: false, filedOpen: {}, sheet: null, arrivals: {}, sel: {}, sessionsOpen: false, sessionLog: 4, logSel: 't-impr', hideDone: false, settings: { quick: false, auto: true, tabs: true, queue: 2, goal: 8, notify: true }, reveal: {} }
  R.arrive = (id, state) => (R.state.arrivals[id] = state)
  R.listeners = []

  /* ---------- toasts / pops / dialogs ---------- */
  R.toast = (msg, act) => {
    const box = R.$('#toasts'), el = document.createElement('div')
    el.className = 'toast'; el.innerHTML = `<span>${msg}</span>${act ? `<button class="btn sm">${act.label}</button>` : ''}`
    if (act) R.$('button', el).onclick = () => { act.run(); close() }
    const close = () => { el.classList.add('out'); setTimeout(() => el.remove(), 240) }
    while (box.children.length >= 3) box.firstChild.remove()
    box.appendChild(el); setTimeout(close, act ? 6500 : 3600)
  }
  R.closePop = () => R.$$('.pop').forEach((p) => p.remove())
  R.pop = (anchor, items, opts) => {
    R.closePop()
    const r = anchor.getBoundingClientRect(), el = document.createElement('div')
    el.className = 'pop'; el.style.top = r.bottom + 6 + 'px'; el.style.left = Math.min(r.left, innerWidth - 260) + 'px'
    if (opts && opts.right) { el.style.left = 'auto'; el.style.right = innerWidth - r.right + 'px' }
    el.innerHTML = items.map((i, n) => i === '-' ? '<hr style="border:0;border-top:1px solid var(--border);margin:4px 0">' : i.h ? `<div class="pgrp" style="padding:10px 10px 4px">${i.h}</div>` : `<button data-i="${n}" class="${i.on ? 'on' : ''}">${i.html}</button>`).join('')
    el.onclick = (e) => { const b = e.target.closest('button'); if (!b || b.dataset.i === undefined) return; R.closePop(); items[+b.dataset.i].run() }
    document.body.appendChild(el)
  }
  R.dialog = (o) => {
    const el = document.createElement('div')
    el.className = 'scrim'; el.style.alignItems = 'center'; el.style.paddingTop = '0'
    el.innerHTML = `<div class="dialog" role="dialog" aria-modal="true"><h2>${o.title}</h2><div class="dbody">${o.body}</div><div class="dfoot"><button class="btn quiet" data-x>Cancel</button><button class="btn primary" data-ok>${o.ok || 'Save'}</button></div></div>`
    const close = () => el.remove()
    el.onclick = (e) => { if (e.target === el || e.target.closest('[data-x]')) close() }
    R.$('[data-ok]', el).onclick = () => { if (o.run(el) !== false) close() }
    document.body.appendChild(el); const f = R.$('input,textarea', el); if (f) f.focus()
  }

  /* ---------- actions ---------- */
  R.acts = {}
  R.clip = (txt) => { try { navigator.clipboard.writeText(txt) } catch (e) {} R.toast('Copied <span class="mono" style="color:var(--anchor)">' + R.esc(txt.length > 48 ? txt.slice(0, 48) + '…' : txt) + '</span>') }
  R.accept = (sid) => {
    const s = R.step(sid); if (!s || s.state !== 'CLAIMED') return
    s.state = 'DONE'; R.arrive(sid, 'DONE'); R.bump = true; R.state.sel[s.task] = null
    R.toast(`Accepted step ${s.n}`, { label: 'Reopen', run: () => { s.state = 'CLAIMED'; R.arrive(sid, 'CLAIMED'); R.render() } })
    const c = R.counts(R.task(s.task)); if (c.total && c.done === c.total) setTimeout(() => R.toast('Every step of this task is done'), 700)
    R.render()
  }
  R.sendBack = (sid) => { const s = R.step(sid); if (!s) return; s.state = 'OPEN'; s.ago = ''; R.arrive(sid, 'OPEN'); R.bump = true; R.toast(`Sent step ${s.n} back to the session`, { label: 'Undo', run: () => { s.state = 'CLAIMED'; s.ago = 'just now'; R.render() } }); R.render() }
  R.markDone = (sid) => { const s = R.step(sid); s.state = 'DONE'; R.arrive(sid, 'DONE'); R.render() }
  R.reopen = (sid) => { const s = R.step(sid); s.state = 'OPEN'; R.arrive(sid, 'OPEN'); R.render() }
  R.promote = (sid) => { const s = R.step(sid); s.state = 'OPEN'; R.arrive(sid, 'OPEN'); R.toast('Step promoted to the checklist'); R.render() }
  R.acceptTask = (tid) => { const t = R.task(tid); t.taskReview = false; t.status = 'DONE'; t.filedNow = true; R.bump = true; R.toast(`Accepted “${R.esc(t.title)}” · filed`, { label: 'Undo', run: () => { t.taskReview = true; t.status = 'IN_PROGRESS'; t.filedNow = false; R.render() } }); R.render() }
  R.sendBackTask = (tid) => { const t = R.task(tid); t.taskReview = false; R.bump = true; R.toast('Sent back to the session'); R.render() }
  R.addStep = (tid, title) => { const n = R.stepsOf(tid).length + 1; const s = { id: 's' + Date.now(), task: tid, title, state: 'DRAFT', ago: '', n }; R.steps.push(s); R.arrive(s.id, 'DRAFT'); R.state.sel[tid] = s.id; R.render() }
  R.newTask = (p, title, desc) => { const t = { id: 'tn' + Date.now(), p, title, label: title.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '').slice(0, 40), status: 'TODO', ago: 'now', tokens: '0', desc: desc || '', fresh: true }; R.tasks.unshift(t); R.state.view = 'open'; R.toast('Task created in ' + R.project(p).name); R.render() }

  /* a session: the running step is claimed after a while, which is the loop this product exists for */
  R.resume = (tid) => {
    const t = R.task(tid), st = R.stepsOf(tid)
    if (R.state.running && R.state.running !== tid) return R.toast('One session is already working. Open it from the dock.')
    R.state.running = tid; R.state.logSel = tid; R.state.sessionLog = 1; if (!R.state.queue.find((q) => q.id === tid)) R.state.queue.unshift({ id: tid, state: 'running' }); R.state.queue.forEach((q) => (q.state = q.id === tid ? 'running' : 'queued'))
    let cur = st.find((s) => s.state === 'RUNNING') || st.find((s) => s.state === 'OPEN')
    if (cur) { cur.state = 'RUNNING'; R.arrive(cur.id, 'RUNNING') }
    R.toast('Session started on <b>' + R.esc(t.title) + '</b>'); R.render(); R.runSession(tid)
  }
  R.runSession = (tid) => {
    clearInterval(R._sim)
    R._sim = setInterval(() => {
      R.state.sessionLog++
      if (R.state.sessionLog >= R.sessionsLog.length) {
        clearInterval(R._sim)
        const cur = R.stepsOf(tid).find((s) => s.state === 'RUNNING')
        if (cur) { cur.state = 'CLAIMED'; cur.ago = 'just now'; R.arrive(cur.id, 'CLAIMED'); R.bump = true; R.toast(`Session claimed step ${cur.n}. It waits for your review.`, { label: 'Review', run: () => (location.hash = '#/review/' + cur.id) }) }
        R.state.running = null; R.state.queue = R.state.queue.filter((q) => q.id !== tid); R.render(); return
      }
      R.render({ soft: true })
    }, 1500)
  }

  /* ---------- router ---------- */
  R.routes = {}
  R.route = () => { const h = (location.hash || '#/work').slice(2).split('/'); return { name: h[0] || 'work', a: h[1], b: h[2], c: h[3], hash: location.hash } }
  R.inputs = {}
  document.addEventListener('input', (e) => { const k = e.target.dataset && e.target.dataset.input; if (k && R.inputs[k]) R.inputs[k](e.target.value, e.target) })
  R.navTab = (r) => ({ task: 'work', work: 'work', review: 'review', projects: 'projects', project: 'projects', companies: 'projects', connections: 'projects', time: 'time', library: 'library', security: 'security', settings: 'settings' }[r.name] || 'work')

  R.screenLabel = (r) => ({ work: r.a === 'board' ? 'Work · Board' : 'Work', task: 'Work · Task', review: 'Review', projects: r.a ? 'Projects · ' + ((R.project(r.a) || {}).name || '') : 'Projects', companies: 'Companies', connections: 'Connections', time: r.a === 'calendar' ? 'Time · Calendar' : 'Time · Report', library: r.a === 'claude' ? 'Library · Claude files' : r.b ? 'Library · Diagram' : 'Library · Diagrams', security: 'Security', settings: 'Settings' }[r.name] || 'Work')
  R.menuGroups = () => [
    ['Work', [['Work', '#/work', 'grid', ''], ['Review', '#/review', 'check', R.reviewCount()]]],
    ['Projects', [['Projects', '#/projects', 'folder', ''], ['Companies', '#/companies', 'building', ''], ['Connections', '#/connections', 'link', '']]],
    ['Time', [['Report', '#/time/report', 'clock', ''], ['Calendar', '#/time/calendar', 'calendar', '']]],
    ['Library', [['Diagrams', '#/library/diagrams', 'flow', ''], ['Claude files', '#/library/claude', 'book', '']]],
    ['Security', [['Advisories', '#/security', 'shield', '']]],
    ['Settings', [['Settings', '#/settings', 'settings', '']]]
  ]
  R.queueItems = () => R.state.queue.map((q) => ({ q, t: R.task(q.id), c: R.counts(R.task(q.id)) }))
  R.topbar = () => {
    const r = R.route(), n = R.reviewCount(), on = R.navTab(r), qs = R.queueItems(), running = qs.filter((x) => x.q.state === 'running').length
    let chips = '<span>project:vega task:setup</span>'
    if (r.name === 'task' && R.task(r.a)) { const t = R.task(r.a); chips = `<span class="chip">project:${R.project(t.p).label}</span><span class="dim">›</span><span class="chip trunc">task:${t.label}</span>` }
    else if (r.name === 'projects' && r.a && R.project(r.a)) chips = `<span class="chip">project:${R.project(r.a).label}</span>`
    const dots = qs.slice(0, 4).map((x) => `<i class="qd ${x.q.state}"></i>`).join('')
    return `<header class="top"><div class="top-l"><a class="logo" href="#/work" aria-label="Rekall"></a>
      <button class="menu-key" data-act="menu" aria-haspopup="menu" aria-label="Menu, on ${R.screenLabel(r)}"><span class="burger"><i></i><i></i><i></i></span><span class="menu-name">${R.screenLabel(r)}</span>${n ? `<span class="badge ${R.bump ? 'bump' : ''}" id="rvbadge">${n}</span>` : ''}${R.icon('chevron', 14)}</button>
      <div class="hist"><button class="ibtn" data-act="back" aria-label="Back">${R.icon('left')}</button><button class="ibtn" data-act="fwd" aria-label="Forward">${R.icon('right')}</button></div></div>
      <button class="locbar" data-act="palette" aria-label="Search by anchor"><b>/rk</b>${chips}<kbd>⌘K</kbd></button>
      <div class="top-r"><div class="cap"><button data-act="usage" title="Usage this window"><span class="ring" style="--p:3"></span><span class="num">3%</span><span class="dim">4h 55m</span></button><button data-act="queue" title="Run queue"><span class="qdots">${dots}</span><span>${qs.length ? (running ? running + ' running' : qs.length + ' queued') : 'Queue empty'}</span></button><a class="rv" href="#/review"><b id="rvcount">${n}</b> to review</a></div>
      <button class="ibtn" data-act="tags" aria-label="Tags" title="Tags">${R.icon('tag')}</button><a class="ibtn ${on === 'settings' ? 'on' : ''}" href="#/settings" aria-label="Settings" title="Settings">${R.icon('settings')}</a></div></header>`
  }
  R.dock = () => {
    const run = R.state.running ? R.task(R.state.running) : null
    return `<button data-act="sessions"><i class="orb ${run ? '' : 'idle'}"></i>${run ? '1 in progress' : 'No session working'}</button><span class="sep"></span><button data-act="sessions">1 session</button>`
  }
  R.sessionsPanel = () => {
    if (!R.state.sessionsOpen) return ''
    const t = R.task(R.state.logSel) || R.task('t-impr'), running = R.state.running === t.id
    const n = running ? R.state.sessionLog : R.sessionsLog.length
    const lines = R.sessionsLog.slice(0, n).map((l, i) => (i === 0 ? `<div><span class="p">$</span> ${R.esc(l)}</div>` : /^(test|Claiming)/.test(l) ? `<div><span class="ok">${R.esc(l)}</span></div>` : /^(Editing|Reading|cargo)/.test(l) ? `<div><span class="c">${R.esc(l)}</span></div>` : `<div><span class="s">${R.esc(l)}</span></div>`)).join('')
    return `<div class="sessions"><div class="sh"><span>Sessions</span><span class="pill ${running ? 'run' : ''}">${running ? 'working' : 'ended'}</span><span class="mut trunc" style="font-weight:400">${R.esc(t.title)}</span><span class="sp"></span><a class="btn sm quiet" href="#/task/${t.id}">Open task</a><button class="ibtn" data-act="sessions" aria-label="Close">${R.icon('close')}</button></div><div class="term" id="term">${lines}${running ? '<span class="caret"></span>' : ''}</div></div>`
  }

  let lastRoute = ''
  R.render = (o) => {
    o = o || {}
    const r = R.route(), view = R.routes[r.name] || R.routes.work
    const act = document.activeElement, aid = act && act.id, caret = act && act.selectionStart
    const sc = R.$$('.scroll').map((s) => s.scrollTop), same = lastRoute === r.hash
    R.$('#app').innerHTML = R.topbar() + '<main class="main" id="main"></main>'
    R.$('#main').innerHTML = view(r)
    R.$('#dock').innerHTML = R.dock()
    R.$('#sessionhost').innerHTML = R.sessionsPanel()
    if (same) R.$$('.scroll').forEach((s, i) => (s.scrollTop = sc[i] || 0))
    if (aid) { const e = document.getElementById(aid); if (e) { e.focus(); try { e.setSelectionRange(caret, caret) } catch (x) {} } }
    const term = R.$('#term'); if (term) term.scrollTop = term.scrollHeight
    lastRoute = r.hash; R.bump = false
    requestAnimationFrame(() => requestAnimationFrame(() => (R.state.arrivals = {})))
    if (R.mount && R.mount[r.name]) R.mount[r.name](r)
    document.title = 'Rekall redesign'
  }
  R.mount = {}

  /* ---------- palette ---------- */
  R.openPalette = () => {
    const el = document.createElement('div'); el.className = 'scrim'; el.id = 'palette'
    el.innerHTML = `<div class="pal"><div class="pal-in">${R.icon('search')}<input id="pq" placeholder="Search tasks, projects, notes, screens, or an anchor" autocomplete="off" aria-label="Search"></div><div class="pal-list" id="pl"></div><div class="pal-foot"><span><kbd>↑</kbd> <kbd>↓</kbd> walk</span><span><kbd>⏎</kbd> open</span><span><kbd>esc</kbd> close</span></div></div>`
    document.body.appendChild(el)
    const input = R.$('#pq', el); let items = [], cur = 0
    const paint = () => {
      const q = input.value.toLowerCase().replace(/^\/rk\s*/, ''), has = (s) => !q || s.toLowerCase().includes(q)
      const screens = [['Work', '#/work', 'grid'], ['Review', '#/review', 'check'], ['Projects', '#/projects', 'folder'], ['Companies', '#/companies', 'building'], ['Connections', '#/connections', 'link'], ['Time report', '#/time/report', 'clock'], ['Calendar', '#/time/calendar', 'calendar'], ['Diagrams', '#/library/diagrams', 'flow'], ['Claude files', '#/library/claude', 'book'], ['Security', '#/security', 'shield'], ['Settings', '#/settings', 'settings']]
      const g = []
      g.push(['Go to', screens.filter((s) => has(s[0])).map((s) => ({ t: s[0], r: '', h: s[1], i: R.icon(s[2], 15) }))])
      g.push(['Projects', R.projects.filter((p) => has(p.name + ' project:' + p.label)).slice(0, 5).map((p) => ({ t: p.name, r: 'project:' + p.label, h: '#/projects/' + p.id, i: R.pAvatar(p, 'sm') }))])
      g.push(['Tasks', R.tasks.filter((t) => has(t.title + ' ' + t.label)).slice(0, 8).map((t) => ({ t: t.title, r: R.needsCount(t) ? R.needsCount(t) + ' to review' : t.ago, h: '#/task/' + t.id, i: R.mk(R.markOf(R.kind(t))) }))])
      g.push(['Notes', R.notes.filter((n) => has(n.title)).slice(0, 4).map((n) => ({ t: n.title, r: R.task(n.task).label.slice(0, 22), h: '#/task/' + n.task + '/notes/' + n.id, i: R.icon('note', 15) }))])
      items = []; let html = ''
      g.forEach(([name, rows]) => { if (!rows.length) return; html += `<div class="pgrp">${name}</div>`; rows.forEach((x) => { html += `<button class="prow ${items.length === cur ? 'cur' : ''}" data-n="${items.length}">${x.i}<span class="trunc">${R.esc(x.t)}</span><span class="r">${R.esc(x.r)}</span></button>`; items.push(x) }) })
      R.$('#pl', el).innerHTML = html || '<div class="empty" style="margin:14px">Nothing matches.</div>'
      const c = R.$('.prow.cur', el); if (c) c.scrollIntoView({ block: 'nearest' })
    }
    const go = (n) => { const x = items[n]; if (!x) return; el.remove(); location.hash = x.h }
    input.oninput = () => { cur = 0; paint() }
    input.onkeydown = (e) => { if (e.key === 'ArrowDown') { cur = Math.min(items.length - 1, cur + 1); paint(); e.preventDefault() } else if (e.key === 'ArrowUp') { cur = Math.max(0, cur - 1); paint(); e.preventDefault() } else if (e.key === 'Enter') go(cur) }
    el.onclick = (e) => { if (e.target === el) el.remove(); const b = e.target.closest('.prow'); if (b) go(+b.dataset.n) }
    paint(); input.focus()
  }

  /* ---------- global events ---------- */
  document.addEventListener('click', (e) => {
    if (!e.target.closest('.pop')) R.closePop()
    const el = e.target.closest('[data-act]'); if (!el) return
    const a = R.acts[el.dataset.act]; if (a) { e.preventDefault(); e.stopPropagation(); a(el, e) }
  })
  document.addEventListener('keydown', (e) => {
    const typing = /INPUT|TEXTAREA|SELECT/.test((e.target.tagName || ''))
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') { e.preventDefault(); if (!R.$('#palette')) R.openPalette(); return }
    if (e.key === 'Escape') { const p = R.$('#palette') || R.$('.scrim'); if (p) { p.remove(); return } if (R.state.sheet) { R.state.sheet = null; R.render(); return } if (R.state.sessionsOpen) { R.state.sessionsOpen = false; R.render() } return }
    if (typing || e.metaKey || e.ctrlKey) return
    if (R.onKey) R.onKey(e)
  })
  R.acts.palette = () => R.openPalette()
  R.acts.back = () => history.back()
  R.acts.fwd = () => history.forward()
  R.acts.menu = (el) => {
    const r = R.route(), cur = R.screenLabel(r), items = []
    R.menuGroups().forEach(([g, rows]) => { items.push({ h: g }); rows.forEach(([l, h, ic, c]) => items.push({ html: `${R.icon(ic, 15)}<span style="flex:1">${l}</span>${c ? `<span class="badge">${c}</span>` : ''}`, on: h.slice(2).split('/')[0] === r.name || (h === '#/work' && r.name === 'task') || (h === '#/projects' && r.name === 'projects'), run: () => (location.hash = h) })) })
    R.pop(el, items); const p = R.$('.pop'); p.style.minWidth = '260px'
  }
  R.acts.usage = (el) => R.pop(el, [{ h: 'Usage this window' }, { html: '<span style="flex:1">Tokens</span><span class="mono dim">3% of the limit</span>', run: () => {} }, { html: '<span style="flex:1">Resets in</span><span class="mono dim">4h 55m</span>', run: () => {} }, '-', { html: `<span style="flex:1">Open the time report</span>${R.icon('right', 14)}`, run: () => (location.hash = '#/time/report') }])
  R.acts.queue = (el) => {
    const qs = R.queueItems(), items = [{ h: `Run queue · ${R.state.settings.queue} at a time` }]
    qs.forEach((x) => items.push({ html: `${R.mk(x.q.state === 'running' ? 'run' : 'todo')}<span class="trunc" style="flex:1;max-width:230px">${R.esc(x.t.title)}</span><span class="mono dim" style="font-size:var(--t-caption)">${x.c.total ? x.c.done + '/' + x.c.total : x.q.state}</span>`, run: () => (location.hash = '#/task/' + x.t.id) }))
    if (!qs.length) items.push({ html: '<span class="dim">Nothing queued</span>', run: () => {} })
    items.push('-', { html: `${R.icon('plus', 14)}<span>Queue a task</span>`, run: () => R.toast('Pick a task, then choose Queue') }, { html: `${R.icon('settings', 14)}<span>Queue settings</span>`, run: () => (location.hash = '#/settings/sessions') })
    R.pop(el, items); R.$('.pop').style.minWidth = '320px'
  }
  R.acts.sessions = () => { R.state.sessionsOpen = !R.state.sessionsOpen; R.render() }
  R.acts.copy = (el) => R.clip(el.dataset.v)
  R.acts.accept = (el) => R.accept(el.dataset.id)
  R.acts.sendback = (el) => R.sendBack(el.dataset.id)
  R.acts.markdone = (el) => R.markDone(el.dataset.id)
  R.acts.reopen = (el) => R.reopen(el.dataset.id)
  R.acts.promote = (el) => R.promote(el.dataset.id)
  R.acts.acceptTask = (el) => R.acceptTask(el.dataset.id)
  R.acts.sendbackTask = (el) => R.sendBackTask(el.dataset.id)
  R.acts.resume = (el) => R.resume(el.dataset.id)
  R.acts.term = (el) => R.toast('Opening a terminal in <span class="mono">' + R.esc(el.dataset.v || '') + '</span>')
  R.acts.tags = () => R.dialog({ title: 'Tags', ok: 'Done', body: `<p class="mut" style="font-size:var(--t-small)">Tags label tasks across projects.</p><div style="display:grid;gap:6px">${R.tags.map((t) => `<div class="list-row" style="grid-template-columns:14px 1fr auto;border:1px solid var(--border);border-radius:var(--r-control);min-height:40px"><span class="dot" style="background:${t.c}"></span><span>${t.n}</span><span class="dim mono" style="font-size:var(--t-caption)">${R.tasks.filter((_, i) => i % 3 === R.tags.indexOf(t) % 3).length} tasks</span></div>`).join('')}</div><label class="lbl">New tag<span class="field"><input placeholder="Name"></span></label>`, run: () => true })

  addEventListener('hashchange', () => { R.closePop(); R.state.sheet = null; R.render() })
  addEventListener('DOMContentLoaded', () => { document.body.insertAdjacentHTML('beforeend', '<div id="dock"></div><div id="sessionhost"></div><div id="toasts"></div>'); R.render(); setTimeout(() => R.state.running && R.runSession(R.state.running), 4000) })
})()
