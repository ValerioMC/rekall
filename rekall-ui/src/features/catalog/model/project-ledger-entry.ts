import type { ProjectComposition } from '@/features/catalog/lib/project-composition'
import type { Project } from '@/features/catalog/model/catalog'
import type { TokenTotals } from '@/features/tokens/model/token-usage'

export interface ProjectLedgerEntry {
  readonly project: Project
  readonly composition: ProjectComposition
  readonly reviewing: number
  readonly tokens: TokenTotals
}
