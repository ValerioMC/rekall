import { defineComponent, h } from 'vue'
import type { ToolInput } from '@/features/chat/model/chat-event'
import type { ToolItem, UserItem } from '@/features/chat/model/timeline'

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
