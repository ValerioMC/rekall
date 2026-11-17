import { describe, expect, it } from 'vitest'
import { contextShareOf } from '@/features/project/lib/context-share'
import type { ProjectConnection } from '@/features/connection/model/connection'
import type { ProjectComponent } from '@/features/project/model/project-component'
import type { ProjectSection } from '@/features/project/model/project-section'

function component(name: string, summary: string): ProjectComponent {
  return { name, folder: 'svc', summary, description: 'x'.repeat(5000) } as ProjectComponent
}

function section(title: string, body: string): ProjectSection {
  return { title, bodyMarkdown: body } as ProjectSection
}

function binding(label: string, target: string | null): ProjectConnection {
  return { label, kind: 'GITLAB', target } as ProjectConnection
}

describe('what a session is handed from a project', () => {
  it('counts the blueprint, a summary per component, a line per binding and a title per page', () => {
    const share = contextShareOf({
      blueprint: 'b'.repeat(800),
      components: [component('billing', 's'.repeat(100))],
      connections: [binding('api', 'fintech/vega/api')],
      sections: [section('Deployment', 'long text'.repeat(1000))]
    })

    expect(share.parts.map((part) => [part.key, part.tokens])).toEqual([
      ['blueprint', 200],
      ['components', 28],
      ['connections', 7],
      ['pages', 3]
    ])
    expect(share.totalTokens).toBe(238)
  })

  it('leaves the description of a component and the text of a page out, and reports the text held back', () => {
    const share = contextShareOf({
      blueprint: null,
      components: [component('billing', 's')],
      connections: [],
      sections: [section('A', 'x'.repeat(60_000)), section('B', 'y'.repeat(2_000))]
    })

    expect(share.heldBackCharacters).toBe(62_000)
    expect(share.totalTokens).toBeLessThan(20)
  })

  it('says how many of each there are', () => {
    const share = contextShareOf({
      blueprint: '',
      components: [component('a', 's'), component('b', 's')],
      connections: [binding('api', null)],
      sections: [section('T', '')]
    })

    expect(share.parts.map((part) => part.detail)).toEqual([
      'Read in every context of the project',
      '2 summaries; a description only when a task names it',
      '1 binding, labels and targets',
      '1 title, one line each'
    ])
  })
})
