import { afterEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick, ref, shallowRef } from 'vue'
import ExplainCodeDialog from '@/features/diagram/components/dialogs/ExplainCodeDialog.vue'
import ViewModeSwitch from '@/features/diagram/components/viewer/ViewModeSwitch.vue'
import WalkthroughBar from '@/features/diagram/components/walkthrough/WalkthroughBar.vue'
import WalkthroughCard from '@/features/diagram/components/walkthrough/WalkthroughCard.vue'
import { BEAT_MILLISECONDS, useWalkthrough } from '@/features/diagram/composables/useWalkthrough'
import { callTreeOf } from '@/features/diagram/lib/call-tree'
import { layoutSequence } from '@/features/diagram/lib/layout/sequence-layout'
import { walkthroughBeats } from '@/features/diagram/lib/walkthrough/walkthrough-beats'
import { ORION, REPORT, VEGA, project, task } from './diagram.fixtures'
import type { SheetDrawing } from '@/features/diagram/lib/sheet-drawing'
import type { SequenceScene } from '@/features/diagram/lib/scene/sequence-scene'
import type { DiagramSheet, Walkthrough } from '@/features/diagram/model/diagram'
import type { GenerationRequest } from '@/features/diagram/store/diagram.store'
import type { CompanyId, DiagramId } from '@/model/branded'

const SCENE: SequenceScene = {
  participants: [
    { id: 'C', label: 'OrderController', isActor: false },
    { id: 'S', label: 'OrderService', isActor: false },
    { id: 'DB', label: 'Postgres', isActor: false }
  ],
  boxes: [],
  autonumber: false,
  steps: [
    { type: 'message', number: 1, from: 'C', to: 'S', text: 'place(order)', line: 'solid', head: 'arrow' },
    { type: 'message', number: 2, from: 'S', to: 'DB', text: 'insert(order)', line: 'solid', head: 'arrow' },
    { type: 'message', number: 3, from: 'DB', to: 'S', text: 'id', line: 'dashed', head: 'arrow' },
    { type: 'message', number: 4, from: 'S', to: 'C', text: 'Order', line: 'dashed', head: 'arrow' }
  ]
}

const WORDS: Walkthrough = {
  external: ['DB'],
  messages: {
    '#1': { explain: 'The service owns the transaction.', input: '{ sku: "A-7" }', output: 'Order', opens: 1 },
    '#2': { explain: null, input: 'Order', output: 'id', opens: null }
  }
}

const BEATS = walkthroughBeats(SCENE, callTreeOf(SCENE), WORDS)
const LABELS = new Map(SCENE.participants.map((participant) => [participant.id, participant.label]))

function sheet(walkthrough: Walkthrough | null, title: string | null = null): DiagramSheet {
  return { kind: 'sequence', title, mermaid: 'sequenceDiagram', links: {}, walkthrough }
}

function drawn(scene: SequenceScene): SheetDrawing {
  const layout = layoutSequence(scene)
  return { drawing: 'sequence', scene, layout, calls: callTreeOf(scene), bounds: layout.bounds }
}

function card(position: number) {
  return mount(WalkthroughCard, {
    props: { diagramId: 'd1' as DiagramId, beat: BEATS[position]!, position, count: BEATS.length, labels: LABELS, external: new Set(['DB']), sources: [], opensTitle: 'Inside place' }
  })
}

describe('the walkthrough card', () => {
  it('says what the call does, what goes in and comes out, and offers to step into it', async () => {
    const wrapper = card(0)
    expect(wrapper.text()).toContain('Step 1 of 4')
    expect(wrapper.text()).toContain('place(order)')
    expect(wrapper.find('[data-testid="walkthrough-explain"]').text()).toBe('The service owns the transaction.')
    expect(wrapper.find('[data-testid="walkthrough-values"]').text()).toContain('{ sku: "A-7" }')
    expect(wrapper.text()).toContain('Inside place')
    await wrapper.find('[data-testid="walkthrough-step-into"]').trigger('click')
    expect(wrapper.emitted('stepInto')).toHaveLength(1)
  })

  it('marks a call that leaves the code, and shows a reply only what it returns', () => {
    const external = card(1)
    expect(external.text()).toContain('external')
    expect(external.text()).toContain('leaves the project')
    const reply = card(2)
    expect(reply.text()).toContain('Returns')
    expect(reply.find('[data-testid="walkthrough-values"]').text()).toBe('Returnedid')
    expect(reply.find('[data-testid="walkthrough-step-into"]').exists()).toBe(false)
  })
})

