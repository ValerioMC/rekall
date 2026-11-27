const providerPaths = {
  claude:
    "m4.7144 15.9555 4.7174-2.6471.079-.2307-.079-.1275h-.2307l-.7893-.0486-2.6956-.0729-2.3375-.0971-2.2646-.1214-.5707-.1215-.5343-.7042.0546-.3522.4797-.3218.686.0608 1.5179.1032 2.2767.1578 1.6514.0972 2.4468.255h.3886l.0546-.1579-.1336-.0971-.1032-.0972L6.973 9.8356l-2.55-1.6879-1.3356-.9714-.7225-.4918-.3643-.4614-.1578-1.0078.6557-.7225.8803.0607.2246.0607.8925.686 1.9064 1.4754 2.4893 1.8336.3643.3035.1457-.1032.0182-.0728-.164-.2733-1.3539-2.4467-1.445-2.4893-.6435-1.032-.17-.6194c-.0607-.255-.1032-.4674-.1032-.7285L6.287.1335 6.6997 0l.9957.1336.419.3642.6192 1.4147 1.0018 2.2282 1.5543 3.0296.4553.8985.2429.8318.091.255h.1579v-.1457l.1275-1.706.2368-2.0947.2307-2.6957.0789-.7589.3764-.9107.7468-.4918.5828.2793.4797.686-.0668.4433-.2853 1.8517-.5586 2.9021-.3643 1.9429h.2125l.2429-.2429.9835-1.3053 1.6514-2.0643.7286-.8196.85-.9046.5464-.4311h1.0321l.759 1.1293-.34 1.1657-1.0625 1.3478-.8804 1.1414-1.2628 1.7-.7893 1.36.0729.1093.1882-.0183 2.8535-.607 1.5421-.2794 1.8396-.3157.8318.3886.091.3946-.3278.8075-1.967.4857-2.3072.4614-3.4364.8136-.0425.0304.0486.0607 1.5482.1457.6618.0364h1.621l3.0175.2247.7892.522.4736.6376-.079.4857-1.2142.6193-1.6393-.3886-3.825-.9107-1.3113-.3279h-.1822v.1093l1.0929 1.0686 2.0035 1.8092 2.5075 2.3314.1275.5768-.3218.4554-.34-.0486-2.2039-1.6575-.85-.7468-1.9246-1.621h-.1275v.17l.4432.6496 2.3436 3.5214.1214 1.0807-.17.3521-.6071.2125-.6679-.1214-1.3721-1.9246L14.38 17.959l-1.1414-1.9428-.1397.079-.674 7.2552-.3156.3703-.7286.2793-.6071-.4614-.3218-.7468.3218-1.4753.3886-1.9246.3157-1.53.2853-1.9004.17-.6314-.0121-.0425-.1397.0182-1.4328 1.9672-2.1796 2.9446-1.7243 1.8456-.4128.164-.7164-.3704.0667-.6618.4008-.5889 2.386-3.0357 1.4389-1.882.929-1.0868-.0062-.1579h-.0546l-6.3385 4.1164-1.1293.1457-.4857-.4554.0608-.7467.2307-.2429 1.9064-1.3114Z",
  codex:
    "M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z",
};

const lines = {
  left: "m10 4-4 4 4 4",
  right: "m6 4 4 4-4 4",
  up: "m4 10 4-4 4 4",
  down: "m4 6 4 4 4-4",
  close: "m4 4 8 8m0-8-8 8",
  plus: "M8 3v10M3 8h10",
  menu: "M3 4h10M3 8h10M3 12h7",
  folder:
    "M2 5a1 1 0 0 1 1-1h3l2 2h5a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1z",
  search:
    "M10.2 10.2 13 13m-1.6-6.2A4.4 4.4 0 1 1 2.6 6.8a4.4 4.4 0 0 1 8.8 0Z",
  check: "m3.5 8 3 3 6-6",
  "check-circle": "M14 8a6 6 0 1 1-12 0 6 6 0 0 1 12 0ZM5.4 8l1.7 1.7 3.5-3.5",
  queue: "M3 4h8M3 8h6m-6 4h6m3-5 2 2-2 2",
  list: "M5 4h8M5 8h8m-8 4h8M2 4h.1M2 8h.1M2 12h.1",
  gauge: "M2.5 11a5.5 5.5 0 1 1 11 0v1h-11zM8 9l2-2",
  settings:
    "M6.5 2h3l.3 1.7 1.1.7 1.7-.6 1.5 2.6L13 7.5v1l1.1 1.2-1.5 2.6-1.7-.6-1.1.7-.3 1.6h-3l-.3-1.6-1.1-.7-1.7.6-1.5-2.6L3 8.5v-1L1.9 6.4l1.5-2.6 1.7.6 1.1-.7zm3.3 6a1.8 1.8 0 1 1-3.6 0 1.8 1.8 0 0 1 3.6 0Z",
  terminal:
    "M3 4h10a1 1 0 0 1 1 1v6a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1Zm2 2 2 2-2 2m4 0h2",
  sliders: "M2 4h12M2 8h12M2 12h12M6 2.5v3M10 6.5v3M5 10.5v3",
  info: "M14 8a6 6 0 1 1-12 0 6 6 0 0 1 12 0ZM8 7v4m0-6h.01",
  copy: "M6 6h7v7H6zm4-3H3v7",
  play: "m6 4 6 4-6 4z",
  ring: "M13 8A5 5 0 1 1 3 8a5 5 0 0 1 10 0Z",
};

