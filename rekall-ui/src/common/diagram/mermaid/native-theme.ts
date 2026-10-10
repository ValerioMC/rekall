/**
 * The theme Mermaid draws its own types in, made from the console's tokens so a mindmap or a
 * Gantt sits in the same palette as the canvas. Mermaid computes shades itself and only reads
 * hex colours, so the kind tints are mixed into the surface here; a token that is not hex is left out.
 */
export type TokenReader = (token: string) => string

const SCALE_TOKENS = [
  '--color-kind-action',
  '--color-kind-decision',
  '--color-kind-concept',
  '--color-kind-event',
  '--color-kind-state',
  '--color-kind-data',
  '--color-kind-external',
  '--color-kind-custom'
] as const

/** How much of a kind tint a filled shape takes over the raised surface: enough to tell them apart, dark enough for light text. */
const TINT_SHARE = 0.38

export function nativeThemeVariables(read: TokenReader): Record<string, string | boolean> {
  const canvas = hex(read('--color-canvas'))
  const surface = hex(read('--color-surface-raised'))
  const text = hex(read('--color-text'))
  const subtle = hex(read('--color-text-subtle'))
  const variables: Record<string, string | boolean> = { darkMode: true, background: 'transparent', fontFamily: read('--font-sans') || 'sans-serif' }
  if (surface) Object.assign(variables, { primaryColor: surface, secondaryColor: surface, tertiaryColor: canvas ?? surface, mainBkg: surface })
  if (text) Object.assign(variables, { primaryTextColor: text, textColor: text, titleColor: text })
  if (subtle) Object.assign(variables, { lineColor: subtle, primaryBorderColor: subtle })
  if (!surface || !text) return variables
  SCALE_TOKENS.forEach((token, index) => {
    const tint = hex(read(token))
    if (!tint) return
    variables[`cScale${index}`] = mix(tint, surface, TINT_SHARE)
    variables[`cScaleLabel${index}`] = text
  })
  return variables
}

function hex(value: string): string | null {
  const trimmed = value.trim()
  return /^#[0-9a-f]{6}$/i.test(trimmed) ? trimmed.toLowerCase() : null
}

/** `share` of `tint` over `base`, both `#rrggbb`. */
export function mix(tint: string, base: string, share: number): string {
  const channel = (color: string, at: number): number => parseInt(color.slice(at, at + 2), 16)
  const mixed = [1, 3, 5].map((at) => Math.round(channel(tint, at) * share + channel(base, at) * (1 - share)))
  return `#${mixed.map((value) => value.toString(16).padStart(2, '0')).join('')}`
}
