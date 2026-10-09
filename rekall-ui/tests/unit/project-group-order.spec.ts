import { describe, expect, it } from 'vitest'
import { compareProjects, type OrderedProject } from '@/common/catalog/project-group-order'

const project = (companyName: string, projectTitle: string, status: OrderedProject['status'] = 'ACTIVE'): OrderedProject => ({
  companyName,
  projectTitle,
  status
})

const titles = (projects: OrderedProject[]): string[] => projects.map((entry) => entry.projectTitle)

describe('project order in the navigator', () => {
  it('gathers each company’s projects together when every company is in view', () => {
    const projects = [project('Beta', 'Apollo'), project('Acme', 'Zephyr'), project('Beta', 'Borealis'), project('Acme', 'Atlas')]

    expect(titles(projects.sort(compareProjects(true)))).toEqual(['Atlas', 'Zephyr', 'Apollo', 'Borealis'])
  })

  it('keeps live projects ahead of paused and done ones inside a company', () => {
    const projects = [project('Acme', 'Atlas', 'DONE'), project('Acme', 'Zephyr'), project('Acme', 'Mira', 'PAUSED')]

    expect(titles(projects.sort(compareProjects(true)))).toEqual(['Zephyr', 'Mira', 'Atlas'])
  })

  it('puts archived projects after every other status', () => {
    const projects = [project('Acme', 'Atlas', 'ARCHIVED'), project('Acme', 'Mira', 'DONE'), project('Acme', 'Zephyr')]

    expect(titles(projects.sort(compareProjects(true)))).toEqual(['Zephyr', 'Mira', 'Atlas'])
  })

  it('ignores the company when the view is already narrowed to one', () => {
    const projects = [project('Beta', 'Apollo'), project('Acme', 'Zephyr')]

    expect(titles(projects.sort(compareProjects(false)))).toEqual(['Apollo', 'Zephyr'])
  })

  it('orders titles by number value, not character by character', () => {
    const projects = [project('Acme', 'Site 10'), project('Acme', 'Site 2')]

    expect(titles(projects.sort(compareProjects(true)))).toEqual(['Site 2', 'Site 10'])
  })
})
