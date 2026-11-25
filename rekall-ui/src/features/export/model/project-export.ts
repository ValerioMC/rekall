import type { DiagramImage } from '@/features/export/model/diagram-image'
import type { CompanyId, ProjectComponentId, ProjectExportId, ProjectId, ProjectSectionId } from '@/model/branded'

export const EXPORT_FORMATS = ['PDF', 'DOCX'] as const
export type ExportFormat = (typeof EXPORT_FORMATS)[number]

export type ExportStatus = 'DONE' | 'FAILED'
export type ExportFailureKind = 'RENDER' | 'STORAGE'

/** One export as the Exports page lists it, with the project and company it is grouped by. */
export interface ProjectExportEntry {
  readonly id: ProjectExportId
  readonly projectId: ProjectId
  readonly projectLabel: string
  readonly projectTitle: string
  readonly companyId: CompanyId
  readonly companyName: string
  readonly format: ExportFormat
  readonly status: ExportStatus
  readonly summary: string
  readonly fileName: string
  readonly sizeBytes: number
  readonly diagramsMissing: number
  readonly failureKind: ExportFailureKind | null
  readonly failureMessage: string | null
  readonly createdAt: string
}

/** Which exports a bulk delete reaches: a company's, a project's, those with a status; every field given narrows it. */
export interface ProjectExportClear {
  readonly companyId?: CompanyId
  readonly projectId?: ProjectId
  readonly status?: ExportStatus
}

/** What is ticked in the dialog: the blueprint, and the pages and components by id. */
export interface ExportSelection {
  readonly blueprint: boolean
  readonly sectionIds: readonly ProjectSectionId[]
  readonly componentIds: readonly ProjectComponentId[]
}

export interface ProjectExportRequest extends ExportSelection {
  readonly format: ExportFormat
  readonly diagrams: readonly DiagramImage[]
}
