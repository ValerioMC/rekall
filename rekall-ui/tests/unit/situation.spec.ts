import { describe, expect, it } from 'vitest'
import { situationOf } from '@/common/home/situation'

describe('situationOf', () => {
  it('says all is quiet when nothing runs and nothing waits', () => {
    expect(situationOf(0, 0)).toEqual({
      working: 'Nothing running,',
      waiting: 'nothing waiting for you.',
      urgent: false
    })
  })

  it('counts sessions in the singular and the plural', () => {
    expect(situationOf(1, 0).working).toBe('1 session working,')
    expect(situationOf(3, 0).working).toBe('3 sessions working,')
  })

  it('marks the sentence urgent when work waits for review', () => {
    expect(situationOf(2, 3)).toEqual({
      working: '2 sessions working,',
      waiting: '3 waiting for your review.',
      urgent: true
    })
  })
})
