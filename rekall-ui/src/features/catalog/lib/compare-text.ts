/** Name order for lists: case-blind and numbers by value, so `task 2` comes before `task 10`. */
export function compareText(a: string, b: string): number {
  return a.localeCompare(b, undefined, { numeric: true, sensitivity: 'base' })
}
