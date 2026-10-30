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
    taskLabel: 'report-builder',
    taskTitle: 'Report builder',
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
