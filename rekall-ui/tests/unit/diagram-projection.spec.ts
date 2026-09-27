import { describe, expect, it } from 'vitest'
import { project } from '@/common/diagram/graph-projection'
import { orderGraph } from './diagram-graph.fixture'

const ids = (values: readonly { id: string }[]): string[] => values.map((value) => value.id)

describe('graph projection', () => {
  it('the concept lens hides code and lets its pieces stand on their own', () => {
    const projection = project(orderGraph(), { lens: 'concept', collapsed: new Set() })

    expect(ids(projection.nodes)).toEqual(['receive', 'validate', 'paid', 'capture', 'orders', 'done'])
    expect(projection.nodes.every((node) => node.parentId === null && !node.isCluster)).toBe(true)
    expect(projection.edges.some((edge) => edge.relation === 'calls')).toBe(false)
  })

  it('the code lens frames each function around the pieces it implements', () => {
    const projection = project(orderGraph(), { lens: 'code', collapsed: new Set() })
    const process = projection.nodes.find((node) => node.id === 'process')!

    expect(process.isCluster).toBe(true)
    expect(projection.nodes.find((node) => node.id === 'validate')!.parentId).toBe('process')
    expect(projection.edges.some((edge) => edge.relation === 'contains')).toBe(false)
    expect(projection.edges.find((edge) => edge.relation === 'calls')).toMatchObject({ from: 'process', to: 'charge' })
  })

  it('a collapsed frame absorbs its pieces and takes over their relations', () => {
    const projection = project(orderGraph(), { lens: 'code', collapsed: new Set(['process']) })
    const process = projection.nodes.find((node) => node.id === 'process')!

    expect(ids(projection.nodes)).not.toContain('validate')
    expect(process).toMatchObject({ collapsed: true, isCluster: false, hiddenCount: 3, foldable: true })
    expect(projection.edges.find((edge) => edge.to === 'capture')).toMatchObject({ from: 'process', label: 'card' })
    expect(projection.edges.some((edge) => edge.from === 'process' && edge.to === 'process')).toBe(false)
  })

  it('folding merges relations that end up joining the same two nodes', () => {
    const graph = orderGraph()
    const doubled = { ...graph, edges: [...graph.edges, { ...graph.edges[5]!, id: 'x', from: 'receive', to: 'capture', relation: 'leads_to', label: null }] }
    const projection = project(doubled, { lens: 'code', collapsed: new Set(['process', 'charge']) })

    expect(projection.edges.filter((edge) => edge.relation === 'leads_to')).toHaveLength(1)
    expect(projection.edges.find((edge) => edge.relation === 'leads_to')!.edgeIds).toEqual(['x'])
  })
})
