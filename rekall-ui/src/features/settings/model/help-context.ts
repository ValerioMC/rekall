export type HelpAccess = 'read' | 'write'

export type HelpToolName =
  | 'rekall_context'
  | 'rekall_search'
  | 'rekall_read_sections'
  | 'rekall_reorganize_sections'
  | 'rekall_component'
  | 'rekall_read_components'
  | 'rekall_create_task'
  | 'rekall_join_epic'
  | 'rekall_description'
  | 'rekall_propose_step'
  | 'rekall_propose_flow'
  | 'rekall_replan'
  | 'rekall_delete_step'
  | 'rekall_read_steps'
  | 'rekall_edit_step'
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

export type HelpGroupId = 'loop' | 'around' | 'elsewhere'

export interface HelpCommand {
  readonly line: string
  readonly does: string
  readonly calls: readonly HelpToolName[]
}

export interface HelpExample {
  readonly title: string
  readonly prompt: string
  readonly tools: readonly HelpToolName[]
}

export interface HelpContext {
  readonly id: HelpContextId
  readonly group: HelpGroupId
  readonly label: string
  readonly summary: string
  readonly commandsHeading: string
  readonly commands: readonly HelpCommand[]
  readonly directTools: readonly HelpToolName[]
  readonly examples: readonly HelpExample[]
}

export const HELP_GROUPS: readonly Readonly<{ id: HelpGroupId; label: string }>[] = [
  { id: 'loop', label: 'The loop' },
  { id: 'around', label: 'Around it' },
  { id: 'elsewhere', label: 'From elsewhere' }
]

