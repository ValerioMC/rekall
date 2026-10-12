import { describe, expect, it } from 'vitest'
import { findingCount, type MemoryAnalysis } from '@/model/memory-analysis'
import { sectionOfPath } from '@/model/claude-resources'

describe('the memory analysis', () => {
  it('counts every suggestion, promotions and obsolete memories alike', () => {
    const analysis = { memoriesRead: 9, documentsRead: 3, promotions: [{}, {}], obsolete: [{}] } as unknown as MemoryAnalysis
    expect(findingCount(analysis)).toBe(3)
  })

  it('finds the section a path is listed under, so a finding opens in its place', () => {
    expect(sectionOfPath('skills/docx/SKILL.md')).toBe('skills')
    expect(sectionOfPath('plugins/cache/m/p/1/skills/s/SKILL.md')).toBe('skills')
    expect(sectionOfPath('projects/-work/memory/x.md')).toBe('memories')
    expect(sectionOfPath('memory/x.md')).toBe('memories')
    expect(sectionOfPath('commands/rk.md')).toBe('commands')
    expect(sectionOfPath('agents/a.md')).toBe('agents')
    expect(sectionOfPath('CLAUDE.md')).toBeNull()
  })
})
