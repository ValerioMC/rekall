/**
 * Whether Claude Code can reach this instance. Mirrors
 * `dev.rekall.bootstrap.ClaudeCodeInstaller.Installation` on the server.
 */
export type ClaudeConnectionStatus = 'CONNECTED' | 'OUTDATED' | 'NOT_CONNECTED' | 'CLI_MISSING'

export interface ClaudeInstallation {
  readonly status: ClaudeConnectionStatus
  /** The MCP endpoint this instance is serving, which is what gets registered. */
  readonly endpoint: string
  /** What Claude Code has registered for every folder, or null when it has nothing. */
  readonly registeredUrl: string | null
  /** Folders keeping a setup of their own, which wins inside them until an install clears it. */
  readonly folderScoped: readonly string[]
  /** Whether the installed `/rk` command matches the one shipped with this build. */
  readonly commandInstalled: boolean
  readonly cliPath: string | null
  /** The shell equivalent, for a machine with no `claude` binary to run it with. */
  readonly manualCommand: string
}
