import { afterEach, describe, expect, it } from 'vitest'
import { markdown, markdownLanguage } from '@codemirror/lang-markdown'
import { EditorState } from '@codemirror/state'
import { EditorView } from '@codemirror/view'
import { livePreview } from '@/common/live-preview/live-preview'
import { referenceTargetsChanged } from '@/common/live-preview/reference-targets'
import { NO_REFERENCE_TARGETS, referenceChipFor, type ReferenceTargets } from '@/common/markdown/reference-chip'
import type { Task } from '@/features/catalog/model/catalog'
import type { ProjectConnection } from '@/features/connection/model/connection'
import type { ProjectComponent } from '@/features/project/model/project-component'
import type { ConnectionId, ProjectComponentId, ProjectConnectionId, ProjectId, TaskId } from '@/model/branded'
import { task } from './features/diagram/diagram.fixtures'

const AUTH: ProjectComponent = {
  id: 'k-1' as ProjectComponentId,
  projectId: 'p-1' as ProjectId,
  name: 'Auth',
  folder: 'services/auth',
  description: 'Issues tokens.',
  position: 0,
  createdAt: '2026-10-08T10:00:00Z',
  updatedAt: '2026-10-08T10:00:00Z'
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

const EXPORT: Task = { ...task('t-1' as TaskId, 'export', 'p-1' as ProjectId), title: 'Export the report' }

const TARGETS: ReferenceTargets = { components: [AUTH], connections: [PROD], tasks: [EXPORT] }

const views: EditorView[] = []

function editorOn(doc: string, targets: ReferenceTargets = TARGETS, caret = 0): EditorView {
  const view = new EditorView({
    parent: document.body.appendChild(document.createElement('div')),
    state: EditorState.create({
      doc,
      selection: { anchor: caret },
      extensions: [markdown({ base: markdownLanguage }), livePreview()]
    })
  })
  view.dispatch({ effects: referenceTargetsChanged.of(targets) })
  views.push(view)
  return view
}

async function focus(view: EditorView): Promise<void> {
  view.focus()
  view.contentDOM.dispatchEvent(new FocusEvent('focus'))
  await new Promise((resolve) => setTimeout(resolve, 30))
}

function chips(view: EditorView): HTMLElement[] {
  return [...view.contentDOM.querySelectorAll<HTMLElement>('.cm-rk-ref')]
}

afterEach(() => {
  views.splice(0).forEach((view) => view.destroy())
  document.body.innerHTML = ''
})

describe('referenceChipFor', () => {
  it('names a component as written, with the component glyph and its folder on hover', () => {
    const chip = referenceChipFor('Auth', TARGETS)

    expect(chip).toMatchObject({ text: 'Auth', icon: 'component', missing: false })
    expect(chip?.title).toContain('services/auth')
  })

  it('names a connection by its label, with the glyph of its kind', () => {
    expect(referenceChipFor('connection:prod', TARGETS)).toMatchObject({ text: 'prod', icon: 'KUBERNETES', missing: false })
  })

  it('names a task by its label, with its title and status on hover', () => {
    const chip = referenceChipFor('task:export', TARGETS)

    expect(chip).toMatchObject({ text: 'export', icon: 'task', missing: false })
    expect(chip?.title).toContain('Export the report')
  })

  it('marks a name the project does not have as missing', () => {
    expect(referenceChipFor('Billing', TARGETS)).toMatchObject({ text: 'Billing', missing: true })
    expect(referenceChipFor('connection:ghost', TARGETS)).toMatchObject({ icon: null, missing: true })
    expect(referenceChipFor('task:ghost', TARGETS)).toMatchObject({ icon: 'task', missing: true })
  })

  it('leaves a reference as written when the text has no project to ask', () => {
    expect(referenceChipFor('Auth', NO_REFERENCE_TARGETS)).toBeNull()
    expect(referenceChipFor('connection:prod', NO_REFERENCE_TARGETS)).toBeNull()
    expect(referenceChipFor('task:export', NO_REFERENCE_TARGETS)).toBeNull()
  })
})

describe('a [[reference]] in the live preview', () => {
  it('is drawn as a chip with its glyph and name, the brackets gone', () => {
    const view = editorOn('Ask [[Auth]] and [[connection:prod]] about [[task:export]].')

    expect(chips(view).map((chip) => chip.textContent)).toEqual(['Auth', 'prod', 'export'])
    expect(chips(view).map((chip) => chip.querySelector<HTMLElement>('.rk-completion-icon')?.dataset['icon'])).toEqual([
      'component',
      'kubernetes',
      'task'
    ])
    expect(view.contentDOM.textContent).not.toContain('[[')
  })

  it('is drawn as a broken reference when the project has no such record', () => {
    const view = editorOn('See [[Billing]].')

    expect(chips(view)[0]?.classList.contains('cm-rk-ref--missing')).toBe(true)
  })

  it('stays as written when the text has no project', () => {
    const view = editorOn('See [[Auth]].', NO_REFERENCE_TARGETS)

    expect(chips(view)).toHaveLength(0)
    expect(view.contentDOM.textContent).toContain('[[Auth]]')
  })

  it('stays as written inside code', () => {
    const view = editorOn('Write `[[Auth]]` like so.\n\n```\n[[Auth]]\n```\n')

    expect(chips(view)).toHaveLength(0)
  })

  it('is drawn once the project’s records arrive', () => {
    const view = editorOn('See [[Auth]].', NO_REFERENCE_TARGETS)

    view.dispatch({ effects: referenceTargetsChanged.of(TARGETS) })

    expect(chips(view).map((chip) => chip.textContent)).toEqual(['Auth'])
  })

  it('never changes the text it draws', () => {
    const doc = 'See [[Auth]] and [[task:export]].'
    const view = editorOn(doc)

    expect(view.state.doc.toString()).toBe(doc)
  })

  it('shows its source when a click puts the caret in it, and the chip again when the caret leaves', async () => {
    const doc = 'See [[Auth]] now'
    const view = editorOn(doc)
    await focus(view)

    chips(view)[0]?.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true }))

    expect(view.state.selection.main.head).toBe(doc.indexOf(']]'))
    expect(chips(view)).toHaveLength(0)
    expect(view.contentDOM.textContent).toContain('[[Auth]]')

    view.dispatch({ selection: { anchor: doc.length } })

    expect(chips(view).map((chip) => chip.textContent)).toEqual(['Auth'])
  })

  it('is drawn the moment it is completed, with the caret just after it', async () => {
    const doc = 'See [[Auth]]'
    const view = editorOn(doc, TARGETS, doc.length)
    await focus(view)

    expect(chips(view).map((chip) => chip.textContent)).toEqual(['Auth'])
  })
})
