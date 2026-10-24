import type { DiscoveredRepository, DiscoveryOutcome } from '@/features/connection/model/connection'

/** What a discovery did with a repository, in the words its row shows. */
export const DISCOVERY_OUTCOME_LABEL: Readonly<Record<DiscoveryOutcome, string>> = {
  BOUND: 'Connected',
  ALREADY_BOUND: 'Already connected',
  NO_ORIGIN: 'No origin remote',
  NO_CONNECTION: 'No GitLab connection for its host',
  NO_CREDENTIAL: 'Its GitLab connection has no token'
}

/** The repository's folder below the project's, or the folder itself when it is the project's. */
export function relativeFolder(root: string, repository: DiscoveredRepository): string {
  const base = root.replace(/[/\\]+$/, '')
  if (repository.folder === base) return '.'
  return repository.folder.startsWith(base) ? repository.folder.slice(base.length + 1) : repository.folder
}
