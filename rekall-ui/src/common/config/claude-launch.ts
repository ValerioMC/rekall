const STORAGE_KEY = 'rekall.claude.skip-permissions'

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
  }
}
