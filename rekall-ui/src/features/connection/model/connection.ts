import type { CompanyId, ConnectionId, ProjectId, TaskId, TerminalId } from '@/model/branded'

export type ConnectionKind = 'GITLAB' | 'KUBERNETES'

export const CONNECTION_KINDS: readonly ConnectionKind[] = ['GITLAB', 'KUBERNETES']

export const CONNECTION_KIND_LABEL: Readonly<Record<ConnectionKind, string>> = {
  GITLAB: 'GitLab',
  KUBERNETES: 'Kubernetes'
}

/** What one connection of a kind is called, as a group's add button and empty line name it. */
export const CONNECTION_KIND_NOUN: Readonly<Record<ConnectionKind, string>> = {
  GITLAB: 'GitLab instance',
  KUBERNETES: 'cluster'
}

/**
 * A system outside Rekall a company's projects reach. The console never holds its credential:
 * it only learns when one was stored, from `credentialSetAt`.
 */
export interface Connection {
  readonly id: ConnectionId
  readonly companyId: CompanyId
  readonly kind: ConnectionKind
  readonly label: string
  readonly title: string
  /** The GitLab instance, or the server the imported kubeconfig names. */
  readonly baseUrl: string | null
  readonly kubeContext: string | null
  readonly credentialSetAt: string | null
  readonly projectCount: number
  readonly updatedAt: string
}

/** A project that binds a connection, and what it uses it for. */
export interface ConnectionProject {
  readonly projectId: ProjectId
  readonly label: string
  readonly title: string
  readonly target: string | null
}

export interface ConnectionDetail extends Connection {
  readonly projects: readonly ConnectionProject[]
}

/** A connection a project binds, as its page shows it. */
export interface ProjectConnection {
  readonly connectionId: ConnectionId
  readonly projectId: ProjectId
  readonly kind: ConnectionKind
  readonly label: string
  readonly title: string
  readonly baseUrl: string | null
  readonly kubeContext: string | null
  /** The GitLab project path or the Kubernetes namespace this project uses it for. */
  readonly target: string | null
  readonly credentialSetAt: string | null
}

/** A field left out on an edit keeps what the connection has; the kind is fixed once created. */
export interface ConnectionInput {
  readonly kind?: ConnectionKind
  readonly title?: string
  readonly label?: string
  readonly baseUrl?: string
}

/**
 * What the person typed or pasted as a credential, held only until it is sent. Never kept in a
 * store: a component drops it as soon as the request returns.
 */
export type CredentialDraft =
  | { readonly kind: 'GITLAB'; readonly token: string }
  | { readonly kind: 'KUBERNETES'; readonly kubeconfig: string; readonly context: string | null }

export type KubeUserKind =
  'token' | 'client-certificate' | 'exec' | 'auth-provider' | 'basic-auth' | 'nothing'

/** One context of a kubeconfig being imported, offered for choosing. */
export interface KubeContextOption {
  readonly name: string
  readonly server: string | null
  readonly namespace: string | null
  readonly userKind: KubeUserKind
  readonly supported: boolean
  readonly refusal: string | null
}

export const KUBE_USER_KIND_LABEL: Readonly<Record<KubeUserKind, string>> = {
  token: 'token',
  'client-certificate': 'client certificate',
  exec: 'runs a command',
  'auth-provider': 'auth provider',
  'basic-auth': 'username and password',
  nothing: 'no credential'
}

/** Whether a stored credential still opens its server, as the check found. Shown, never stored. */
export interface ConnectionCheck {
  readonly ok: boolean
  readonly detail: string
  readonly checkedAt: string
}

/** One request a terminal made for the credential. */
export interface ConnectionAccess {
  readonly id: string
  readonly accessedAt: string
  readonly granted: boolean
  readonly reason: string | null
  readonly terminalId: string
  readonly taskId: TaskId | null
  readonly taskLabel: string | null
  readonly taskTitle: string | null
}

/** What deleting a connection took with it. */
export interface ConnectionRemoved {
  readonly unboundProjects: number
}

/** The `connection` frame of the console's event stream. */
export interface ConnectionStreamEvent {
  readonly companyId: CompanyId
}

/** The `credential-access` frame: a terminal's helper asked for a connection's credential. */
export interface CredentialAccessEvent {
  readonly terminalId: TerminalId
  readonly connectionId: ConnectionId
  readonly granted: boolean
}

/** What one terminal did with one connection since the console opened. */
export interface CredentialAccessTally {
  readonly granted: number
  readonly refused: number
  /** Changes with every access, so a chip can replay its pulse. */
  readonly last: number
}