const providers = { claude: "Claude Code", codex: "Codex" };
const effortLabels = {
  default: "Account default",
  low: "Low",
  medium: "Medium",
  high: "High",
  xhigh: "Extra high",
  max: "Max",
};
const claudeModels = {
  default: "Account default",
  sonnet: "Sonnet",
  fable: "Fable",
  opus: "Opus",
  haiku: "Haiku",
};
const codexModels = {
  default: { label: "Account default", efforts: ["default"] },
  "gpt-5.4": {
    label: "GPT-5.4",
    efforts: ["default", "low", "medium", "high", "xhigh"],
  },
  "gpt-5.4-mini": {
    label: "GPT-5.4 Mini",
    efforts: ["default", "medium", "high"],
  },
};
const tasks = {
  "RV-212": {
    title: "Follow up of differentiate execution",
    project: "Rekall vR",
    route: "task",
    steps: 5,
    label: "console",
  },
  "RV-216": {
    title: "Review session launch wording",
    project: "Rekall vR",
    route: "unplanned",
    steps: 0,
    label: "console",
  },
  "AT-18": {
    title: "Prepare the next release",
    project: "Atlas",
    route: "atlas-task",
    steps: 0,
    label: "release",
  },
};
const planSteps = [
  "Show the complete console mockup for review",
  "Save the model and effort chosen for each queue entry",
  "Put the provider choice and navigation in the top bar",
  "Confirm queue launch settings and show them on the task row",
  "Verify the console and run queue end to end",
];
const decisions = [
  {
    question: "Which provider should a queued task start with?",
    answer: "",
    detail:
      "The provider shown when the form opens is the starting point for this entry. You can choose another provider before confirming.",
  },
  {
    question:
      "Which provider should Refine use when a session is already open?",
    answer: "The live session’s provider",
    detail: "Refine goes to the session that is already open for this task.",
  },
  {
    question: "What should the queue retain for each task?",
    answer: "Provider, model and effort",
    detail: "The queue row uses the choices saved at confirmation.",
  },
];
const byId = (id) => document.getElementById(id);
const queue = [];
const sessions = new Map();
let globalProvider = "codex";
let currentView = "task";
let currentHistory = 2;
let lastHistory = 2;
let activeSession = null;
let launchDraft = null;
let launchReturn = null;
let catalogState = "idle";
let catalogTimer;
let announceTimer;
let queueRunning = false;
let queueMatches = [];
let matchIndex = 0;

function glyph(name) {
  const path = providerPaths[name];
  if (path)
    return `<svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="${path}"/></svg>`;
  return `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="${lines[name]}"/></svg>`;
}

function icon(name, classes = "") {
  return `<i data-icon="${name}" class="${classes}">${glyph(name)}</i>`;
}

function drawIcons(container) {
  for (const element of container.querySelectorAll("i[data-icon]:empty")) {
    element.innerHTML = glyph(element.dataset.icon);
  }
}

function routeHref(view) {
  return `#${view}`;
}

function taskCode(view = currentView) {
  if (["task", "decisions", "plan"].includes(view)) return "RV-212";
  return Object.keys(tasks).find((code) => tasks[code].route === view);
}

function renderHistory() {
  byId("back").disabled = currentHistory === 0;
  byId("forward").disabled = currentHistory === lastHistory;
}

function navigate(view) {
  closePopovers();
  if (currentView === view) return;
  history.pushState({ view, index: ++currentHistory }, "", routeHref(view));
  lastHistory = currentHistory;
  currentView = view;
  renderScreen();
}

function onHistory(event) {
  if (!event.state?.view) return;
  currentHistory = event.state.index;
  currentView = event.state.view;
  closePopovers();
  renderScreen();
}

function seedHistory() {
  history.replaceState(
    { view: "projects", index: 0 },
    "",
    routeHref("projects"),
  );
  history.pushState({ view: "rekall", index: 1 }, "", routeHref("rekall"));
  history.pushState({ view: "task", index: 2 }, "", routeHref("task"));
}

function announce(message) {
  const toast = byId("announcement");
  clearTimeout(announceTimer);
  toast.textContent = message;
  announceTimer = setTimeout(() => {
    toast.textContent = "";
  }, 3500);
}

