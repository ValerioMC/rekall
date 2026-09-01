/**
 * The line that loads an anchor, ready to paste into Claude Code.
 *
 * <p>An anchor chip is copied to be pasted into a session, and a bare `project:x task:y` is not
 * a command there: it has to have `/rk` typed in front of it every single time. So the chips
 * copy the whole line, and show the whole line, because a chip that copies more than it shows
 * is a chip nobody trusts twice.
 */
export function rkCommand(anchor: string): string {
  return anchor ? `/rk ${anchor}` : ''
}
