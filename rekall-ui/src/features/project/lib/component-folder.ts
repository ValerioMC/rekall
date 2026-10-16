/** The last segment of a folder path: what a component is called until someone names it. */
export function folderName(folder: string): string {
  const segments = folder.trim().split(/[\\/]+/).filter((segment) => segment !== '')
  return segments.at(-1) ?? folder.trim()
}
