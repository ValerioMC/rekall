import { describe, expect, it } from 'vitest'
import { excerpt } from '@/common/format/excerpt'

/**
 * The preview line on the description, wrapup and note cards. Three cards read from one
 * function, because three copies of it is how two of them end up cutting differently.
 */
describe('excerpt', () => {
  it('leaves a short body alone, markup and all whitespace collapsed', () => {
    expect(excerpt('## Cosa deve fare\n\nIl report  builder gira.')).toBe(
      'Cosa deve fare Il report builder gira.'
    )
  })

  it('cuts on a word, not in the middle of one', () => {
    const body = 'Il report builder genera il report settimanale a partire dalle run della pipeline, e lo pubblica su S3.'

    const cut = excerpt(body, 60)

    expect(cut.endsWith('…')).toBe(true)
    expect(cut.length).toBeLessThanOrEqual(61)
    expect(cut).not.toContain('partir…')
  })

  /** A single word longer than the limit still has to give something back. */
  it('cuts inside a word when there is no earlier space to cut at', () => {
    expect(excerpt('Antidisestablishmentarianism', 12)).toBe('Antidisestab…')
  })

  it('says nothing about an empty body', () => {
    expect(excerpt('')).toBe('')
  })
})