function unavailable(provider) {
  return provider === "codex" && byId("missing-codex").checked;
}

function effectiveProvider(code) {
  return sessions.get(code)?.provider ?? globalProvider;
}

function renderGlobalProvider() {
  const mark = byId("provider-mark");
  mark.dataset.icon = globalProvider;
  mark.innerHTML = glyph(globalProvider);
  byId("provider-label").textContent = providers[globalProvider];
  byId("provider-key").dataset.missing = unavailable(globalProvider);
  byId("provider-key").setAttribute(
    "aria-label",
    `Session provider · ${providers[globalProvider]}${unavailable(globalProvider) ? " unavailable" : ""}`,
  );
  const entries = Object.entries(providers).map(([provider, name]) => {
    const chosen = provider === globalProvider;
    const missing = unavailable(provider);
    return `<button role="menuitemradio" tabindex="-1" data-provider="${provider}" aria-checked="${chosen}" aria-disabled="${missing}" ${chosen ? "autofocus" : ""}>${icon(provider)}<span>${name}${missing ? "<small>CLI is not installed</small>" : ""}</span>${chosen ? icon("check", "tail") : ""}</button>`;
  });
  byId("provider-menu").innerHTML =
    `<p class="menu-caption">Session provider</p>${entries.join("")}${unavailable("codex") ? '<div class="menu-divider"></div><button role="menuitem" tabindex="-1" data-route="settings">Agent setup in Settings</button>' : ""}`;
}

function selectGlobal(provider) {
  if (unavailable(provider)) return;
  globalProvider = provider;
  closePopovers();
  renderGlobalProvider();
  renderRunMenu();
  byId("provider-key").focus();
  announce(`${providers[provider]} selected for new sessions`);
}

function taskActions(code) {
  const live = sessions.get(code);
  const buttonTone = currentView === "decisions" ? "secondary" : "primary";
  return `<div class="task-actions">
    ${live ? `<span class="live-note">${icon(live.provider)}${providers[live.provider]} session open</span>` : ""}
    <span class="split">
      <button class="button small ${buttonTone}" data-session="Run here">${icon("terminal")}${live ? "Open session" : "Run here"}</button>
      <button class="button small ${buttonTone}" id="run-arrow" popovertarget="run-menu" aria-haspopup="menu" aria-expanded="false" aria-label="More ways to run" title="More ways to run">${icon("down")}</button>
    </span>
  </div>`;
}

function taskTabs(code) {
  if (code !== "RV-212")
    return '<nav class="tabs" aria-label="Task"><a aria-current="page" href="#description">Description</a></nav>';
  return `<nav class="tabs" aria-label="Task">
    <a href="#task"${currentView === "task" ? ' aria-current="page"' : ""}>Description</a>
    <a href="#decisions"${currentView === "decisions" ? ' aria-current="page"' : ""}>Decisions <span class="count">3</span></a>
    <a href="#plan"${currentView === "plan" ? ' aria-current="page"' : ""}>Plan <span class="count">5</span></a>
  </nav>`;
}

function taskRail(code, task) {
  return `<aside class="rail" aria-label="Task properties">
    <span class="pill progress">In progress</span>
    <div class="identity"><code class="chip">task:${code}</code><button class="icon-button" data-copy="${code}" aria-label="Copy task anchor" title="Copy task anchor">${icon("copy")}</button></div>
    <section><h2>Labels</h2><span class="tag">${task.label}</span></section>
    <section><h2>Plan</h2><p>${task.steps ? `0 of ${task.steps} steps done` : "No checklist"}</p></section>
  </aside>`;
}

function taskContent(code) {
  if (currentView === "decisions") return decisionsPage();
  if (currentView === "plan") {
    return `<section class="page-copy"><h2>Plan</h2><ul class="steps">${planSteps.map((step) => `<li>${icon("ring")}${step}</li>`).join("")}</ul></section>`;
  }
  const copy = {
    "RV-212":
      "<p>Choose Claude Code or Codex in the console header and use that choice when opening a new session from any task.</p><p>When you send a task to the run queue, choose the provider, model and effort for that entry. Confirm to save the settings and show them on the queue row.</p>",
    "RV-216":
      "<p>Review the language used to open and resume sessions in the console.</p>",
    "AT-18":
      "<p>Prepare the release checklist and agree on what needs to be verified before shipping.</p>",
  };
  return `<section class="page-copy"><h2>Description</h2>${copy[code]}</section>`;
}

function renderTask(code) {
  const task = tasks[code];
  const projectRoute = task.project === "Atlas" ? "atlas" : "rekall";
  return `<header class="task-heading">
    <div class="heading-line"><a class="breadcrumb" href="#${projectRoute}">VFORGE · ${task.project}</a>${taskActions(code)}</div>
    <h1>${code} · ${task.title}</h1>
  </header>
  <div class="task-layout">${taskRail(code, task)}<div>${taskTabs(code)}${taskContent(code)}</div></div>`;
}

