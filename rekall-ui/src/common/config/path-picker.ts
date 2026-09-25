const HIDDEN_KEY = 'rekall.pathPicker.showHidden'

/** Whether the path picker lists dotfiles, remembered per browser like Finder's own toggle. */
export function pathPickerShowsHidden(): boolean {
  try {
    return window.localStorage.getItem(HIDDEN_KEY) === 'true'
  } catch {
    return false
  }
}

export function setPathPickerShowsHidden(value: boolean): void {
  try {
    window.localStorage.setItem(HIDDEN_KEY, String(value))
  } catch {
    // Private windows refuse storage; the toggle still works for this picker, it just is not remembered.
  }
}
