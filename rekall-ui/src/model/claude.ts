export type ClaudeConnectionStatus = 'CONNECTED' | 'OUTDATED' | 'NOT_CONNECTED' | 'CLI_MISSING'

export interface ClaudeInstallation {
  readonly status: ClaudeConnectionStatus
  readonly endpoint: string
  readonly registeredUrl: string | null
  readonly folderScoped: readonly string[]
  readonly commandInstalled: boolean
  readonly cliPath: string | null
  readonly manualCommand: string
}
