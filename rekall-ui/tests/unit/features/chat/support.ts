import { defineComponent, h } from 'vue'
import type { ToolInput } from '@/features/chat/model/chat-event'
import type { ToolItem, UserItem } from '@/features/chat/model/timeline'
import type { Terminal } from '@/features/claude/model/terminal'
import type { ProjectId, TaskId, TaskStepId, TerminalId } from '@/model/branded'

export const MarkdownStub = defineComponent({
  props: { modelValue: { type: String, required: true } },
  setup: (props) => () => h('div', { 'data-testid': 'markdown' }, props.modelValue)
})

export function toolItem(over: Partial<ToolItem> & { input: ToolInput }): ToolItem {
  return {
    kind: 'tool',
    key: 'tool-u1',
    toolUseId: 'u1',
    name: 'Tool',
    result: null,
    state: 'running',
    parentToolUseId: null,
    ...over
  }
}

export function userItem(index: number): UserItem {
  return { kind: 'user', key: `user-${index}`, text: `message ${index}` }
}

export const CHAT_ID = 'chat-1' as TerminalId

export function chatSession(over: Partial<Terminal> = {}): Terminal {
  return {
    id: CHAT_ID,
    projectId: 'p-1' as ProjectId,
    taskId: 't-1' as TaskId,
    stepId: null as TaskStepId | null,
    anchors: 'project:vega task:report-builder',
    workingDir: '/code/vega',
    projectLabel: 'vega',
    taskCode: 'report-builder',
    taskTitle: 'Report builder',
    mode: 'WORK',
    provider: 'CLAUDE',
    skipPermissions: false,
    model: null,
    effort: null,
    kind: 'CHAT',
    live: true,
    awaitingPermission: false,
    startedAt: '',
    lastActivityAt: '',
    connections: [],
    ...over
  }
}

/** Stands in for the browser's `EventSource`; clear `instances` before each test. */
export class FakeEventSource {
  static readonly CONNECTING = 0
  static readonly OPEN = 1
  static readonly CLOSED = 2
  static instances: FakeEventSource[] = []

  readyState = FakeEventSource.CONNECTING
  closed = false
  onopen: (() => void) | null = null
  onmessage: ((message: MessageEvent<string>) => void) | null = null
  onerror: (() => void) | null = null

  constructor(readonly url: string) {
    FakeEventSource.instances.push(this)
  }

  addEventListener(): void {}

  close(): void {
    this.closed = true
    this.readyState = FakeEventSource.CLOSED
  }

  open(): void {
    this.readyState = FakeEventSource.OPEN
    this.onopen?.()
  }

  emit(payload: unknown): void {
    this.onmessage?.({ data: typeof payload === 'string' ? payload : JSON.stringify(payload) } as MessageEvent<string>)
  }

  drop(): void {
    this.readyState = FakeEventSource.CONNECTING
    this.onerror?.()
  }
}
