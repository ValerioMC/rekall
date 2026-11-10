import type { Connection, ConnectionKind, CredentialDraft } from '@/features/connection/model/connection'

/** An empty credential of the kind the connection takes. */
export function emptyDraft(kind: ConnectionKind): CredentialDraft {
  switch (kind) {
    case 'GITLAB':
    case 'GITHUB':
      return { kind, token: '' }
    case 'GRAFANA':
      return { kind, authentication: 'SERVICE_ACCOUNT', user: '', token: '' }
    case 'KUBERNETES':
      return { kind, kubeconfig: '', context: null }
    case 'JIRA':
      return { kind, deployment: 'CLOUD', email: '', token: '' }
  }
}

/** Whether the draft holds enough to send: a token, a kubeconfig with a context chosen, on Jira Cloud an email and a token, or on Grafana a token or a user and a password. */
export function isComplete(draft: CredentialDraft): boolean {
  switch (draft.kind) {
    case 'GITLAB':
    case 'GITHUB':
      return draft.token.trim().length > 0
    case 'GRAFANA':
      return draft.token.trim().length > 0 && (draft.authentication === 'SERVICE_ACCOUNT' || draft.user.trim().length > 0)
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
    case 'GITHUB':
      return draft.token.length > 0
    case 'GRAFANA':
      return draft.token.length > 0 || (draft.authentication === 'PASSWORD' && draft.user.length > 0)
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
    case 'GITHUB':
    case 'GRAFANA':
      return 'token'
    case 'KUBERNETES':
      return 'kubeconfig'
    case 'JIRA':
      return 'token'
  }
}

/** The credential as edited beside the one stored: Jira Cloud when an email was stored, Grafana with a password when a user was, a Kubernetes one keeps its stored context. */
export function draftOfEdit(connection: Connection, secret: string, email: string | null, user: string | null): CredentialDraft {
  switch (connection.kind) {
    case 'GITLAB':
    case 'GITHUB':
      return { kind: connection.kind, token: secret }
    case 'GRAFANA':
      return user === null
        ? { kind: 'GRAFANA', authentication: 'SERVICE_ACCOUNT', user: '', token: secret }
        : { kind: 'GRAFANA', authentication: 'PASSWORD', user, token: secret }
    case 'KUBERNETES':
      return { kind: 'KUBERNETES', kubeconfig: secret, context: connection.kubeContext }
    case 'JIRA':
      return email === null
        ? { kind: 'JIRA', deployment: 'DATA_CENTER', email: '', token: secret }
        : { kind: 'JIRA', deployment: 'CLOUD', email, token: secret }
  }
}
