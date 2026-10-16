import type { Mermaid } from 'mermaid'
import { nativeThemeVariables } from './native-theme'

/**
 * Mermaid, loaded on first use. The console uses it for two things only: reading a sheet into
 * Mermaid's own model, which the scene readers translate, and drawing the types it has no
 * components for.
 */

let loading: Promise<Mermaid> | null = null

function mermaid(): Promise<Mermaid> {
  loading ??= import('mermaid').then(({ default: loaded }) => {
    loaded.initialize({ startOnLoad: false, theme: 'dark' })
    return loaded
  })
  return loading
}

/** Mermaid's parsed model of `source`: its type name and the database its parser filled. */
export async function parseMermaid(source: string): Promise<{ readonly type: string; readonly db: unknown }> {
  const parsed = await (await mermaid()).mermaidAPI.getDiagramFromText(source)
  return { type: parsed.type, db: parsed.db }
}

let renders = 0

/** Mermaid's own SVG for `source`, sanitised by Mermaid at its default security level. */
export async function renderMermaid(source: string): Promise<string> {
  renders += 1
  const { svg } = await (await mermaid()).render(`rk-diagram-sheet-${renders}`, themed(source))
  return svg
}

/** Front matter has to stay first, so a sheet that carries its own keeps its own config. */
function themed(source: string): string {
  if (source.trimStart().startsWith('---')) return source
  const styles = getComputedStyle(document.documentElement)
  const themeVariables = nativeThemeVariables((token) => styles.getPropertyValue(token))
  return `%%{init: ${JSON.stringify({ theme: 'base', themeVariables })}}%%\n${source}`
}
