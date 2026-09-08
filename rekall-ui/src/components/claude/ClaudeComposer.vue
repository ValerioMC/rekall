<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'

const props = defineProps<{
  disabled: boolean
  hint?: string
}>()

const emit = defineEmits<{ send: [text: string] }>()

const draft = ref('')
const field = ref<HTMLTextAreaElement | null>(null)

const canSend = computed(() => !props.disabled && draft.value.trim().length > 0)

function grow(): void {
  const element = field.value
  if (!element) return
  element.style.height = 'auto'
  element.style.height = `${Math.min(element.scrollHeight, 180)}px`
}

watch(draft, () => void nextTick(grow))

function submit(): void {
  if (!canSend.value) return
  emit('send', draft.value.trim())
  draft.value = ''
  void nextTick(grow)
}

function onKeydown(event: KeyboardEvent): void {
  if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
    event.preventDefault()
    submit()
  }
}

defineExpose({
  focus: () => field.value?.focus()
})
</script>

<template>
  <form
    class="dock-lane-safe flex shrink-0 items-end gap-2 border-t border-border bg-surface px-5 py-3"
    @submit.prevent="submit"
  >
    <div class="min-w-0 flex-1">
      <textarea
        ref="field"
        v-model="draft"
        :placeholder="disabled ? (hint ?? 'This session is not taking input.') : 'Write a prompt. ⌘↵ to send.'"
        :disabled="disabled"
        rows="1"
        spellcheck="false"
        class="focus-ring block max-h-[180px] w-full resize-none rounded-[var(--radius-control)] border border-border bg-canvas px-3 py-2 text-[13px] leading-relaxed text-text transition-colors placeholder:text-text-subtle hover:border-border-strong focus-visible:border-accent-deep disabled:cursor-not-allowed disabled:opacity-50"
        data-testid="claude-composer-input"
        @keydown="onKeydown"
      />
    </div>
    <button
      type="submit"
      class="focus-ring inline-flex h-9 shrink-0 items-center gap-1.5 rounded-[var(--radius-control)] border border-accent bg-accent-soft px-3.5 text-[12.5px] font-medium text-accent transition-colors hover:bg-accent hover:text-accent-ink disabled:cursor-not-allowed disabled:opacity-40"
      :disabled="!canSend"
      data-testid="claude-composer-send"
    >
      <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
        <path
          d="M1.5 7h9M6.5 3l4 4-4 4"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
      Send
    </button>
  </form>
</template>
