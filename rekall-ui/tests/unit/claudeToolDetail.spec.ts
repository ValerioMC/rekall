import { describe, expect, it } from 'vitest'
import { claudeToolDetail, parseToolMeta } from '@/model/claude'

describe('claudeToolDetail', () => {
  it('picks the field that says what a known tool did', () => {
    expect(claudeToolDetail('Bash', '{"command":"ls -la","description":"list"}')).toBe('ls -la')
    expect(claudeToolDetail('Read', '{"file_path":"/a/b.ts","limit":40}')).toBe('/a/b.ts')
    expect(claudeToolDetail('Grep', '{"pattern":"TODO","path":"src"}')).toBe('TODO')
  })

  it('collapses whitespace so the row stays one line', () => {
    expect(claudeToolDetail('Bash', '{"command":"git add .\\ngit commit"}')).toBe('git add . git commit')
  })

  it('falls back to the first string field for an unknown tool', () => {
    expect(claudeToolDetail('Whatever', '{"count":3,"target":"the thing"}')).toBe('the thing')
  })

  it('returns null when there is nothing worth showing', () => {
    expect(claudeToolDetail('Bash', null)).toBeNull()
    expect(claudeToolDetail('Bash', 'not json')).toBeNull()
    expect(claudeToolDetail('Bash', '{"count":3}')).toBeNull()
  })
})

describe('parseToolMeta', () => {
  it('reads the pairing id and the error flag', () => {
    expect(parseToolMeta('{"toolUseId":"tu_1","error":true}')).toEqual({ toolUseId: 'tu_1', error: true })
  })

  it('is an empty object for null or junk', () => {
    expect(parseToolMeta(null)).toEqual({})
    expect(parseToolMeta('{')).toEqual({})
  })
})