function decisionsPage() {
  return `<section aria-label="Decisions">
    <div class="decision-band">
      <div class="decision-progress"><div class="tube"><span style="width: 66.67%"></span></div><span class="muted"><strong>2 of 3</strong> answered</span></div>
      <p class="decision-status">Updates as you talk in your session.</p>
      <button class="button small primary" data-session="Refine">Refine</button>
    </div>
    <div class="find"><label class="field">${icon("search")}<input type="search" id="decision-query" aria-label="Search decisions" placeholder="Search decisions"></label><span class="muted" id="decision-count" role="status"></span></div>
    <div class="decision-list" id="decision-rows"></div>
  </section>`;
}

function renderDecisions() {
  const term = byId("decision-query").value.trim().toLowerCase();
  const matches = decisions.filter((decision) =>
    Object.values(decision).some((value) => value.toLowerCase().includes(term)),
  );
  byId("decision-count").textContent = term
    ? `${matches.length} of ${decisions.length}`
    : "";
  byId("decision-rows").innerHTML = matches.length
    ? `<div class="decision-columns"><span></span><span>Decision</span><span>Answer</span><span>Asked in</span></div>
    ${decisionGroup(
      "Open",
      matches.filter((decision) => !decision.answer),
    )}
    ${decisionGroup(
      "Answered",
      matches.filter((decision) => decision.answer),
    )}`
    : '<p class="muted">No matching decisions.</p>';
}

function decisionGroup(title, entries) {
  if (!entries.length) return "";
  return `<h3 class="decision-group">${title} <span class="count">${entries.length}</span></h3>${entries
    .map(
      (decision) => `<details class="decision-row"><summary>
    <span class="seal ${decision.answer ? "answered" : ""}">${decision.answer ? icon("check") : ""}</span>
    <strong>${decision.question}</strong><span class="muted">${decision.answer || "Waiting for an answer"}</span><span class="subtle">RV-212</span>
  </summary><p>${decision.detail}</p></details>`,
    )
    .join("")}`;
}

function taskLink(code) {
  return `<a href="#${tasks[code].route}"><code class="chip">${code}</code><span>${tasks[code].title}</span></a>`;
}

function projectPage(title) {
  const codes = Object.keys(tasks).filter(
    (code) => tasks[code].project === title,
  );
  return `<section class="project-page"><h1>${title}</h1><h2>Tasks</h2><div class="project-table">${codes.map(taskLink).join("")}</div></section>`;
}

function settingsPage() {
  return `<section class="project-page"><h1>Settings</h1><h2>Sessions</h2>
    <p class="muted">Choose the provider for new console sessions in the top bar.</p>
    <div class="project-table">
      <p class="settings-row">${icon("claude")} Claude Code <span class="pill">Available</span></p>
      <p class="settings-row">${icon("codex")} Codex <span class="pill ${unavailable("codex") ? "progress" : ""}">${unavailable("codex") ? "CLI is not installed" : "Available"}</span></p>
    </div>
    ${unavailable("codex") ? '<p class="muted setup-note">Install the Codex CLI and reopen Rekall to use it for sessions.</p>' : ""}
  </section>`;
}

function screenContent(code) {
  if (code) return renderTask(code);
  const pages = {
    projects: () =>
      '<section class="project-page"><h1>Projects</h1><div class="project-table"><a href="#rekall">Rekall vR <span class="muted">VFORGE</span></a><a href="#atlas">Atlas <span class="muted">VFORGE</span></a></div></section>',
    rekall: () => projectPage("Rekall vR"),
    atlas: () => projectPage("Atlas"),
    review: () =>
      `<section class="project-page"><h1>Review</h1><div class="project-table">${taskLink("RV-212")}${taskLink("RV-216")}</div></section>`,
    settings: settingsPage,
  };
  return pages[currentView]();
}

function renderScreen() {
  const code = taskCode();
  byId("screen").innerHTML = screenContent(code);
  byId("current-project").textContent =
    tasks[code]?.project ??
    { atlas: "Atlas", rekall: "Rekall vR" }[currentView] ??
    "Projects";
  const section = code
    ? "Tasks"
    : ({ review: "Review", settings: "Settings" }[currentView] ?? "Projects");
  byId("sections-key").querySelector("span").textContent = section;
  byId("sections-key").setAttribute("aria-label", `Section menu · ${section}`);
  byId("projects-key").setAttribute(
    "aria-label",
    `Switch project · ${byId("current-project").textContent}`,
  );
  byId("session-dock").hidden = sessions.size === 0;
  byId("session-dock").querySelector("span").textContent =
    `${sessions.size} ${sessions.size === 1 ? "session" : "sessions"}`;
  if (currentView === "decisions") renderDecisions();
  renderHistory();
  renderRunMenu();
  window.scrollTo(0, 0);
}

