const keyOf = (group: string): string => `rekall.projectInspector.${group}.collapsed`

/** Whether a group of the project page's inspector is folded, remembered per browser. */
export function inspectorGroupCollapsed(group: string): boolean {
  try {
    return window.localStorage.getItem(keyOf(group)) === 'true'
  } catch {
    return false
  }
}

export function setInspectorGroupCollapsed(group: string, value: boolean): void {
  try {
    window.localStorage.setItem(keyOf(group), String(value))
  } catch {
    // Private windows refuse storage; the group still folds, it just is not remembered.
  }
}
