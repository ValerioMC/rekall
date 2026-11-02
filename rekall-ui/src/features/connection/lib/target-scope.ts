/**
 * The prefix a target is expected to start with when its connection covers a root path, or
 * `null` when the target is empty, there is no root path, or the target already sits under it.
 * Case is ignored, since GitHub owners and GitLab groups are case-insensitive.
 */
export function expectedTargetPrefix(rootPath: string | null, target: string | null): string | null {
  if (!rootPath || !target) return null
  const prefix = `${rootPath}/`
  return target.trim().toLowerCase().startsWith(prefix.toLowerCase()) ? null : prefix
}
