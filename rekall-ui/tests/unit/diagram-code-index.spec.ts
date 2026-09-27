import { describe, expect, it } from 'vitest'
import { coverageOf, filesOf, spanLabel } from '@/common/diagram/code-index'
import { neighbourhoodOf } from '@/common/diagram/neighbourhood'
import { project } from '@/common/diagram/graph-projection'
import { orderGraph } from './diagram-graph.fixture'

describe('code index', () => {
  it('groups elements by the file they live in, the same file however it was spelled', () => {
    expect(filesOf(orderGraph())).toEqual([
      { file: 'src/order.rs', nodeIds: ['process', 'receive', 'validate'] },
      { file: 'src/pay.rs', nodeIds: ['charge', 'capture'] }
    ])
    expect(coverageOf(orderGraph())).toEqual({ traced: 5, total: 8 })
  })

  it('labels a span the way a reader looks it up', () => {
    expect(spanLabel({ file: 'a', startLine: 12, endLine: 30, symbol: null })).toBe('L12–30')
    expect(spanLabel({ file: 'a', startLine: 12, endLine: null, symbol: null })).toBe('L12')
    expect(spanLabel({ file: 'a', startLine: null, endLine: null, symbol: null })).toBe('whole file')
  })
})

describe('neighbourhood', () => {
  it('lights the node, its relations, their other ends and the frames around it', () => {
    const projection = project(orderGraph(), { lens: 'code', collapsed: new Set() })
    const lit = neighbourhoodOf(projection, 'validate')
    expect([...lit.nodes].sort()).toEqual(['paid', 'process', 'receive', 'validate'])
    expect(lit.edges.size).toBe(2)
  })
})
