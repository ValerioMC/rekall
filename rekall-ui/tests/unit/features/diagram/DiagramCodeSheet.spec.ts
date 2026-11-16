import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import CodeModal from '@/features/diagram/components/walkthrough/CodeModal.vue'
import { ApiError } from '@/api/client'
import { callTreeOf } from '@/features/diagram/lib/call-tree'
import { walkthroughBeats } from '@/features/diagram/lib/walkthrough/walkthrough-beats'
import type { SequenceScene } from '@/features/diagram/lib/scene/sequence-scene'
import type { CodeLinks, SourceExcerpt } from '@/features/diagram/model/diagram'
import type { DiagramId } from '@/model/branded'

const fetchSourceFile = vi.fn()
vi.mock('@/features/diagram/api/diagrams.api', () => ({ fetchSourceFile: (...args: unknown[]) => fetchSourceFile(...args) }))

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
    { type: 'message', number: 3, from: 'DB', to: 'S', text: 'id', line: 'dashed', head: 'arrow' }
  ]
}

const BEATS = walkthroughBeats(SCENE, callTreeOf(SCENE), { external: ['DB'], messages: { '#1': { explain: 'The service owns the transaction.', input: null, output: null, opens: null } } })
const LABELS = new Map(SCENE.participants.map((participant) => [participant.id, participant.label]))

const LINKS: CodeLinks = {
  '#1': [{ file: 'src/service.rs', startLine: 3, endLine: 5, symbol: 'OrderService::place' }],
  '#2': [{ file: 'src/db.rs', startLine: 2, endLine: 2, symbol: 'insert' }]
}

function file(name: string, lines: readonly string[]): SourceExcerpt {
  return { file: name, language: 'rust', highlightStart: null, highlightEnd: null, totalLines: lines.length, lines: lines.map((text, index) => ({ number: index + 1, text })), truncated: false }
}

const FILES: Record<string, SourceExcerpt> = {
  'src/service.rs': file('src/service.rs', ['use db;', '', 'fn place(order: Order) {', '    insert(order);', '}']),
  'src/db.rs': file('src/db.rs', ['// the table', 'fn insert(order: Order) {}'])
}

function sheet(position: number, links: CodeLinks = LINKS) {
  return mount(CodeModal, {
    props: { diagramId: 'd1' as DiagramId, beats: BEATS, position, links, labels: LABELS, external: new Set(['DB']) },
    attachTo: document.body
  })
}

describe('the code sheet', () => {
  beforeEach(() => {
    fetchSourceFile.mockReset()
    fetchSourceFile.mockImplementation(async (_id: DiagramId, name: string) => FILES[name])
  })

  it('opens the file of the step, whole, with its lines marked and one tab per file of the walk', async () => {
    const wrapper = sheet(0)
    await flushPromises()

    expect(fetchSourceFile).toHaveBeenCalledWith('d1', 'src/service.rs')
    const tabs = wrapper.findAll('[data-testid="code-tab"]')
    expect(tabs.map((tab) => tab.text())).toEqual(['service.rs 1', 'db.rs 1'])
    expect(tabs[0]!.attributes('aria-selected')).toBe('true')
    const marked = wrapper.findAll('.code-line.is-marked').map((line) => line.attributes('data-line'))
    expect(marked).toEqual(['3', '4', '5'])
    expect(wrapper.findAll('.code-line')).toHaveLength(5)
    expect(wrapper.find('[data-testid="code-crumbs"]').text()).toContain('OrderService::place')
    expect(wrapper.find('[data-testid="code-crumbs"]').text()).toContain('L3–5')
    expect(wrapper.text()).toContain('The service owns the transaction.')
    wrapper.unmount()
  })

  it('walks to the step a clicked method belongs to', async () => {
    const wrapper = sheet(0)
    await flushPromises()

    const links = wrapper.findAll('[data-testid="code-link"]')
    expect(links.map((link) => link.text())).toEqual(['place', 'insert'])
    await links[1]!.trigger('click')

    expect(wrapper.emitted('walk')).toEqual([[1]])
    wrapper.unmount()
  })

  it('walks to the first step of a file whose tab is picked', async () => {
    const wrapper = sheet(0)
    await flushPromises()

    await wrapper.findAll('[data-testid="code-tab"]')[1]!.trigger('click')

    expect(wrapper.emitted('walk')).toEqual([[1]])
    wrapper.unmount()
  })

  it('steps back and forth from its own header and closes', async () => {
    const wrapper = sheet(1)
    await flushPromises()

    await wrapper.find('[data-testid="code-previous"]').trigger('click')
    await wrapper.find('[data-testid="code-next"]').trigger('click')
    await wrapper.find('[data-testid="code-close"]').trigger('click')

    expect(wrapper.emitted('walk')).toEqual([[0], [2]])
    expect(wrapper.emitted('close')).toHaveLength(1)
    wrapper.unmount()
  })

  it('says so when the step is not linked to the code', async () => {
    const wrapper = sheet(2)
    await flushPromises()

    expect(wrapper.find('[data-testid="code-unlinked"]').exists()).toBe(true)
    expect(fetchSourceFile).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('shows why the code could not be read', async () => {
    fetchSourceFile.mockRejectedValueOnce(new ApiError(400, 'This diagram\'s project has no folder.', 'c1'))
    const wrapper = sheet(0)
    await flushPromises()

    expect(wrapper.find('[data-testid="code-failure"]').text()).toContain('has no folder')
    wrapper.unmount()
  })
})
