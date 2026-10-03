import { describe, expect, it } from 'vitest'
import { groupProjectsByCompany, matchesProjectQuery } from '@/common/catalog/project-picker'
import type { Project } from '@/model/catalog'
import type { CompanyId, ProjectId } from '@/model/branded'

function project(id: string, title: string, companyName: string, status: Project['status'] = 'ACTIVE'): Project {
  return {
    id: id as ProjectId,
    label: title.toLowerCase().replace(/\s+/g, '-'),
    title,
    status,
    icon: 'folder',
    description: null,
    blueprintMarkdown: null,
    repoFolder: null,
    autoCommit: false,
    companyId: companyName as CompanyId,
    companyName,
    taskCount: 0,
    anchor: `project:${title.toLowerCase().replace(/\s+/g, '-')}`,
    updatedAt: ''
  }
}

describe('matchesProjectQuery', () => {
  const vega = project('p1', 'Vega Platform', 'Acme')

  it('matches every part of the needle across company, title and anchor', () => {
    expect(matchesProjectQuery(vega, 'acme vega')).toBe(true)
    expect(matchesProjectQuery(vega, 'project:vega')).toBe(true)
  })

  it('rejects a needle with one part nowhere in the project', () => {
    expect(matchesProjectQuery(vega, 'vega zeta')).toBe(false)
  })

  it('accepts an empty needle', () => {
    expect(matchesProjectQuery(vega, '  ')).toBe(true)
  })
})

describe('groupProjectsByCompany', () => {
  it('orders companies by name and active projects before the others', () => {
    const groups = groupProjectsByCompany([
      project('p1', 'Zulu', 'Beta'),
      project('p2', 'Alpha', 'Beta', 'PAUSED'),
      project('p3', 'Mid', 'Acme'),
      project('p4', 'Bravo', 'Beta')
    ])

    expect(groups.map((group) => group.companyName)).toEqual(['Acme', 'Beta'])
    expect(groups[1]!.projects.map((entry) => entry.title)).toEqual(['Bravo', 'Zulu', 'Alpha'])
  })

  it('returns nothing for no projects', () => {
    expect(groupProjectsByCompany([])).toEqual([])
  })
})
