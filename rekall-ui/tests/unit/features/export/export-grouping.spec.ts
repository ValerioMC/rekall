import { describe, expect, it } from 'vitest'
import { groupExports } from '@/features/export/lib/export-grouping'
import { filterExports, isNarrowing, NO_EXPORT_FILTER } from '@/features/export/lib/export-filter'
import { asCompanyId, asProjectExportId, asProjectId } from '@/model/branded'
import { entryOf } from './export-fixtures'

const VFORGE = asCompanyId('d0000000-0000-4000-8000-00000000000a')
const NORTHWIND = asCompanyId('d0000000-0000-4000-8000-00000000000b')
const VEGA = asProjectId('11111111-1111-4111-8111-11111111111a')
const ATLAS = asProjectId('11111111-1111-4111-8111-11111111111b')
const KESTREL = asProjectId('11111111-1111-4111-8111-11111111111c')

function at(id: string, createdAt: string, overrides: Parameters<typeof entryOf>[0] = {}) {
  return entryOf({ id: asProjectExportId(`e0000000-0000-4000-8000-00000000000${id}`), createdAt, companyId: VFORGE, companyName: 'VFORGE', projectId: VEGA, projectLabel: 'vega', projectTitle: 'Vega', ...overrides })
}

const VEGA_OLD = at('1', '2026-11-20T10:00:00Z', { summary: 'Blueprint' })
const VEGA_NEW = at('2', '2026-11-25T10:00:00Z', { summary: 'Layers', format: 'DOCX' })
const ATLAS_MID = at('3', '2026-11-22T10:00:00Z', { projectId: ATLAS, projectLabel: 'atlas', projectTitle: 'Atlas', status: 'FAILED', failureKind: 'RENDER', failureMessage: 'Render error', summary: 'Pages' })
const KESTREL_LATEST = at('4', '2026-11-26T10:00:00Z', { companyId: NORTHWIND, companyName: 'Northwind', projectId: KESTREL, projectLabel: 'kestrel', projectTitle: 'Kestrel', summary: 'Everything' })

describe('groupExports', () => {
  const groups = groupExports([VEGA_OLD, ATLAS_MID, KESTREL_LATEST, VEGA_NEW])

  it('groups by company, then by project, the group with the newest export first', () => {
    expect(groups.map((company) => company.companyName)).toEqual(['Northwind', 'VFORGE'])
    expect(groups[1]?.projects.map((project) => project.projectTitle)).toEqual(['Vega', 'Atlas'])
  })

  it('puts the newest row first inside a project', () => {
    expect(groups[1]?.projects[0]?.entries.map((entry) => entry.summary)).toEqual(['Layers', 'Blueprint'])
  })

  it('counts what is in each group and how much of it failed', () => {
    expect(groups[1]).toMatchObject({ count: 3, failedCount: 1 })
    expect(groups[1]?.projects.map((project) => project.failedCount)).toEqual([0, 1])
  })

  it('gives nothing for no exports', () => {
    expect(groupExports([])).toEqual([])
  })
})

describe('filterExports', () => {
  const all = [VEGA_OLD, VEGA_NEW, ATLAS_MID, KESTREL_LATEST]

  it('keeps everything when nothing narrows it', () => {
    expect(filterExports(all, NO_EXPORT_FILTER)).toHaveLength(4)
    expect(isNarrowing(NO_EXPORT_FILTER)).toBe(false)
  })

  it('filters by status and by format', () => {
    expect(filterExports(all, { ...NO_EXPORT_FILTER, status: 'failed' })).toEqual([ATLAS_MID])
    expect(filterExports(all, { ...NO_EXPORT_FILTER, format: 'DOCX' })).toEqual([VEGA_NEW])
  })

  it('searches what was exported, the file, the project and the company without regard to case', () => {
    expect(filterExports(all, { ...NO_EXPORT_FILTER, query: 'layers' })).toEqual([VEGA_NEW])
    expect(filterExports(all, { ...NO_EXPORT_FILTER, query: 'NORTHWIND' })).toEqual([KESTREL_LATEST])
    expect(isNarrowing({ ...NO_EXPORT_FILTER, query: 'x' })).toBe(true)
  })
})