describe('the walkthrough bar', () => {
  function bar(position: number, trail: string[] = ['Place an order']) {
    return mount(WalkthroughBar, { props: { beats: BEATS, position, playing: false, pace: 'normal' as const, trail } })
  }

  it('counts the steps, ticks each one and sends the reader where they click', async () => {
    const wrapper = bar(1)
    expect(wrapper.find('[data-testid="walkthrough-counter"]').text()).toBe('2 / 4')
    const ticks = wrapper.findAll('ol button')
    expect(ticks).toHaveLength(4)
    expect(ticks[1]!.attributes('aria-current')).toBe('step')
    await ticks[3]!.trigger('click')
    await wrapper.find('[data-testid="walkthrough-next"]').trigger('click')
    await wrapper.find('[data-testid="walkthrough-toggle"]').trigger('click')
    expect(wrapper.emitted('goTo')).toEqual([[3]])
    expect(wrapper.emitted('next')).toHaveLength(1)
    expect(wrapper.emitted('toggle')).toHaveLength(1)
  })

  it('shows the calls stepped into and backs out to one of them', async () => {
    const wrapper = bar(0, ['Place an order', 'place(order)'])
    const trail = wrapper.findAll('[data-testid="walkthrough-trail"]')
    expect(trail.map((stop) => stop.text())).toEqual(['Place an order'])
    await trail[0]!.trigger('click')
    expect(wrapper.emitted('backTo')).toEqual([[0]])
  })
})

describe('the view switch', () => {
  it('offers the walkthrough only on a sequence', async () => {
    const wrapper = mount(ViewModeSwitch, { props: { mode: 'diagram' as const, playable: false } })
    expect(wrapper.find('[data-testid="view-mode-walkthrough"]').attributes('disabled')).toBeDefined()
    await wrapper.setProps({ playable: true })
    await wrapper.find('[data-testid="view-mode-walkthrough"]').trigger('click')
    expect(wrapper.emitted('change')).toEqual([['walkthrough']])
  })
})

describe('useWalkthrough', () => {
  afterEach(() => vi.useRealTimers())

  const INNER: SequenceScene = { ...SCENE, steps: [{ type: 'message', number: 1, from: 'S', to: 'S', text: 'validate', line: 'solid', head: 'arrow' }] }

  function player() {
    const sheets = ref<readonly DiagramSheet[]>([sheet(WORDS, 'Place an order'), sheet(null, 'Inside place')])
    const sheetIndex = ref(0)
    const drawing = shallowRef<SheetDrawing | null>(drawn(SCENE))
    return { sheetIndex, drawing, walk: useWalkthrough(sheets, sheetIndex, drawing) }
  }

  it('steps into a call by opening its sheet, and back out onto the caller', async () => {
    const { sheetIndex, drawing, walk } = player()
    walk.stepInto()
    expect(sheetIndex.value).toBe(1)
    drawing.value = drawn(INNER)
    await nextTick()
    expect(walk.beat.value?.message.text).toBe('validate')

    walk.next()
    expect(sheetIndex.value).toBe(0)
    expect(walk.position.value).toBe(1)
  })

  it('plays one beat per pace and stops at the end', async () => {
    vi.useFakeTimers()
    const { walk } = player()
    walk.goTo(2)
    walk.toggle()
    await nextTick()
    vi.advanceTimersByTime(BEAT_MILLISECONDS.normal)
    await nextTick()
    expect(walk.position.value).toBe(3)
    vi.advanceTimersByTime(BEAT_MILLISECONDS.normal)
    await nextTick()
    expect(walk.playing.value).toBe(false)
    walk.toggle()
    expect(walk.position.value).toBe(0)
  })

  it('starts again on a sheet picked by hand', async () => {
    const { sheetIndex, walk } = player()
    walk.goTo(2)
    sheetIndex.value = 1
    await nextTick()
    expect(walk.stack.value).toEqual([{ sheetIndex: 1, position: 0, via: null }])
  })
})

describe('the explain dialog', () => {
  it('picks a company, then one of its projects, and asks for a walkthrough of what was described', async () => {
    const other = { ...project(ORION, 'orion'), companyId: 'c-beta' as CompanyId, companyName: 'beta' }
    const wrapper = mount(ExplainCodeDialog, {
      props: { projects: [project(VEGA, 'vega'), other], tasks: [task(REPORT, 'report-builder', VEGA)], initialProjectId: VEGA, initialTaskId: null, sending: false },
      attachTo: document.body
    })
    expect(document.querySelector('[data-testid="explain-submit"]')?.hasAttribute('disabled')).toBe(true)

    await wrapper.find('[data-testid="explain-request"]').setValue('How an order is placed')

    expect(document.querySelector('[data-testid="explain-preview"]')?.textContent).toContain('/rk project:vega explain "How an order is placed"')
    await wrapper.find('[data-testid="explain-submit"]').trigger('click')
    const [[asked]] = wrapper.emitted('explain') as [[GenerationRequest]]
    expect(asked).toEqual({ projectId: VEGA, taskId: null, request: 'How an order is placed', kinds: ['sequence'], walkthrough: true })
    wrapper.unmount()
  })
})