function renderRunMenu() {
  const code = taskCode();
  if (!code) return;
  const { steps } = tasks[code];
  const provider = effectiveProvider(code);
  const missing = unavailable(provider) && !sessions.has(code);
  const queueWarning = sessions.has(code)
    ? "Close the task’s session before queueing."
    : "";
  const actions = ["Discuss", "Plan here", steps ? "Replan" : "Plan a flow"];
  const menu = byId("run-menu");
  menu.innerHTML = `<p class="menu-caption">${icon(provider)}With ${providers[provider]}</p>
    ${missing ? '<p class="menu-caption">Codex CLI is not installed.</p><button role="menuitem" tabindex="-1" data-route="settings">Agent setup in Settings</button>' : ""}
    <button role="menuitem" tabindex="-1" autofocus data-queue-task aria-disabled="${Boolean(queueWarning)}"><span>Run in queue${queueWarning ? `<small>${queueWarning}</small>` : ""}</span></button>
    <div class="menu-divider" role="separator"></div>
    ${actions.map((action) => `<button role="menuitem" tabindex="-1" data-session="${action}" aria-disabled="${missing}">${action}</button>`).join("")}`;
}

function positionMenu(menu, anchor) {
  const box = anchor.getBoundingClientRect();
  const width = menu.offsetWidth;
  const x = menu.dataset.align === "end" ? box.right - width : box.left;
  menu.style.left = `${Math.max(8, Math.min(innerWidth - width - 8, x))}px`;
  const y =
    menu.dataset.side === "top"
      ? box.top - menu.offsetHeight - 6
      : box.bottom + 5;
  menu.style.top = `${Math.max(8, y)}px`;
}

function onMenuToggle(menu, event) {
  const anchor = byId(menu.dataset.anchor);
  if (!anchor) return;
  anchor.setAttribute("aria-expanded", event.newState === "open");
  if (event.newState === "closed") return;
  positionMenu(menu, anchor);
  if (
    menu.getAttribute("role") === "menu" &&
    !menu.contains(document.activeElement)
  )
    menu.querySelector("[autofocus]")?.focus();
}

function focusMenuItem(menu, index) {
  const entries = [...menu.querySelectorAll('[role^="menuitem"]')];
  entries[(index + entries.length) % entries.length]?.focus();
}

function onMenuKey(event) {
  const menu = event.target.closest('[role="menu"]');
  if (!menu) return;
  const entries = [...menu.querySelectorAll('[role^="menuitem"]')];
  const index = entries.indexOf(document.activeElement);
  const positions = {
    ArrowDown: index + 1,
    ArrowUp: index - 1,
    Home: 0,
    End: entries.length - 1,
  };
  if (!(event.key in positions)) return;
  event.preventDefault();
  focusMenuItem(menu, positions[event.key]);
}

function closePopovers() {
  for (const menu of document.querySelectorAll("[popover]:popover-open"))
    menu.hidePopover();
}

function setSession(code, provider, command) {
  sessions.set(code, { provider, command });
  renderScreen();
}

function startSession(action) {
  const code = taskCode();
  const live = sessions.get(code);
  const provider = effectiveProvider(code);
  closePopovers();
  if (!live && unavailable(provider)) {
    byId("run-menu").showPopover({ source: byId("run-arrow") });
    return;
  }
  setSession(
    code,
    provider,
    action === "Run here" && live ? live.command : action,
  );
  showSession(code, action);
}

function showSession(code, action) {
  const { provider, command } = sessions.get(code);
  activeSession = code;
  byId("session-title").textContent = `${providers[provider]} session`;
  byId("session-detail").innerHTML =
    `${icon(provider)}${action || command} · ${code}`;
  const commands = {
    "Run here": "",
    Refine: " refine",
    Discuss: " discuss",
    "Plan here": " plan",
    "Plan a flow": " plan flow",
    Replan: " replan",
  };
  byId("session-command").textContent =
    `> /rk task:${code}${commands[command]}\n\n${command === "Run here" ? "Task context loaded." : `${command} · task context loaded.`}`;
  byId("session-dialog").showModal();
}

function closeSession() {
  sessions.delete(activeSession);
  if (activeSession === "RV-212") byId("live-claude").checked = false;
  byId("session-dialog").close();
  activeSession = null;
  renderScreen();
}

function openSearch() {
  byId("global-query").value = "";
  renderGlobalSearch();
  byId("search-dialog").showModal();
  byId("global-query").focus();
}

function findTasks(query, codes) {
  const term = query.trim().toLowerCase();
  return codes.filter((code) =>
    `task:${code} ${tasks[code].title}`.toLowerCase().includes(term),
  );
}

