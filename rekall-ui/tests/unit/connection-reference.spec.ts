import MarkdownIt from 'markdown-it'
import { describe, expect, it } from 'vitest'
import { markComponentReferences } from '@/common/markdown/component-reference'
import { paintConnectionReferences } from '@/common/markdown/connection-reference'
import type { ProjectConnection } from '@/features/connection/model/connection'
import type { ConnectionId, ProjectConnectionId, ProjectId } from '@/model/branded'

/** The page draws `[[connection:label]]` as a chip and settles it against the project's bindings. */
function render(markdown: string): string {
  const md = new MarkdownIt({ html: false })
  markComponentReferences(md as never)
  return md.render(markdown)
}

const PROD: ProjectConnection = {
  id: 'b-1' as ProjectConnectionId,
  connectionId: 'c-1' as ConnectionId,
  projectId: 'p-1' as ProjectId,
  kind: 'KUBERNETES',
  label: 'prod',
  title: 'Production cluster',
  baseUrl: null,
  kubeContext: null,
  target: 'payments',
  credentialSetAt: null
}

function chipOf(markdown: string, connections: readonly ProjectConnection[] | null): HTMLElement | null {
  const page = document.createElement('div')
  page.innerHTML = render(markdown)
  paintConnectionReferences(page, connections)
  return page.querySelector<HTMLElement>('.rk-connection-ref')
}

describe('the [[connection:label]] syntax', () => {
  it('becomes a connection chip carrying the label, never a component chip', () => {
    const html = render('Deploy on [[connection:prod]].')

    expect(html).toContain('<span class="rk-connection-ref" data-connection="prod" data-state="plain">prod</span>')
    expect(html).not.toContain('rk-component-ref')
  })

  it('is found with its kind, title and target on hover, ignoring case', () => {
    const chip = chipOf('[[connection:PROD]]', [PROD])

    expect(chip?.dataset['state']).toBe('found')
    expect(chip?.title).toBe('Kubernetes · Production cluster\npayments')
  })

  it('is missing when the project binds no such label, and plain with no project', () => {
    expect(chipOf('[[connection:staging]]', [PROD])?.dataset['state']).toBe('missing')
    expect(chipOf('[[connection:prod]]', null)?.dataset['state']).toBe('plain')
  })
})
