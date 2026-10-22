import type { ConnectionKind } from '@/features/connection/model/connection'

/** The URL a person types for a kind, or none when it comes from the credential (a kubeconfig). */
export interface UrlField {
  readonly title: string
  /** How an error names it. */
  readonly label: string
  readonly placeholder: string
}

export const URL_FIELD: Readonly<Record<ConnectionKind, UrlField | null>> = {
  GITLAB: { title: 'Instance URL', label: 'GitLab instance URL', placeholder: 'https://gitlab.example.com' },
  KUBERNETES: null,
  JIRA: { title: 'Site URL', label: 'Jira site URL', placeholder: 'https://example.atlassian.net' }
}

/** Only a Jira site is reached through a proxy typed in; a cluster's comes from its kubeconfig. */
export const TAKES_PROXY: Readonly<Record<ConnectionKind, boolean>> = {
  GITLAB: false,
  KUBERNETES: false,
  JIRA: true
}

/** How a kind's credential is asked for when a connection is created. */
export interface CredentialField {
  readonly title: string
  readonly hint: string
  /** Said when something was typed but not enough to send. */
  readonly incomplete: string
}

export const CREDENTIAL_FIELD: Readonly<Record<ConnectionKind, CredentialField>> = {
  GITLAB: {
    title: 'Personal access token',
    hint: "Optional now; stored in this machine's credential store.",
    incomplete: 'Type the token, or leave it empty to set it later.'
  },
  KUBERNETES: {
    title: 'Kubeconfig',
    hint: 'Optional now. A user that runs a command or a password is refused.',
    incomplete: 'Read the kubeconfig and choose a context, or leave it empty to set it later.'
  },
  JIRA: {
    title: 'Account email and API token',
    hint: "Optional now. An API token of the Atlassian account, stored in this machine's credential store.",
    incomplete: 'Type both the email and the API token, or leave them empty to set them later.'
  }
}

/** What the detail pane calls the address a connection points at. */
export const ADDRESS_TERM: Readonly<Record<ConnectionKind, string>> = {
  GITLAB: 'Instance',
  KUBERNETES: 'Server',
  JIRA: 'Site'
}

/** What a project narrows a connection of each kind to. */
export interface TargetField {
  /** What the input is, before the connection's title in its accessible name. */
  readonly name: string
  readonly placeholder: string
}

export const TARGET_FIELD: Readonly<Record<ConnectionKind, TargetField>> = {
  GITLAB: { name: 'GitLab project path', placeholder: 'group/project' },
  KUBERNETES: { name: 'Namespace', placeholder: 'namespace' },
  JIRA: { name: 'Jira project key', placeholder: 'project key, such as ENG' }
}
