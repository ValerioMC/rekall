/** A CSS length in pixels (`'520px'`) as a number; `fallback` for any other unit. */
export function pixelsOf(length: string, fallback: number): number {
  const match = /^(\d+(?:\.\d+)?)px$/.exec(length.trim())
  return match ? Number(match[1]) : fallback
}

/** Editor height that fits the text exactly, never below `minimum`: the text plus the toolbar and footer around it. */
export function fittedEditorHeight(contentHeight: number, chromeHeight: number, minimum: number): number {
  return Math.max(minimum, Math.ceil(contentHeight + chromeHeight))
}
