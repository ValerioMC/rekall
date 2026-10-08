const COLLAPSED_KEY = 'rekall.projectComponents.collapsed'

/** Whether the project page folds its components away, remembered per browser. */
export function componentsPanelCollapsed(): boolean {
  try {
    return window.localStorage.getItem(COLLAPSED_KEY) === 'true'
  } catch {
    return false
  }
}

export function setComponentsPanelCollapsed(value: boolean): void {
  try {
    window.localStorage.setItem(COLLAPSED_KEY, String(value))
  } catch {
    // Private windows refuse storage; the panel still folds, it just is not remembered.
  }
}
