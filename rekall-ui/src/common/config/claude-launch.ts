const STORAGE_KEY = 'rekall.claude.skip-permissions'

/**
 * Whether a session opened from a button skips Claude Code's permission prompts.
 *
 * Kept on the machine rather than in the database, and deliberately. "Run without asking" is a
 * property of this terminal on this Mac, not of the work: a database opened on a second machine,
 * or handed to someone else, has no business carrying that answer with it.
 *
 * Storage that throws is read as off. A browser with site data blocked, a private window and a
 * first run are the same answer here, and the safe one is the same in all three.
 */
export function skipsPermissions(): boolean {
  try {
    return window.localStorage.getItem(STORAGE_KEY) === 'true'
  } catch {
    return false
  }
}

export function setSkipsPermissions(value: boolean): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, String(value))
  } catch {
    // Nothing to do and nothing to say: the switch still holds for this window, and a machine
    // that cannot remember it is a machine that asks for permissions next time.
  }
}
