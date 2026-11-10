import { describe, expect, it } from 'vitest'
import { drawnKindOf } from '@/features/diagram/lib/mermaid/source-kind'

describe('drawnKindOf', () => {
  it.each([
    ['sequenceDiagram\n  A->>B: hi', 'sequence'],
    ['flowchart TD\n  A --> B', 'flowchart'],
    ['graph LR\n  A --> B', 'flowchart'],
    ['stateDiagram-v2\n  [*] --> Idle', 'state'],
    ['classDiagram\n  class Order', 'class'],
    ['erDiagram\n  ORDER ||--o{ LINE : has', 'er']
  ])('reads %j as %s', (source, kind) => {
    expect(drawnKindOf(source)).toBe(kind)
  })

  it('looks past front matter, directives, comments and blank lines', () => {
    const source = '---\ntitle: Checkout\n---\n\n%%{init: {"theme": "dark"}}%%\n%% a comment\nsequenceDiagram\n  A->>B: hi'

    expect(drawnKindOf(source)).toBe('sequence')
  })

  it('leaves the types Mermaid draws and what is not a declaration to Mermaid', () => {
    expect(drawnKindOf('mindmap\n  root')).toBeNull()
    expect(drawnKindOf('gantt\n  title Plan')).toBeNull()
    expect(drawnKindOf('constructor')).toBeNull()
    expect(drawnKindOf('')).toBeNull()
  })
})
