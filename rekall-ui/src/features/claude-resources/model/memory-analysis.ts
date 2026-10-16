/** A memory named by a finding. */
export interface MemoryRef {
  readonly path: string
  readonly name: string
  readonly groupId: string
  readonly memoryType: string | null
}

export type PromotionTarget = 'instructions' | 'skill'

/** Memories in several projects that say the same thing. */
export interface PromotionCandidate {
  readonly topic: readonly string[]
  readonly memories: readonly MemoryRef[]
  readonly groupCount: number
  readonly target: PromotionTarget
  readonly reason: string
}

export type CoveringKind = 'globalInstructions' | 'projectInstructions' | 'skill'

/** A `CLAUDE.md` or skill that already holds what a memory says. */
export interface CoveringDocument {
  readonly kind: CoveringKind
  readonly label: string
  readonly location: string
  /** Its path under the Claude folder when the console can open it. */
  readonly resourcePath: string | null
}

export interface ObsoleteMemory {
  readonly memory: MemoryRef
  readonly coveredBy: CoveringDocument
  readonly overlapPercent: number
  readonly sharedPhrases: readonly string[]
}

export interface MemoryAnalysis {
  readonly memoriesRead: number
  readonly documentsRead: number
  readonly promotions: readonly PromotionCandidate[]
  readonly obsolete: readonly ObsoleteMemory[]
}

export const TARGET_LABEL: Readonly<Record<PromotionTarget, string>> = {
  instructions: 'Add to CLAUDE.md',
  skill: 'Make a skill'
}

export const COVERING_LABEL: Readonly<Record<CoveringKind, string>> = {
  globalInstructions: 'Global CLAUDE.md',
  projectInstructions: 'Project CLAUDE.md',
  skill: 'Skill'
}

/** How many suggestions the review holds. */
export function findingCount(analysis: MemoryAnalysis): number {
  return analysis.promotions.length + analysis.obsolete.length
}