function renderGlobalSearch() {
  const codes = findTasks(byId("global-query").value, Object.keys(tasks));
  byId("global-results").innerHTML = codes.length
    ? codes.map(taskLink).join("")
    : '<p class="muted">No matching tasks.</p>';
}

function openQueue() {
  renderQueue();
  byId("queue-panel").showModal();
}

function queueRow(item, index) {
  const { task, provider, model, effort } = item;
  const modelLabel =
    provider === "claude" ? claudeModels[model] : codexModels[model].label;
  return `<li class="queue-row">
    <span class="count">${index + 1}</span><span class="bead" aria-hidden="true"></span>
    <div>
      <button class="queue-row-title" data-queued-task="${task}">${tasks[task].title}</button>
      <p class="queue-row-meta"><code>task:${task}</code><span>Waiting</span></p>
      <div class="queue-tags" aria-label="Saved launch settings">
        <span class="tag" title="Provider">${icon(provider)}${providers[provider]}</span>
        <span class="tag" title="Model"><span class="sr-only">Model </span>${modelLabel}</span>
        <span class="tag" title="Effort"><span class="sr-only">Effort </span>${effortLabels[effort]}</span>
      </div>
      ${tasks[task].steps ? `<span class="count">0 of ${tasks[task].steps} steps</span>` : ""}
    </div>
    <button class="button small quiet" data-remove="${task}" aria-label="Remove ${task} from queue">Remove</button>
  </li>`;
}

function renderQueue() {
  byId("queue-rows").innerHTML = queue.map(queueRow).join("");
  byId("empty-order").hidden = queue.length > 0;
  byId("queue-count").textContent = queue.length;
  byId("queue-status").textContent = queueRunning ? "Running" : "Idle";
  byId("queue-start").textContent = queueRunning ? "Stop queue" : "Start queue";
  byId("queue-start").disabled = queue.length === 0;
}

function renderTaskMatches() {
  const input = byId("task-query");
  const list = byId("task-matches");
  const term = input.value.trim();
  queueMatches = term
    ? findTasks(
        term,
        Object.keys(tasks).filter(
          (code) => !queue.some((row) => row.task === code),
        ),
      )
    : [];
  matchIndex = Math.max(0, Math.min(matchIndex, queueMatches.length - 1));
  const show = Boolean(term) && document.activeElement === input;
  list.hidden = !show;
  input.setAttribute("aria-expanded", show);
  if (queueMatches.length && show)
    input.setAttribute("aria-activedescendant", `match-${matchIndex}`);
  else input.removeAttribute("aria-activedescendant");
  list.innerHTML = queueMatches.length
    ? queueMatches
        .map(
          (code, index) =>
            `<li id="match-${index}" role="option" aria-selected="${index === matchIndex}" data-pick="${code}">${tasks[code].title}<small class="mono">task:${code}</small></li>`,
        )
        .join("")
    : '<li class="no-match" role="option" aria-disabled="true">No matching tasks.</li>';
}

function pickWithKeyboard(event) {
  if (!["ArrowDown", "ArrowUp", "Enter", "Escape"].includes(event.key)) return;
  if (event.key === "Escape" && byId("task-query").value === "") return;
  event.preventDefault();
  if (event.key === "Escape") {
    byId("task-query").value = "";
  } else if (event.key === "Enter" && queueMatches[matchIndex]) {
    pickTask(queueMatches[matchIndex]);
  } else if (queueMatches.length) {
    const delta = event.key === "ArrowDown" ? 1 : -1;
    matchIndex =
      (matchIndex + delta + queueMatches.length) % queueMatches.length;
  }
  renderTaskMatches();
}

function pickTask(code) {
  byId("task-query").value = "";
  renderTaskMatches();
  openLaunch(code, "add", byId("task-query"));
}

function openLaunch(task, entry, source) {
  closePopovers();
  launchReturn = source;
  source.focus();
  launchDraft = {
    task,
    entry,
    provider: globalProvider,
    model: "default",
    effort: "default",
  };
  const dialog = byId("launch-dialog");
  dialog.classList.toggle("over-queue", entry === "add");
  byId("launch-title").textContent =
    entry === "add" ? "Add to run queue" : "Run in queue";
  byId("launch-code").textContent = `task:${task}`;
  byId("launch-task").textContent = tasks[task].title;
  byId("launch-copy").textContent =
    entry === "add"
      ? "These settings are saved for this task. The queue starts separately."
      : "These settings are saved for this task. Confirming starts the queue if it is idle.";
  byId("queue-confirm").textContent =
    entry === "add"
      ? "Add to queue"
      : queueRunning
        ? "Add to running queue"
        : "Add and start queue";
  renderLaunchProviders();
  renderLaunchFields();
  dialog.showModal();
  if (sessions.has(task)) byId("launch-state").focus();
  else focusQueueProvider();
  if (
    globalProvider === "codex" &&
    !unavailable("codex") &&
    catalogState === "idle"
  )
    loadCatalog();
}

