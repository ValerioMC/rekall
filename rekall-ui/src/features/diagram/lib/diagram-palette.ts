/**
 * Which colours a diagram is drawn in. The console shows it on its dark tokens; an export is read on
 * paper, so it is drawn again on a fixed light palette and never shares a cache entry with the screen.
 */
export type DiagramPalette = 'console' | 'paper'

/** White ground, dark lines and text, the kind tints deepened so they hold on white. Only the tokens a drawing reads. */
export const PAPER_TOKENS: Readonly<Record<string, string>> = {
  '--color-canvas': '#ffffff',
  '--color-surface': '#f6f8fa',
  '--color-surface-raised': '#ffffff',
  '--color-border-strong': '#9aa4b0',
  '--color-text': '#1f2328',
  '--color-text-muted': '#4a5560',
  '--color-text-subtle': '#5a6470',
  '--color-accent': '#9a5b12',
  '--color-accent-strong': '#7a470c',
  '--color-kind-concept': '#0f8a76',
  '--color-kind-action': '#1f6fb8',
  '--color-kind-decision': '#b3307c',
  '--color-kind-state': '#6f47c4',
  '--color-kind-event': '#c4601f',
  '--color-kind-data': '#3f8a2b',
  '--color-kind-external': '#5f6f86',
  '--color-kind-code': '#5a6470',
  '--color-kind-custom': '#85764f',
  '--font-sans': 'Fira Sans, ui-sans-serif, sans-serif',
  '--font-mono': 'Fira Code Variable, ui-monospace, monospace'
}

export function paperTokenReader(token: string): string {
  return PAPER_TOKENS[token] ?? ''
}
