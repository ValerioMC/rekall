export interface SlashCommand {
  readonly name: string
  readonly hint: string
  readonly insert: string
}

/** The commands that work when `claude` runs headless; `/clear` is left out because it changes the session id. */
export const SLASH_COMMANDS: readonly SlashCommand[] = [
  { name: '/context', hint: 'What fills the context window', insert: '/context' },
  { name: '/model', hint: 'Switch the model', insert: '/model ' },
  { name: '/effort', hint: 'Set the reasoning effort', insert: '/effort ' },
  { name: '/compact', hint: 'Summarise the conversation so far', insert: '/compact' },
  { name: '/rk', hint: 'Load a Rekall context', insert: '/rk ' }
]

/** The commands a field starting with `/` is still choosing between; none once a space follows the name. */
export function matchingCommands(text: string): readonly SlashCommand[] {
  if (!text.startsWith('/') || /\s/.test(text)) return []
  return SLASH_COMMANDS.filter((command) => command.name.startsWith(text.toLowerCase()))
}
