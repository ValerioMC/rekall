import MarkdownIt from 'markdown-it'
import { describe, expect, it } from 'vitest'
import { componentNameKey, markComponentReferences, paintComponentReferences } from '@/common/markdown/component-reference'
import type { ProjectComponent } from '@/features/project/model/project-component'
import type { ProjectComponentId, ProjectId } from '@/model/branded'

/** The page draws `[[Name]]` as a chip and settles it against the project's components. */
function render(markdown: string): string {
  const md = new MarkdownIt({ html: false })
  markComponentReferences(md as never)
  return md.render(markdown)
}

function component(name: string, folder = 'services/auth', description = 'Issues tokens.'): ProjectComponent {
  return {
    id: name as ProjectComponentId,
    projectId: 'p-1' as ProjectId,
    name,
    folder,
    description,
    position: 0,
    createdAt: '2026-10-08T10:00:00Z',
    updatedAt: '2026-10-08T10:00:00Z'
  }
}

function chipsOf(markdown: string, components: readonly ProjectComponent[] | null): HTMLElement[] {
  const page = document.createElement('div')
  page.innerHTML = render(markdown)
  paintComponentReferences(page, components)
  return [...page.querySelectorAll<HTMLElement>('.rk-component-ref')]
}

describe('the [[Name]] syntax', () => {
  it('becomes a chip carrying the written name', () => {
    expect(render('Use [[Auth Service]] here.')).toContain(
      '<span class="rk-component-ref" data-component="Auth Service" data-state="plain">Auth Service</span>'
    )
  })

  it('is left alone in a code span and in a fenced block', () => {
    expect(render('Write `[[Auth Service]]` here.')).not.toContain('rk-component-ref')
    expect(render('```\n[[Auth Service]]\n```')).not.toContain('rk-component-ref')
  })

  it('is left alone with no closing, a blank name, or a name spanning lines', () => {
    expect(render('open [[Auth Service and more')).not.toContain('rk-component-ref')
    expect(render('[[   ]] and [[]]')).not.toContain('rk-component-ref')
    expect(render('across [[Auth\nService]] lines')).not.toContain('rk-component-ref')
  })

  it('keeps a leading [ as text and still reads the reference after it, as the backend does', () => {
    expect(render('[[[Auth Service]]')).toContain('<p>[<span class="rk-component-ref"')
  })

  it('escapes a name so it cannot inject markup', () => {
    expect(render('[[<b>x</b>]]')).toContain('data-component="&lt;b&gt;x&lt;/b&gt;"')
  })
})

describe('componentNameKey', () => {
  it('ignores case and surrounding spaces', () => {
    expect(componentNameKey('  Auth SERVICE ')).toBe('auth service')
  })
})

describe('paintComponentReferences', () => {
  it('marks a known component found and gives its folder and description on hover', () => {
    const [chip] = chipsOf('[[auth service]]', [component('Auth Service')])

    expect(chip?.dataset['state']).toBe('found')
    expect(chip?.title).toBe('services/auth\n\nIssues tokens.')
  })

  it('marks a name that matches nothing as a broken reference', () => {
    const [chip] = chipsOf('[[Billing]]', [component('Auth Service')])

    expect(chip?.dataset['state']).toBe('missing')
    expect(chip?.title).toContain('no component named “Billing”')
  })

  it('treats every reference in a project with no components as broken', () => {
    expect(chipsOf('[[Auth Service]]', [])[0]?.dataset['state']).toBe('missing')
  })

  it('leaves the text plain when it belongs to no project', () => {
    const [chip] = chipsOf('[[Auth Service]]', null)

    expect(chip?.dataset['state']).toBe('plain')
    expect(chip?.hasAttribute('title')).toBe(false)
  })

  it('follows a rename without a new render', () => {
    const page = document.createElement('div')
    page.innerHTML = render('[[Auth Service]]')
    paintComponentReferences(page, [component('Auth Service')])
    paintComponentReferences(page, [component('Identity')])

    expect(page.querySelector<HTMLElement>('.rk-component-ref')?.dataset['state']).toBe('missing')
  })

  it('cuts a long description in the hover text', () => {
    const [chip] = chipsOf('[[Auth Service]]', [component('Auth Service', 'a', 'x'.repeat(400))])

    expect(chip?.title.endsWith('…')).toBe(true)
    expect(chip?.title.length).toBeLessThan(300)
  })
})
