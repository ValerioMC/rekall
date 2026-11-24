import { describe, expect, it } from 'vitest'
import { pageOptions } from '@/common/markdown/page-completion'
import {
  PAGE_CHIP_CLASS,
  pageChipHtml,
  pageNamed,
  pageReferenceTitle,
  paintPageReferences,
  type ReferencePage
} from '@/common/markdown/page-reference'

const PAGES: ReferencePage[] = [
  { title: 'Deployment', contextTitle: 'Operations', characters: 3180 },
  { title: 'Glossary', contextTitle: 'Team', characters: 1260 }
]

function chipFor(title: string): { root: HTMLElement; chip: HTMLElement } {
  const root = document.createElement('div')
  root.innerHTML = pageChipHtml(title)
  return { root, chip: root.querySelector<HTMLElement>(`.${PAGE_CHIP_CLASS}`)! }
}

describe('page references', () => {
  it('reads the title a reference names, and nothing from one that names something else', () => {
    expect(pageReferenceTitle('page:Release process')).toBe('Release process')
    expect(pageReferenceTitle('  page: Deployment ')).toBe('Deployment')
    expect(pageReferenceTitle('task:RK-12')).toBeNull()
    expect(pageReferenceTitle('Deployment')).toBeNull()
  })

  it('finds a page ignoring case and surrounding spaces', () => {
    expect(pageNamed(PAGES, '  deployment ')?.contextTitle).toBe('Operations')
    expect(pageNamed(PAGES, 'Ghost')).toBeUndefined()
  })

  it('settles a chip against the project: found with its context on hover, or missing', () => {
    const found = chipFor('glossary')
    paintPageReferences(found.root, PAGES)
    expect(found.chip.dataset['state']).toBe('found')
    expect(found.chip.title).toContain('Team')

    const missing = chipFor('Ghost')
    paintPageReferences(missing.root, PAGES)
    expect(missing.chip.dataset['state']).toBe('missing')
    expect(missing.chip.title).toContain('no page titled')
  })

  it('leaves a chip as plain text when the text has no project', () => {
    const plain = chipFor('Deployment')

    paintPageReferences(plain.root, null)

    expect(plain.chip.dataset['state']).toBe('plain')
    expect(plain.chip.hasAttribute('title')).toBe(false)
  })

  it('offers the pages that contain what was typed, each written as a reference to its title', () => {
    const typed = pageOptions(PAGES, 'dep')
    expect(typed.map((option) => option.label)).toEqual(['page:Deployment'])
    expect(typed[0]?.detail).toBe('Operations')
    expect(pageOptions(PAGES, '')).toHaveLength(2)
    expect(pageOptions(PAGES, 'zzz')).toEqual([])
  })
})
