export const IDENTITY_HUE_COUNT = 6

export interface IdentityHue {
  readonly index: number
  readonly base: string
  readonly soft: string
  readonly line: string
}

function hashString(value: string): number {
  let h = 2166136261
  for (let i = 0; i < value.length; i++) {
    h ^= value.charCodeAt(i)
    h = Math.imul(h, 16777619)
  }
  return h >>> 0
}

export function identityIndex(id: string): number {
  return hashString(id) % IDENTITY_HUE_COUNT
}

export function identityHue(id: string): IdentityHue {
  const index = identityIndex(id)
  const n = index + 1
  return {
    index,
    base: `var(--color-identity-${n})`,
    soft: `var(--color-identity-${n}-soft)`,
    line: `var(--color-identity-${n}-line)`
  }
}
