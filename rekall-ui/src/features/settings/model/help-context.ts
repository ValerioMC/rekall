export type HelpAccess = 'read' | 'write'

export type HelpToolName =
  | 'rekall_context'
  | 'rekall_search'
  | 'rekall_read_sections'
  | 'rekall_create_task'
  | 'rekall_join_epic'
  | 'rekall_description'
  | 'rekall_propose_step'
  | 'rekall_propose_flow'
  | 'rekall_replan'
  | 'rekall_delete_step'
  | 'rekall_step'
  | 'rekall_deviation'
  | 'rekall_wrapup'
  | 'rekall_record_commit'
  | 'rekall_note'
  | 'rekall_project_section'
  | 'rekall_feedback'
  | 'rekall_diagram'
  | 'rekall_inventory'
  | 'rekall_vulnerabilities'
  | 'rekall_assess_vulnerability'

export type HelpContextId =
  | 'load'
  | 'tasks'
  | 'steps'
  | 'wrapup'
  | 'notes'
  | 'revisions'
  | 'diagrams'
  | 'security'
  | 'outside'

export interface HelpTool {
  readonly name: HelpToolName
  readonly access: HelpAccess
  readonly effect: string
}

export interface HelpCommand {
  readonly line: string
  readonly does: string
}

export interface HelpExample {
  readonly title: string
  readonly prompt: string
  readonly tools: readonly HelpToolName[]
}

export interface HelpContext {
  readonly id: HelpContextId
  readonly label: string
  readonly summary: string
  readonly commands: readonly HelpCommand[]
  readonly tools: readonly HelpToolName[]
  readonly examples: readonly HelpExample[]
}

export const HELP_TOOLS: Readonly<Record<HelpToolName, HelpTool>> = {
  rekall_context: { name: 'rekall_context', access: 'read', effect: 'Resolves anchors and returns the whole working context as markdown.' },
  rekall_search: { name: 'rekall_search', access: 'read', effect: 'Finds a phrase in descriptions, steps, wrapups and notes; returns anchors to load.' },
  rekall_read_sections: { name: 'rekall_read_sections', access: 'read', effect: 'Lists a project’s pages by title, or reads one whole.' },
  rekall_create_task: { name: 'rekall_create_task', access: 'write', effect: 'Creates a task or an epic in a project, with its description and up to 20 steps, as a checklist or a flow.' },
  rekall_join_epic: { name: 'rekall_join_epic', access: 'write', effect: 'Files a task under an epic of its project, or moves it to another one.' },
  rekall_description: { name: 'rekall_description', access: 'write', effect: 'Replaces a task’s description whole; the old text is not kept.' },
  rekall_propose_step: { name: 'rekall_propose_step', access: 'write', effect: 'Adds one step, open and ready to work.' },
  rekall_propose_flow: { name: 'rekall_propose_flow', access: 'write', effect: 'Adds a whole flow, steps and routes, to a task that has no steps.' },
  rekall_replan: { name: 'rekall_replan', access: 'write', effect: 'Replaces a task’s draft and open steps with a new plan; running, claimed and done steps stay.' },
  rekall_delete_step: { name: 'rekall_delete_step', access: 'write', effect: 'Deletes one draft or open step; a step already started is refused.' },
  rekall_step: { name: 'rekall_step', access: 'write', effect: 'Moves a step to running or claimed. Never done: that tick is yours.' },
  rekall_deviation: { name: 'rekall_deviation', access: 'write', effect: 'Records where the work departed from the plan, for one step.' },
  rekall_wrapup: { name: 'rekall_wrapup', access: 'write', effect: 'Replaces the task’s wrapup: what the implementation looks like now.' },
  rekall_record_commit: { name: 'rekall_record_commit', access: 'write', effect: 'Logs a commit of the project’s repository against a task or a step.' },
  rekall_note: { name: 'rekall_note', access: 'write', effect: 'Writes a note on a task, or rewrites one by its title.' },
  rekall_project_section: { name: 'rekall_project_section', access: 'write', effect: 'Writes a titled page on the project, or rewrites one by its title.' },
  rekall_feedback: { name: 'rekall_feedback', access: 'write', effect: 'Replies to the open revisions of a task’s thread; each moves to applied.' },
  rekall_diagram: { name: 'rekall_diagram', access: 'write', effect: 'Stores Mermaid sheets for a project or a task, or replaces one by id.' },
  rekall_inventory: { name: 'rekall_inventory', access: 'write', effect: 'Replaces a project’s dependency inventory whole.' },
  rekall_vulnerabilities: { name: 'rekall_vulnerabilities', access: 'read', effect: 'Lists the advisories that reach a project, or reads one whole.' },
  rekall_assess_vulnerability: { name: 'rekall_assess_vulnerability', access: 'write', effect: 'Records whether an advisory really reaches the project: affected or not affected.' }
}

