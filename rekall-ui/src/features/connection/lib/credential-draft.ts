import type { ConnectionKind, CredentialDraft } from '@/features/connection/model/connection'

/** An empty credential of the kind the connection takes. */
export function emptyDraft(kind: ConnectionKind): CredentialDraft {
  switch (kind) {
    case 'GITLAB':
      return { kind, token: '' }
    case 'KUBERNETES':
      return { kind, kubeconfig: '', context: null }
    case 'JIRA':
      return { kind, deployment: 'CLOUD', email: '', token: '' }
  }
}

/** Whether the draft holds enough to send: a token, a kubeconfig with a context chosen, or on Jira Cloud an email and a token. */
export function isComplete(draft: CredentialDraft): boolean {
  switch (draft.kind) {
    case 'GITLAB':
      return draft.token.trim().length > 0
    case 'KUBERNETES':
      return draft.kubeconfig.trim().length > 0 && draft.context !== null
    case 'JIRA':
      return draft.token.trim().length > 0 && (draft.deployment === 'DATA_CENTER' || draft.email.trim().includes('@'))
  }
}

/** Whether anything at all was typed, so an untouched field is not sent. */
export function isStarted(draft: CredentialDraft): boolean {
  switch (draft.kind) {
    case 'GITLAB':
      return draft.token.length > 0
    case 'KUBERNETES':
      return draft.kubeconfig.length > 0
    case 'JIRA':
      return draft.token.length > 0 || (draft.deployment === 'CLOUD' && draft.email.length > 0)
  }
}

/** What the credential is called in a button or a toast. */
export function credentialNoun(kind: ConnectionKind): string {
  switch (kind) {
    case 'GITLAB':
      return 'token'
    case 'KUBERNETES':
      return 'kubeconfig'
    case 'JIRA':
      return 'token'
  }
}
