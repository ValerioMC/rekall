import type { ExportFormat, ProjectExportEntry } from '@/features/export/model/project-export'

export type ExportStatusFilter = 'all' | 'done' | 'failed'
export type ExportFormatFilter = 'all' | ExportFormat

export interface ExportFilter {
  readonly query: string
  readonly status: ExportStatusFilter
  readonly format: ExportFormatFilter
}

export const NO_EXPORT_FILTER: ExportFilter = { query: '', status: 'all', format: 'all' }

export function isNarrowing(filter: ExportFilter): boolean {
  return filter.query.trim() !== '' || filter.status !== 'all' || filter.format !== 'all'
}

function matchesQuery(entry: ProjectExportEntry, query: string): boolean {
  const needle = query.trim().toLowerCase()
  if (needle === '') return true
  return [entry.summary, entry.fileName, entry.projectTitle, entry.projectLabel, entry.companyName].some((text) => text.toLowerCase().includes(needle))
}

export function filterExports(entries: readonly ProjectExportEntry[], filter: ExportFilter): ProjectExportEntry[] {
  return entries.filter(
    (entry) =>
      matchesQuery(entry, filter.query) &&
      (filter.status === 'all' || entry.status === filter.status.toUpperCase()) &&
      (filter.format === 'all' || entry.format === filter.format)
  )
}
