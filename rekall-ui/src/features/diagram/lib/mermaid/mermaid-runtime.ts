import type { Mermaid, MermaidConfig } from 'mermaid'
import { paperTokenReader, type DiagramPalette } from '../diagram-palette'
import { nativeThemeVariables } from './native-theme'

/**
 * Mermaid, loaded on first use. The console uses it for two things only: reading a sheet into
 * Mermaid's own model, which the scene readers translate, and drawing the types it has no
 * components for.
 */

const PIE_SLICES = 8

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
export async function renderMermaid(source: string, palette: DiagramPalette = 'console'): Promise<string> {
  renders += 1
  const { svg } = await (await mermaid()).render(`rk-diagram-sheet-${renders}`, themed(source, palette))
  return svg
}

/** Mermaid's settings made from the console's tokens, for every Mermaid drawing that is not the console's own. */
export function consoleMermaidConfig(): MermaidConfig {
  const styles = getComputedStyle(document.documentElement)
  const themeVariables = nativeThemeVariables((token) => styles.getPropertyValue(token))
  return { startOnLoad: false, theme: 'base', themeVariables }
}

/** Mermaid's settings for the paper palette. Labels are plain SVG text: a foreignObject would taint the canvas the export draws on. */
export function paperMermaidConfig(): MermaidConfig {
  const themeVariables = { ...nativeThemeVariables(paperTokenReader, false), ...pieSlices() }
  return { theme: 'base', themeVariables, htmlLabels: false, flowchart: { htmlLabels: false } }
}

/** Front matter has to stay first, so a sheet that carries its own keeps its own config. */
function themed(source: string, palette: DiagramPalette): string {
  if (source.trimStart().startsWith('---')) return source
  return `%%{init: ${JSON.stringify(initOf(palette))}}%%\n${source}`
}

function pieSlices(): Record<string, string> {
  const scale = nativeThemeVariables(paperTokenReader, false)
  return Object.fromEntries(Array.from({ length: PIE_SLICES }, (_, index) => [`pie${index + 1}`, String(scale[`cScale${index}`])]))
}

function initOf(palette: DiagramPalette): MermaidConfig {
  if (palette === 'paper') return paperMermaidConfig()
  const { theme, themeVariables } = consoleMermaidConfig()
  return { theme, themeVariables }
}
