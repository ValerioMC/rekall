import { describe, expect, it } from 'vitest'
import { isAnchorRun, markAnchorCode } from '@/common/config/anchor-code'

/**
 * The page tells an anchor from any other code span, so the strings a session can load wear the
 * console's anchor colour and the rest of the code a neutral one.
 */
describe('isAnchorRun', () => {
  it('takes one anchor, several, and the /rk in front of them', () => {
    expect(isAnchorRun('task:mailer')).toBe(true)
    expect(isAnchorRun('project:vega task:report-builder')).toBe(true)
    expect(isAnchorRun('/rk project:vega task:report-builder')).toBe(true)
    expect(isAnchorRun('note:3f2a9c1e')).toBe(true)
  })

  it('leaves other code alone, colons and all', () => {
    expect(isAnchorRun('http://localhost:47355')).toBe(false)
    expect(isAnchorRun('RouteRunRepository.findLate(window)')).toBe(false)
    expect(isAnchorRun('epic:vega')).toBe(false)
    expect(isAnchorRun('project:vega and more')).toBe(false)
    expect(isAnchorRun('/rk')).toBe(false)
  })
})

describe('markAnchorCode', () => {
  type Rule = (tokens: FakeToken[], index: number) => string

  class FakeToken {
    classes: string[] = []
    constructor(readonly content: string) {}
    attrJoin(name: string, value: string): void {
      if (name === 'class') this.classes.push(value)
    }
  }

  function install(): Rule {
    const rules: Record<string, Rule> = {
      code_inline: (tokens, index) =>
        `<code class="${tokens[index]!.classes.join(' ')}">${tokens[index]!.content}</code>`
    }
    const md = { renderer: { rules } } as unknown as Parameters<typeof markAnchorCode>[0]
    markAnchorCode(md)
    return rules.code_inline!
  }

  it('classes an anchor span and hands rendering back to the original rule', () => {
    const render = install()

    expect(render([new FakeToken('task:mailer')], 0)).toBe('<code class="rk-anchor">task:mailer</code>')
    expect(render([new FakeToken('npm test')], 0)).toBe('<code class="">npm test</code>')
  })
})