export const HELP_TOOLS: Readonly<Record<HelpToolName, HelpTool>> = {
  rekall_context: { name: 'rekall_context', access: 'read', effect: 'Resolves anchors and returns the whole working context as markdown.' },
  rekall_search: { name: 'rekall_search', access: 'read', effect: 'Finds a phrase in descriptions, steps, wrapups and notes; returns anchors to load.' },
  rekall_read_sections: { name: 'rekall_read_sections', access: 'read', effect: 'Lists a project’s pages by title, or reads one whole.' },
  rekall_reorganize_sections: { name: 'rekall_reorganize_sections', access: 'write', effect: 'Regroups a project’s pages: names every page once under the contexts it wants. Pages move; none is renamed, rewritten or deleted.' },
  rekall_component: { name: 'rekall_component', access: 'write', effect: 'Records a folder of a project as a component: a summary and a structured description; one with that name or folder is rewritten.' },
  rekall_read_components: { name: 'rekall_read_components', access: 'read', effect: 'Lists a project’s components with their summaries, or reads one whole.' },
  rekall_create_task: { name: 'rekall_create_task', access: 'write', effect: 'Creates a task or an epic in a project, with its description and up to 20 steps, as a checklist or a flow.' },
  rekall_join_epic: { name: 'rekall_join_epic', access: 'write', effect: 'Files a task under an epic of its project, or moves it to another one.' },
  rekall_description: { name: 'rekall_description', access: 'write', effect: 'Replaces a task’s description whole; the old text is not kept.' },
  rekall_propose_step: { name: 'rekall_propose_step', access: 'write', effect: 'Adds one step, open and ready to work.' },
  rekall_propose_flow: { name: 'rekall_propose_flow', access: 'write', effect: 'Adds a whole flow, steps and routes, to a task that has no steps.' },
  rekall_replan: { name: 'rekall_replan', access: 'write', effect: 'Replaces a task’s draft and open steps with a new plan; running, claimed and done steps stay.' },
  rekall_delete_step: { name: 'rekall_delete_step', access: 'write', effect: 'Deletes one draft or open step; a step already started is refused.' },
  rekall_read_steps: { name: 'rekall_read_steps', access: 'read', effect: 'Reads a task’s draft and open steps as typed: number, title, state, detail and routes.' },
  rekall_edit_step: { name: 'rekall_edit_step', access: 'write', effect: 'Rewrites the title or detail of one draft or open step; everything else on it stays.' },
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
    group: 'loop',
    label: 'Load',
    summary: 'Hand a session the current state of the work in one call, or find the work first.',
    commandsHeading: 'Commands',
    commands: [
      { line: '/rk task:{task}', does: 'Loads the task: description, wrapup, steps and notes. Open steps become the work.', calls: ['rekall_context'] },
      { line: '/rk project:{project}', does: 'Loads a project and waits: with no task there is no brief.', calls: ['rekall_context'] },
      { line: '/rk company:acme', does: 'Loads a company and its projects, and waits.', calls: ['rekall_context'] }
    ],
    directTools: ['rekall_search', 'rekall_read_sections'],
    examples: []
  },
  {
    id: 'tasks',
    group: 'loop',
    label: 'Tasks',
    summary: 'Start from a task, or create one. A created task starts as TODO, with its steps open.',
    commandsHeading: 'Commands',
    commands: [{ line: '/rk task:{task}', does: 'Works the task: the open steps in order, or the description when there are none.', calls: ['rekall_context'] }],
    directTools: ['rekall_create_task', 'rekall_join_epic'],
    examples: []
  },
  {
    id: 'steps',
    group: 'loop',
    label: 'Steps',
    summary: 'Plan the checklist, then move each step as it is worked. Only you tick a step done.',
    commandsHeading: 'Commands',
    commands: [
      { line: '/rk task:{task} plan', does: 'Turns the task into a checklist, open and ready to work.', calls: ['rekall_propose_step'] },
      { line: '/rk task:{task} plan flow', does: 'The same, as a flow: steps joined by routes, for work whose outcome decides what is next.', calls: ['rekall_propose_flow'] },
      { line: '/rk task:{task} replan "also cover the csv export"', does: 'Redesigns the draft and open steps. The quoted text is what the plan missed.', calls: ['rekall_replan'] },
      { line: '/rk task:{task} discuss', does: 'A talk you lead about the work still to do. What you agree is written to the task; nothing is executed.', calls: ['rekall_read_steps', 'rekall_edit_step', 'rekall_description'] },
      { line: '/rk task:{task} step:3 start', does: 'Marks step 3 running.', calls: ['rekall_step'] },
      { line: '/rk task:{task} step:3 done', does: 'Claims step 3 for your review. It does not tick it.', calls: ['rekall_step'] },
      { line: '/rk task:{task} step:3 done outcome:failed report:"job test: expected 1.2, got 1.3"', does: 'On a flow, claims the step with the route that happened and what was seen.', calls: ['rekall_step'] }
    ],
    directTools: ['rekall_delete_step', 'rekall_deviation'],
    examples: []
  },
  {
    id: 'wrapup',
    group: 'loop',
    label: 'Wrapup',
    summary: 'Record what the implementation is now, then bring the work home.',
    commandsHeading: 'Commands',
    commands: [
      { line: '/rk task:{task} wrapup', does: 'Replaces the wrapup with what the task’s implementation looks like now.', calls: ['rekall_wrapup'] },
      { line: '/rk task:{task} wrapup "only the export module"', does: 'The same, with a directive on what it has to say.', calls: ['rekall_wrapup'] },
      { line: '/rk task:{task} merge', does: 'An accepted task conflicts with the project’s branch: merges that branch into the task’s and resolves.', calls: [] },
      { line: '/rk task:{task} merge pull-request base:main', does: 'The task’s pull request conflicts with its base: merges the base in, resolves and pushes.', calls: [] }
    ],
    directTools: ['rekall_record_commit'],
    examples: []
  },
  {
    id: 'notes',
    group: 'around',
    label: 'Notes and pages',
    summary: 'Keep what a session produced: a note belongs to one task, a page to the whole project.',
    commandsHeading: 'Commands',
    commands: [
      { line: '/rk task:{task} note "the export formats we settled on"', does: 'Writes a note on the task, or rewrites the one it already has on that subject.', calls: ['rekall_note'] },
      { line: '/rk task:{task} section "Deployment" "how we ship to production"', does: 'Writes a project page under that title, or rewrites the one that exists.', calls: ['rekall_project_section'] },
      { line: '/rk project:{project} components', does: 'Reads the project\'s folders and writes each part as a component: a summary and a structured description.', calls: ['rekall_component', 'rekall_read_components'] },
      { line: '/rk project:{project} autoconfigure', does: 'Reads the repository and stores how it is built and how it takes a change, as pages.', calls: ['rekall_project_section'] },
      { line: '/rk project:{project} reorganize', does: 'Reads every page and regroups them in contexts by concept, moving pages and naming contexts without rewriting any page.', calls: ['rekall_reorganize_sections', 'rekall_read_sections'] }
    ],
    directTools: ['rekall_read_sections'],
    examples: []
  },
  {
    id: 'revisions',
    group: 'around',
    label: 'Revisions',
    summary: 'You revise a description in the console; a session answers it.',
    commandsHeading: 'Commands',
    commands: [
      { line: '/rk task:{task} revise', does: 'Works the pending revisions of the thread, on top of what is built.', calls: ['rekall_feedback'] },
      { line: '/rk task:{task} rewrite fix', does: 'Corrects the wording of the description and stores it. Builds nothing.', calls: ['rekall_description'] },
      { line: '/rk task:{task} rewrite restructure', does: 'Lays the description out under the standard headings.', calls: ['rekall_description'] }
    ],
    directTools: [],
    examples: []
  },
  {
    id: 'diagrams',
    group: 'around',
    label: 'Diagrams',
    summary: 'A session reads the code and stores Mermaid sheets; the console draws them.',
    commandsHeading: 'Commands',
    commands: [
      { line: '/rk project:{project} generate kinds:sequence,state "how checkout decides what to charge"', does: 'One sheet per kind, linked to the code. With a task anchor it is stored on the task too.', calls: ['rekall_diagram'] },
      { line: '/rk project:{project} explain "how an order is placed"', does: 'A walkthrough the console plays one call at a time.', calls: ['rekall_diagram'] }
    ],
    directTools: [],
    examples: []
  },
  {
    id: 'security',
    group: 'around',
    label: 'Security',
    summary: 'List what a project is built from, then decide which advisories really reach it.',
    commandsHeading: 'Commands',
    commands: [
      { line: '/rk project:{project} inventory', does: 'Records every library at its exact version; Rekall checks it against critical advisories.', calls: ['rekall_inventory'] },
      { line: '/rk project:{project} assess', does: 'Gives a verdict on each advisory not yet assessed.', calls: ['rekall_vulnerabilities', 'rekall_assess_vulnerability'] },
      { line: '/rk project:{project} assess advisory:GHSA-xxxx-xxxx-xxxx', does: 'Gives a verdict on one advisory.', calls: ['rekall_vulnerabilities', 'rekall_assess_vulnerability'] }
    ],
    directTools: [],
    examples: []
  },
  {
    id: 'outside',
    group: 'elsewhere',
    label: 'Outside Rekall',
    summary:
      'Registered once for every folder, the MCP server answers in any Claude Code session, including one Rekall did not open. Ask in plain words and the session picks the tool. Use labels and codes, never titles.',
    commandsHeading: 'Set up once',
    commands: [
      {
        line: 'claude mcp add --scope user --transport http rekall http://localhost:47355/mcp',
        does: 'Registers the server for every folder. Sessions and run queue does the same in one click and repairs a registration.',
        calls: []
      }
    ],
    directTools: [],
    examples: [
      {
        title: 'Create a task in a project',
        prompt:
          'In Rekall, create a task in project {project} titled "Export the report as CSV". Describe what it is for, and add three steps: expose the endpoint, add the download button, cover it with tests.',
        tools: ['rekall_create_task']
      },
      {
        title: 'Create an epic and file a task under it',
        prompt: 'In Rekall, create an epic "Reporting" in project {project}, then file task {task} under it.',
        tools: ['rekall_create_task', 'rekall_join_epic']
      },
      {
        title: 'Add a step to a task',
        prompt: 'In Rekall, add a step "Cover the empty report" to task {task}.',
        tools: ['rekall_propose_step']
      },
      {
        title: 'Keep what this conversation settled',
        prompt: 'In Rekall, write a note on task {task} titled "Export formats" with what we settled on in this conversation.',
        tools: ['rekall_note']
      },
      {
        title: 'Log the commit you just made',
        prompt: 'In Rekall, log the commit I just made against task {task}.',
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

export function helpContextOf(segment: string | undefined): HelpContext {
  return HELP_CONTEXTS.find((context) => context.id === segment) ?? HELP_CONTEXTS[0]!
}
