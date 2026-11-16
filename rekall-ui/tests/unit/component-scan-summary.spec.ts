import { describe, expect, it } from 'vitest'
import { scanSummary } from '@/features/project/lib/component-scan-summary'
import type { ComponentScan, ProjectComponent } from '@/features/project/model/project-component'

function scanned(added: number, alreadyKnown: number): ComponentScan {
  return { folder: '/work', added: Array.from({ length: added }, () => ({}) as ProjectComponent), alreadyKnown }
}

describe('scanSummary', () => {
  it('counts what was added and what was already there', () => {
    expect(scanSummary(scanned(3, 2))).toBe('Added 3 components, 2 already there')
    expect(scanSummary(scanned(1, 0))).toBe('Added 1 component')
  })

  it('says so when every repository already had a component', () => {
    expect(scanSummary(scanned(0, 1))).toBe('Nothing new: 1 repository already had a component')
    expect(scanSummary(scanned(0, 4))).toBe('Nothing new: 4 repositories already had a component')
  })

  it('says so when the folder holds no repository', () => {
    expect(scanSummary(scanned(0, 0))).toBe('No git repository found under the project folder')
  })
})
