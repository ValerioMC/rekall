import { describe, expect, it } from 'vitest'
import { groupByProject } from '@/features/workingset/lib/group-by-project'
import { company, open, project, task } from './fixtures'

const COMPANIES = [company('c2', 'Northwind'), company('c1', 'Helix Labs')]
const PROJECTS = [project('vega', 'Vega', 'c2'), project('atlas', 'Atlas', 'c2'), project('ledger', 'Ledger', 'c1')]
const TASKS = [task('t1', 'vega'), task('t2', 'vega'), task('t3', 'atlas'), task('t4', 'ledger')]

describe('grouping the open tasks by company and project', () => {
  it('orders companies by name and projects by title, and keeps each project\'s tasks in the order they were opened', () => {
    const groups = groupByProject([open('t2', 'vega'), open('t4', 'ledger'), open('t3', 'atlas'), open('t1', 'vega')], COMPANIES, PROJECTS, TASKS)

    expect(groups.map((group) => group.company.name)).toEqual(['Helix Labs', 'Northwind'])
    expect(groups[1]?.projects.map((group) => group.project.title)).toEqual(['Atlas', 'Vega'])
    expect(groups[1]?.projects[1]?.tasks.map((entry) => entry.id)).toEqual(['t2', 't1'])
  })

  it('drops a task the catalog does not hold, and a project left with none', () => {
    const groups = groupByProject([open('gone', 'atlas'), open('t1', 'vega')], COMPANIES, PROJECTS, TASKS)

    expect(groups.map((group) => group.projects.map((entry) => entry.project.title))).toEqual([['Vega']])
  })

  it('is empty when nothing is open', () => {
    expect(groupByProject([], COMPANIES, PROJECTS, TASKS)).toEqual([])
  })
})
