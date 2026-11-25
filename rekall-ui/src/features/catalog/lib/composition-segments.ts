import type { ProjectComposition } from '@/features/catalog/lib/project-composition'

export interface CompositionSegment {
  readonly key: keyof ProjectComposition
  readonly label: string
  readonly tube: string
}

export const COMPOSITION_SEGMENTS: readonly CompositionSegment[] = [
  { key: 'filed', label: 'filed', tube: 'var(--color-safe)' },
  { key: 'review', label: 'to review', tube: 'var(--color-accent)' },
  { key: 'active', label: 'in progress', tube: 'var(--color-session)' },
  { key: 'todo', label: 'to do', tube: 'var(--color-border-strong)' }
]
