import { describe, expect, it } from 'vitest'
import { alignedAddress, alignmentOf } from '@/features/picture/lib/picture-alignment'

const ADDRESS = '/api/images/acme/web/web-12/login-flow-3f9a1c2b.png'

describe('picture alignment', () => {
  it('reads it from the end of the address, left when nothing says so', () => {
    expect(alignmentOf(ADDRESS)).toBe('left')
    expect(alignmentOf(`${ADDRESS}#center`)).toBe('center')
    expect(alignmentOf(`${ADDRESS}#right`)).toBe('right')
    expect(alignmentOf(`${ADDRESS}#left`)).toBe('left')
    expect(alignmentOf('https://example.com/a.png#section')).toBe('left')
    expect(alignmentOf('<my picture.png#right>')).toBe('right')
  })

  it('writes it back, replacing the one before and writing nothing for left', () => {
    expect(alignedAddress(ADDRESS, 'center')).toBe(`${ADDRESS}#center`)
    expect(alignedAddress(`${ADDRESS}#center`, 'right')).toBe(`${ADDRESS}#right`)
    expect(alignedAddress(`${ADDRESS}#right`, 'left')).toBe(ADDRESS)
    expect(alignedAddress(ADDRESS, 'left')).toBe(ADDRESS)
  })

  it('keeps the rest of an address on the web, and stays inside angle brackets', () => {
    expect(alignedAddress('https://example.com/a.png?w=2#dark', 'right')).toBe('https://example.com/a.png?w=2#dark#right')
    expect(alignedAddress('<my picture.png#right>', 'center')).toBe('<my picture.png#center>')
  })
})