function focusQueueProvider() {
  const choices = byId("queue-providers");
  const active =
    choices.querySelector(":checked:not(:disabled)") ??
    choices.querySelector("input:not(:disabled)");
  active?.focus();
}

function renderLaunchProviders() {
  byId("queue-providers").innerHTML = Object.entries(providers)
    .map(([provider, name]) => {
      const missing = unavailable(provider);
      return `<label class="provider-choice">
      <input type="radio" name="queue-provider" value="${provider}" ${launchDraft.provider === provider ? "checked" : ""} ${missing ? "disabled" : ""}>
      <span>${icon(provider)}${name}</span>${missing ? "<small>CLI is not installed</small>" : ""}
    </label>`;
    })
    .join("");
}

function options(select, values, selected) {
  select.replaceChildren(
    ...Object.entries(values).map(
      ([value, label]) => new Option(label, value, false, selected === value),
    ),
  );
}

function launchMessage(provider, task) {
  const live = sessions.get(task);
  if (live)
    return {
      state: "blocked",
      text: `Close the live ${providers[live.provider]} session before queueing this task.`,
    };
  if (unavailable(provider)) {
    return {
      state: "blocked",
      text: "Codex CLI is not installed. Choose Claude Code or open agent setup.",
      action:
        '<a href="#settings" data-launch-setup>Agent setup in Settings</a>',
    };
  }
  if (queue.some((row) => row.task === task))
    return { state: "blocked", text: "This task is already in the run queue." };
  if (provider === "codex" && catalogState === "failed") {
    return {
      state: "error",
      text: "Could not load Codex models.",
      action:
        '<button class="button small secondary" type="button" id="retry-catalog">Retry</button>',
    };
  }
  if (provider === "codex" && catalogState !== "ready")
    return { state: "loading", text: "Loading Codex models…" };
  return null;
}

function renderLaunchFields() {
  const { provider, task, model } = launchDraft;
  const modelSelect = byId("model");
  const effortSelect = byId("effort");
  const state = byId("launch-state");
  const message = launchMessage(provider, task);
  const models =
    provider === "claude"
      ? claudeModels
      : Object.fromEntries(
          Object.entries(codexModels).map(([value, entry]) => [
            value,
            entry.label,
          ]),
        );
  const efforts =
    provider === "claude"
      ? Object.keys(effortLabels)
      : codexModels[model].efforts;
  if (!efforts.includes(launchDraft.effort)) launchDraft.effort = "default";
  options(modelSelect, message ? { default: "—" } : models, model);
  options(
    effortSelect,
    message
      ? { default: "—" }
      : Object.fromEntries(
          efforts.map((effort) => [effort, effortLabels[effort]]),
        ),
    launchDraft.effort,
  );
  modelSelect.disabled = Boolean(message);
  effortSelect.disabled = Boolean(message);
  byId("queue-confirm").disabled = Boolean(message);
  state.hidden = !message;
  state.dataset.state = message?.state ?? "";
  state.tabIndex = -1;
  state.innerHTML = message
    ? `<span role="status">${message.text}</span>${message.action ?? ""}`
    : "";
}

function loadCatalog(retry = false) {
  if (catalogState === "loading") return;
  catalogState = "loading";
  renderLaunchFields();
  catalogTimer = setTimeout(() => {
    const fail = !retry && byId("catalog-fails").checked;
    catalogState = fail ? "failed" : "ready";
    if (fail) byId("catalog-fails").checked = false;
    if (byId("launch-dialog").open) {
      renderLaunchFields();
      if (retry && launchDraft.provider === "codex") byId("model").focus();
    }
  }, 650);
}

function changeQueueProvider(provider) {
  launchDraft.provider = provider;
  launchDraft.model = "default";
  launchDraft.effort = "default";
  renderLaunchFields();
  if (provider === "codex" && catalogState === "idle") loadCatalog();
}

function confirmLaunch(event) {
  event.preventDefault();
  if (!launchDraft || launchMessage(launchDraft.provider, launchDraft.task))
    return;
  const { task, provider, model, effort, entry } = launchDraft;
  queue.push({ task, provider, model, effort });
  if (entry === "run") queueRunning = true;
  byId("launch-dialog").close("confirmed");
  renderQueue();
  announce(`${task} added to the run queue`);
  if (entry === "run") openQueue();
}

function finishLaunch() {
  if (byId("launch-dialog").open) return;
  launchDraft = null;
  if (!byId("queue-panel").open || launchReturn !== byId("run-arrow"))
    launchReturn?.focus();
  launchReturn = null;
}

function removeQueueTask(code) {
  queue.splice(
    queue.findIndex((item) => item.task === code),
    1,
  );
  if (!queue.length) queueRunning = false;
  renderQueue();
  byId("task-query").focus();
}

