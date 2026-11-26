import type { SessionKind } from '@/features/claude/model/terminal'

export const SESSION_PROVIDERS = ['CLAUDE', 'CODEX'] as const

export type SessionProvider = (typeof SESSION_PROVIDERS)[number]

export const SESSION_PROVIDER_LABEL: Readonly<Record<SessionProvider, string>> = {
  CLAUDE: 'Claude Code',
  CODEX: 'Codex'
}

export const RUN_WITH_PROVIDER_LABEL: Readonly<Record<SessionProvider, string>> = {
  CLAUDE: `Run with ${SESSION_PROVIDER_LABEL.CLAUDE}`,
  CODEX: `Run with ${SESSION_PROVIDER_LABEL.CODEX}`
}

export const SESSION_WORKFLOW_LABEL: Readonly<Record<SessionProvider, string>> = {
  CLAUDE: '/rk',
  CODEX: '$rekall'
}

export function sessionCommandPrompt(provider: SessionProvider, line: string): string {
  if (provider === 'CODEX' && line.startsWith('/rk ')) {
    return `Use the ${SESSION_WORKFLOW_LABEL.CODEX} skill with this exact argument line: ${line.slice('/rk '.length)}`
  }
  return line
}

export function sessionViewLabel(kind: SessionKind): string {
  return kind === 'CHAT' ? 'Chat' : 'Terminal'
}
