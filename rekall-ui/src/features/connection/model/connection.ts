import type { CompanyId, ConnectionId, ProjectConnectionId, ProjectId, TaskId, TerminalId } from '@/model/branded'

export type ConnectionKind = 'GITLAB' | 'GITHUB' | 'KUBERNETES' | 'JIRA' | 'GRAFANA'

export const CONNECTION_KINDS: readonly ConnectionKind[] = ['GITLAB', 'GITHUB', 'KUBERNETES', 'JIRA', 'GRAFANA']

export const CONNECTION_KIND_LABEL: Readonly<Record<ConnectionKind, string>> = {
  GITLAB: 'GitLab',
  GITHUB: 'GitHub',
  KUBERNETES: 'Kubernetes',
  JIRA: 'Jira',
  GRAFANA: 'Grafana'
}

/** What one connection of a kind is called, as a group's add button and empty line name it. */
export const CONNECTION_KIND_NOUN: Readonly<Record<ConnectionKind, string>> = {
  GITLAB: 'GitLab instance',
  GITHUB: 'GitHub server',
  KUBERNETES: 'cluster',
  JIRA: 'Jira site',
  GRAFANA: 'Grafana instance'
}

/**
 * A system outside Rekall that projects reach, shared with the companies in `companyIds`. The
 * console never holds its credential: it only learns when one was stored, from `credentialSetAt`.
 */
export interface Connection {
  readonly id: ConnectionId
  readonly companyIds: readonly CompanyId[]
  readonly kind: ConnectionKind
  readonly label: string
  readonly title: string
  /** The GitLab instance, GitHub server, Jira site or Grafana instance, or the server the imported kubeconfig names. */
  readonly baseUrl: string | null
  readonly kubeContext: string | null
  /** Jira only: the proxy every call to the site goes through. */
  readonly proxyUrl: string | null
  /** Grafana only: the instance's certificate is not verified, because it serves a self-signed one. */
  readonly acceptSelfSigned: boolean
  /** GitLab and GitHub only: the group, namespace or owner the credential covers. */
  readonly rootPath: string | null
  readonly credentialSetAt: string | null
  readonly projectCount: number
  readonly updatedAt: string
}

/** A company a connection is shared with. */
export interface ConnectionCompany {
  readonly id: CompanyId
  readonly name: string
}

/** A project that binds a connection, and what it uses it for: one per binding. */
export interface ConnectionProject {
  readonly projectId: ProjectId
  readonly companyId: CompanyId
  readonly label: string
  readonly title: string
  readonly bindingLabel: string
  readonly target: string | null
}

export interface ConnectionDetail extends Connection {
  readonly companies: readonly ConnectionCompany[]
  readonly projects: readonly ConnectionProject[]
}

/**
 * A connection a project binds, as its page shows it. A project may bind one connection several
 * times, one target each, so the binding has its own id and label.
 */
export interface ProjectConnection {
  readonly id: ProjectConnectionId
  readonly connectionId: ConnectionId
  readonly projectId: ProjectId
  readonly kind: ConnectionKind
  /** The binding's own label, which a session names it by. */
  readonly label: string
  readonly title: string
  readonly baseUrl: string | null
  readonly kubeContext: string | null
  /** The GitLab project path, the GitHub repository, the Kubernetes namespace, the Jira project key or the Grafana folder UID this project uses it for. */
  readonly target: string | null
  /** The project writes its tasks to this binding's GitLab, GitHub or Jira project. At most one binding is. */
  readonly taskManager: boolean
  readonly credentialSetAt: string | null
}

/** A binding as it is sent: a label left out keeps the one it has, or starts as the connection's. */
export interface ProjectConnectionInput {
  readonly label?: string
  readonly target: string | null
  /** Left out, the binding stays as it is. */
  readonly taskManager?: boolean
}

export type DiscoveryOutcome = 'BOUND' | 'ALREADY_BOUND' | 'NO_ORIGIN' | 'NO_CONNECTION' | 'NO_CREDENTIAL'

/** One repository found under a project's folder, and what discovering it did. */
export interface DiscoveredRepository {
  readonly folder: string
  /** `host/group/name`, never the remote's URL. */
  readonly origin: string | null
  readonly outcome: DiscoveryOutcome
  readonly connectionLabel: string | null
  readonly bindingId: ProjectConnectionId | null
  readonly label: string | null
  readonly target: string | null
  /** The component made for its folder, when one was. */
  readonly component: string | null
}

export interface RepositoryDiscovery {
  readonly folder: string
  readonly repositories: readonly DiscoveredRepository[]
}

/** A field left out on an edit keeps what the connection has; the kind is fixed once created. */
export interface ConnectionInput {
  readonly kind?: ConnectionKind
  readonly title?: string
  readonly label?: string
  readonly baseUrl?: string
  /** Blank clears it. */
  readonly proxyUrl?: string
  /** Grafana only. */
  readonly acceptSelfSigned?: boolean
  /** GitLab and GitHub only; blank clears it. */
  readonly rootPath?: string
  /** Read on a create only: an existing connection is shared through `setConnectionCompanies`. */
  readonly companyIds?: readonly CompanyId[]
}

/** Jira Cloud signs in with an email and an API token, Data Center with a personal access token. */
export type JiraDeployment = 'CLOUD' | 'DATA_CENTER'

/** A Grafana instance is signed in to with a service account token, or with a user and its password. */
export type GrafanaAuthentication = 'SERVICE_ACCOUNT' | 'PASSWORD'

/**
 * What the person typed or pasted as a credential, held only until it is sent. Never kept in a
 * store: a component drops it as soon as the request returns.
 */
export type CredentialDraft =
  | { readonly kind: 'GITLAB'; readonly token: string }
  | { readonly kind: 'GITHUB'; readonly token: string }
  | { readonly kind: 'GRAFANA'; readonly authentication: GrafanaAuthentication; readonly user: string; readonly token: string }
  | { readonly kind: 'KUBERNETES'; readonly kubeconfig: string; readonly context: string | null }
  | { readonly kind: 'JIRA'; readonly deployment: JiraDeployment; readonly email: string; readonly token: string }

/**
 * A stored credential read back on request: a `token`, a `kubeconfig` (Kubernetes), on Jira Cloud
 * the `email` the token belongs to and on Grafana, signed in with a password, the `user` whose
 * password the `token` is. Held by the component that asked, never by a store.
 */
export interface RevealedCredential {
  readonly token: string | null
  readonly kubeconfig: string | null
  readonly email: string | null
  readonly user: string | null
}

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

/** An issue of a project's Jira project, and the task it was imported as, if it was. */
export interface JiraIssue {
  readonly key: string
  readonly summary: string
  readonly status: string | null
  readonly issueType: string | null
  readonly url: string
  readonly taskId: TaskId | null
}

/** What deleting a connection took with it. */
export interface ConnectionRemoved {
  readonly unboundProjects: number
}

/** The `connection` frame of the console's event stream. */
export interface ConnectionStreamEvent {
  readonly companyIds: readonly CompanyId[]
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