function changePreviewState(target) {
  if (target.id === "live-claude") {
    if (target.checked) setSession("RV-212", "claude", "Run here");
    else {
      sessions.delete("RV-212");
      renderScreen();
    }
  }
  if (target.id === "missing-codex") {
    renderGlobalProvider();
    renderRunMenu();
  }
  if (target.id === "catalog-fails") {
    clearTimeout(catalogTimer);
    catalogState = "idle";
  }
}

function resetPreview() {
  location.href = location.pathname;
}

function onRouteClick(control) {
  const dialog = control.closest("dialog");
  if (control.hasAttribute("data-launch-setup")) {
    byId("launch-dialog").close();
    if (byId("queue-panel").open) byId("queue-panel").close();
  } else if (dialog?.open) dialog.close();
  navigate(control.dataset.route || control.hash.slice(1));
}

function onClick(event) {
  const control = event.target.closest("button, a");
  if (!control || control.getAttribute("aria-disabled") === "true") return;
  if (control.dataset.route || control.matches('a[href^="#"]')) {
    event.preventDefault();
    if (control.hash !== "#description") onRouteClick(control);
    return;
  }
  if (control.hasAttribute("data-close")) control.closest("dialog").close();
  if (control.dataset.provider) selectGlobal(control.dataset.provider);
  if (control.dataset.session) startSession(control.dataset.session);
  if (control.hasAttribute("data-queue-task"))
    openLaunch(taskCode(), "run", byId("run-arrow"));
  if (control.dataset.queuedTask) {
    byId("queue-panel").close();
    navigate(tasks[control.dataset.queuedTask].route);
  }
  if (control.dataset.remove) removeQueueTask(control.dataset.remove);
  if (control.dataset.copy) {
    navigator.clipboard.writeText(`/rk task:${control.dataset.copy}`).then(
      () => announce("Task anchor copied"),
      () => announce(`/rk task:${control.dataset.copy}`),
    );
  }
}

function handleShortcut(event) {
  if (
    (event.metaKey || event.ctrlKey) &&
    event.key.toLowerCase() === "k" &&
    !document.querySelector("dialog[open]")
  ) {
    event.preventDefault();
    openSearch();
  }
}

function wirePreview() {
  document.addEventListener("click", onClick);
  document.addEventListener("keydown", onMenuKey);
  document.addEventListener("keydown", handleShortcut);
  window.addEventListener("popstate", onHistory);
  window.addEventListener("resize", () => {
    for (const menu of document.querySelectorAll("[popover]:popover-open"))
      positionMenu(menu, byId(menu.dataset.anchor));
  });
  for (const menu of document.querySelectorAll("[data-anchor]"))
    menu.addEventListener("toggle", (event) => onMenuToggle(menu, event));
  byId("back").addEventListener("click", () => history.back());
  byId("forward").addEventListener("click", () => history.forward());
  byId("search-key").addEventListener("click", openSearch);
  byId("global-query").addEventListener("input", renderGlobalSearch);
  byId("screen").addEventListener("input", (event) => {
    if (event.target.id === "decision-query") renderDecisions();
  });
  byId("queue-key").addEventListener("click", openQueue);
  byId("queue-start").addEventListener("click", () => {
    queueRunning = !queueRunning;
    renderQueue();
  });
  byId("task-query").addEventListener("input", () => {
    matchIndex = 0;
    renderTaskMatches();
  });
  byId("task-query").addEventListener("focus", renderTaskMatches);
  byId("task-query").addEventListener("blur", renderTaskMatches);
  byId("task-query").addEventListener("keydown", pickWithKeyboard);
  byId("task-matches").addEventListener("pointerdown", (event) => {
    const match = event.target.closest("[data-pick]");
    if (match) {
      event.preventDefault();
      pickTask(match.dataset.pick);
    }
  });
  byId("launch-form").addEventListener("submit", confirmLaunch);
  byId("launch-dialog").addEventListener("close", finishLaunch);
  byId("launch-state").addEventListener("click", (event) => {
    if (event.target.id === "retry-catalog") loadCatalog(true);
  });
  byId("queue-providers").addEventListener("change", (event) =>
    changeQueueProvider(event.target.value),
  );
  byId("model").addEventListener("change", (event) => {
    launchDraft.model = event.target.value;
    renderLaunchFields();
  });
  byId("effort").addEventListener("change", (event) => {
    launchDraft.effort = event.target.value;
  });
  byId("demo-menu").addEventListener("change", (event) =>
    changePreviewState(event.target),
  );
  byId("reset").addEventListener("click", resetPreview);
  byId("session-dock").addEventListener("click", () =>
    showSession(
      sessions.has(taskCode()) ? taskCode() : sessions.keys().next().value,
    ),
  );
  byId("end-session").addEventListener("click", closeSession);
}

drawIcons(document);
seedHistory();
wirePreview();
renderGlobalProvider();
renderScreen();
renderQueue();
