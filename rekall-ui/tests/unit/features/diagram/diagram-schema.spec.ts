import { describe, expect, it } from 'vitest'
import { DiagramSchema } from '@/features/diagram/api/schemas/diagram.schema'
import { SCENARIO, VEGA, summary } from './diagram.fixtures'

const SHEET = { kind: 'sequence', mermaid: 'sequenceDiagram\n  A->>B: hi' }

describe('the diagram schema', () => {
  it('reads the scenario a diagram follows', () => {
    const diagram = DiagramSchema.parse({ ...summary('6f9c1f0e-9a51-4f0a-8f5e-3c1b2a4d5e6f', VEGA), projectId: '0b1c2d3e-4f50-4617-8899-aabbccddeeff', scenario: SCENARIO, sheets: [SHEET] })
    expect(diagram.scenario).toEqual(SCENARIO)
  })

  it('reads a diagram without one, imported or stored earlier, as having none', () => {
    const base = { ...summary('6f9c1f0e-9a51-4f0a-8f5e-3c1b2a4d5e6f', VEGA), projectId: '0b1c2d3e-4f50-4617-8899-aabbccddeeff', sheets: [SHEET] }
    expect(DiagramSchema.parse(base).scenario).toBeNull()
    expect(DiagramSchema.parse({ ...base, scenario: null }).scenario).toBeNull()
  })
})
