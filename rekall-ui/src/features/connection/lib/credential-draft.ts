import type { ConnectionKind, CredentialDraft } from '@/features/connection/model/connection'

/** An empty credential of the kind the connection takes. */
export function emptyDraft(kind: ConnectionKind): CredentialDraft {
  return kind === 'GITLAB' ? { kind, token: '' } : { kind, kubeconfig: '', context: null }
}

/** Whether the draft holds enough to send: a token, or a kubeconfig with a context chosen. */
export function isComplete(draft: CredentialDraft): boolean {
  return draft.kind === 'GITLAB'
    ? draft.token.trim().length > 0
    : draft.kubeconfig.trim().length > 0 && draft.context !== null
}

/** Whether anything at all was typed, so an untouched field is not sent. */
export function isStarted(draft: CredentialDraft): boolean {
  return draft.kind === 'GITLAB' ? draft.token.length > 0 : draft.kubeconfig.length > 0
}
