const MERGE_REQUEST_PATH = '/merge_requests/'

export type RequestKind = { readonly noun: 'pull request' | 'merge request'; readonly short: 'PR' | 'MR'; readonly prefix: '#' | '!' }

const PULL_REQUEST: RequestKind = { noun: 'pull request', short: 'PR', prefix: '#' }
const MERGE_REQUEST: RequestKind = { noun: 'merge request', short: 'MR', prefix: '!' }

/** GitLab serves a merge request under `/merge_requests/`; any other host calls it a pull request. */
export function requestKindOf(url: string | null): RequestKind {
  return url !== null && url.includes(MERGE_REQUEST_PATH) ? MERGE_REQUEST : PULL_REQUEST
}
