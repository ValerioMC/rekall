import { HELP_CONTEXTS, HELP_TOOLS, type HelpContext } from '@/features/settings/model/help-context'

export interface HelpValues {
  readonly project: string
  readonly task: string
}

export interface HelpSegment {
  readonly text: string
  readonly replaceable: boolean
}

export const HELP_DEFAULTS: HelpValues = { project: 'vega', task: 'RK-12' }

const PLACEHOLDER = /(\{project\}|\{task\})/

function valueOf(token: string, typed: HelpValues): HelpSegment {
  const key = token === '{project}' ? 'project' : 'task'
  const value = typed[key].trim()
  return value ? { text: value, replaceable: false } : { text: HELP_DEFAULTS[key], replaceable: true }
}

export function helpSegments(template: string, typed: HelpValues): readonly HelpSegment[] {
  return template
    .split(PLACEHOLDER)
    .filter((part) => part !== '')
    .map((part) => (PLACEHOLDER.test(part) ? valueOf(part, typed) : { text: part, replaceable: false }))
}

export function helpText(template: string, typed: HelpValues): string {
  return helpSegments(template, typed)
    .map((segment) => segment.text)
    .join('')
}

function mentions(words: readonly string[], ...texts: readonly string[]): boolean {
  const haystack = texts.join(' ').toLowerCase()
  return words.every((word) => haystack.includes(word))
}

function narrowedTo(words: readonly string[], context: HelpContext): HelpContext {
  return {
    ...context,
    commands: context.commands.filter((command) => mentions(words, command.line, command.does, ...command.calls)),
    directTools: context.directTools.filter((name) => mentions(words, name, HELP_TOOLS[name].effect)),
    examples: context.examples.filter((example) => mentions(words, example.title, example.prompt, ...example.tools))
  }
}

function hasContent(context: HelpContext): boolean {
  return context.commands.length + context.directTools.length + context.examples.length > 0
}

/** Every context cut down to what mentions every word of the query; a context left empty is dropped. */
export function findInHelp(query: string): readonly HelpContext[] {
  const words = query.toLowerCase().split(/\s+/).filter((word) => word !== '')
  return HELP_CONTEXTS.map((context) => narrowedTo(words, context)).filter(hasContent)
}