export const HELP_CONTEXTS: readonly HelpContext[] = [
  {
    id: 'load',
    label: 'Load',
    summary: 'Hand a session the current state of the work in one call, or find the work first.',
    commands: [
      { line: '/rk task:RK-12', does: 'Loads the task: description, wrapup, steps and notes. Open steps become the work.' },
      { line: '/rk project:vega', does: 'Loads a project and waits: with no task there is no brief.' },
      { line: '/rk company:acme', does: 'Loads a company and its projects, and waits.' }
    ],
    tools: ['rekall_context', 'rekall_search', 'rekall_read_sections'],
    examples: []
  },
  {
    id: 'tasks',
    label: 'Tasks',
    summary: 'Start from a task, or create one. A created task starts as TODO, with its steps open.',
    commands: [{ line: '/rk task:RK-12', does: 'Works the task: the open steps in order, or the description when there are none.' }],
    tools: ['rekall_create_task', 'rekall_join_epic', 'rekall_context'],
    examples: []
  },
  {
    id: 'steps',
    label: 'Steps',
    summary: 'Plan the checklist, then move each step as it is worked. Only you tick a step done.',
    commands: [
      { line: '/rk task:RK-12 plan', does: 'Turns the task into a checklist, open and ready to work.' },
      { line: '/rk task:RK-12 plan flow', does: 'The same, as a flow: steps joined by routes, for work whose outcome decides what is next.' },
      { line: '/rk task:RK-12 replan "also cover the csv export"', does: 'Redesigns the draft and open steps. The quoted text is what the plan missed.' },
      { line: '/rk task:RK-12 step:3 start', does: 'Marks step 3 running.' },
      { line: '/rk task:RK-12 step:3 done', does: 'Claims step 3 for your review. It does not tick it.' },
      { line: '/rk task:RK-12 step:3 done outcome:failed report:"job test: expected 1.2, got 1.3"', does: 'On a flow, claims the step with the route that happened and what was seen.' }
    ],
    tools: ['rekall_step', 'rekall_propose_step', 'rekall_propose_flow', 'rekall_replan', 'rekall_delete_step', 'rekall_deviation'],
    examples: []
  },
  {
    id: 'wrapup',
    label: 'Wrapup',
    summary: 'Record what the implementation is now, then bring the work home.',
    commands: [
      { line: '/rk task:RK-12 wrapup', does: 'Replaces the wrapup with what the task’s implementation looks like now.' },
      { line: '/rk task:RK-12 wrapup "only the export module"', does: 'The same, with a directive on what it has to say.' },
      { line: '/rk task:RK-12 merge', does: 'An accepted task conflicts with the project’s branch: merges that branch into the task’s and resolves.' },
      { line: '/rk task:RK-12 merge pull-request base:main', does: 'The task’s pull request conflicts with its base: merges the base in, resolves and pushes.' }
    ],
    tools: ['rekall_wrapup', 'rekall_record_commit'],
    examples: []
  },
  {
    id: 'notes',
    label: 'Notes',
    summary: 'Keep what a session produced: a note belongs to one task, a page to the whole project.',
    commands: [
      { line: '/rk task:RK-12 note "the export formats we settled on"', does: 'Writes a note on the task, or rewrites the one it already has on that subject.' },
      { line: '/rk task:RK-12 section "Deployment" "how we ship to production"', does: 'Writes a project page under that title, or rewrites the one that exists.' },
      { line: '/rk project:vega autoconfigure', does: 'Reads the repository and stores how it is built and how it takes a change, as pages.' }
    ],
    tools: ['rekall_note', 'rekall_project_section', 'rekall_read_sections'],
    examples: []
  },
  {
    id: 'revisions',
    label: 'Revisions',
    summary: 'You revise a description in the console; a session answers it.',
    commands: [
      { line: '/rk task:RK-12 revise', does: 'Works the pending revisions of the thread, on top of what is built.' },
      { line: '/rk task:RK-12 rewrite fix', does: 'Corrects the wording of the description and stores it. Builds nothing.' },
      { line: '/rk task:RK-12 rewrite restructure', does: 'Lays the description out under the standard headings.' }
    ],
    tools: ['rekall_feedback', 'rekall_description'],
    examples: []
  },
  {
    id: 'diagrams',
    label: 'Diagrams',
    summary: 'A session reads the code and stores Mermaid sheets; the console draws them.',
    commands: [
      { line: '/rk project:vega generate kinds:sequence,state "how checkout decides what to charge"', does: 'One sheet per kind, linked to the code. With a task anchor it is stored on the task too.' },
      { line: '/rk project:vega explain "how an order is placed"', does: 'A walkthrough the console plays one call at a time.' }
    ],
    tools: ['rekall_diagram'],
    examples: []
  },
  {
    id: 'security',
    label: 'Security',
    summary: 'List what a project is built from, then decide which advisories really reach it.',
    commands: [
      { line: '/rk project:vega inventory', does: 'Records every library at its exact version; Rekall checks it against critical advisories.' },
      { line: '/rk project:vega assess', does: 'Gives a verdict on each advisory not yet assessed.' },
      { line: '/rk project:vega assess advisory:GHSA-xxxx-xxxx-xxxx', does: 'Gives a verdict on one advisory.' }
    ],
    tools: ['rekall_inventory', 'rekall_vulnerabilities', 'rekall_assess_vulnerability'],
    examples: []
  },
  {
    id: 'outside',
    label: 'Outside Rekall',
    summary:
      'Registered with --scope user, the MCP server answers in any folder and any Claude Code session, one Rekall did not open. Ask in plain words; the session picks the tool. Use labels and codes, never titles.',
    commands: [],
    tools: [],
    examples: [
      {
        title: 'Create a task in a project',
        prompt:
          'In Rekall, create a task in project vega titled "Export the report as CSV". Describe what it is for, and add three steps: expose the endpoint, add the download button, cover it with tests.',
        tools: ['rekall_create_task']
      },
      {
        title: 'Create an epic and file a task under it',
        prompt: 'In Rekall, create an epic "Reporting" in project vega, then file task RK-12 under it.',
        tools: ['rekall_create_task', 'rekall_join_epic']
      },
      {
        title: 'Add a step to a task',
        prompt: 'In Rekall, add a step "Cover the empty report" to task RK-12.',
        tools: ['rekall_propose_step']
      },
      {
        title: 'Keep what this conversation settled',
        prompt: 'In Rekall, write a note on task RK-12 titled "Export formats" with what we settled on in this conversation.',
        tools: ['rekall_note']
      },
      {
        title: 'Log the commit you just made',
        prompt: 'In Rekall, log the commit I just made against task RK-12.',
        tools: ['rekall_record_commit']
      },
      {
        title: 'Find where something was decided',
        prompt: 'Search Rekall for where we decided the CSV delimiter, then load the task it points to.',
        tools: ['rekall_search', 'rekall_context']
      }
    ]
  }
]

export const DEFAULT_HELP_CONTEXT: HelpContextId = 'load'

export function helpContextOf(id: HelpContextId): HelpContext {
  return HELP_CONTEXTS.find((context) => context.id === id) ?? HELP_CONTEXTS[0]!
}
