import { describe, expect, it } from 'vitest'
import { switcherSections } from '@/features/catalog/lib/switcher-sections'
import type { Project } from '@/features/catalog/model/catalog'
import type { CompanyId, ProjectId } from '@/model/branded'

function project(id: string, title: string, companyName: string, status: Project['status'] = 'ACTIVE'): Project {
  return {
    id: id as ProjectId,
    label: title.toLowerCase(),
    taskKey: 'PR',
    title,
    status,
    icon: 'folder',
    iconImageVersion: null,
    description: null,
    blueprintMarkdown: null,
    repoFolder: null,
    autoCommit: false,
    mainBranch: null,
    pullRequestFlow: false,
    companyId: companyName as CompanyId,
    companyName,
    taskCount: 0,
    anchor: `project:${title.toLowerCase()}`,
    updatedAt: ''
  }
}

const vega = project('p1', 'Vega', 'Acme')
const atlas = project('p2', 'Atlas', 'Acme')
const lumen = project('p3', 'Lumen', 'Globex')
const old = project('p4', 'Old', 'Globex', 'ARCHIVED')
const all = [vega, atlas, lumen, old]

describe('the switcher sections', () => {
  it('lists the recent projects first, newest first, and every company with its projects', () => {
    const sections = switcherSections(all, ['p3' as ProjectId, 'p1' as ProjectId], '', null)

    expect(sections.recent.map((p) => p.title)).toEqual(['Lumen', 'Vega'])
    expect(sections.companies.map((c) => [c.companyName, c.projects.map((p) => p.title)])).toEqual([
      ['Acme', ['Atlas', 'Vega']],
      ['Globex', ['Lumen']]
    ])
  })

  it('shows at most three recent projects and skips one that no longer exists', () => {
    const ids = ['gone', 'p1', 'p2', 'p3'] as ProjectId[]

    expect(switcherSections(all, ids, '', null).recent.map((p) => p.title)).toEqual(['Vega', 'Atlas', 'Lumen'])
  })

  it('leaves an archived project out unless it is the one open', () => {
    const ids = ['p4' as ProjectId]

    expect(switcherSections(all, ids, '', null).recent).toEqual([])
    expect(switcherSections(all, ids, '', null).companies.flatMap((c) => c.projects)).not.toContain(old)
    expect(switcherSections(all, ids, '', old.id).recent).toEqual([old])
    expect(switcherSections(all, ids, '', old.id).companies.flatMap((c) => c.projects)).toContain(old)
  })

  it('filters by company, title or anchor and drops Recent while searching', () => {
    const sections = switcherSections(all, ['p1' as ProjectId], 'globex', null)

    expect(sections.recent).toEqual([])
    expect(sections.companies.map((c) => c.projects.map((p) => p.title))).toEqual([['Lumen']])
    expect(switcherSections(all, [], 'project:atl', null).companies[0]?.projects).toEqual([atlas])
  })

  it('returns no company when nothing matches', () => {
    expect(switcherSections(all, [], 'zzz', null).companies).toEqual([])
  })
})
