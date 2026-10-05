/** A CSS length in pixels (`'520px'`) as a number; `fallback` for any other unit. */
export function pixelsOf(length: string, fallback: number): number {
  const match = /^(\d+(?:\.\d+)?)px$/.exec(length.trim())
  return match ? Number(match[1]) : fallback
}

/** Editor height that fits the text exactly, never below `minimum`: the text plus the toolbar and footer around it. */
export function fittedEditorHeight(contentHeight: number, chromeHeight: number, minimum: number): number {
  return Math.max(minimum, Math.ceil(contentHeight + chromeHeight))
}

/** The editor's least height as the window allows it: what is left under the page's own chrome, never below `floor`. */
export function viewportEditorHeight(viewportHeight: number, chromeHeight: number, floor: number): string {
  return `${Math.max(floor, Math.floor(viewportHeight - chromeHeight))}px`
}
