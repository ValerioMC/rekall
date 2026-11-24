<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import PopoverMenu, { type PopoverMenuEntry } from '@/components/ui/PopoverMenu.vue'
import type { ProjectSection } from '@/features/project/model/project-section'
import type { SectionContext } from '@/features/project/model/section-context'
import type { SectionContextId } from '@/model/branded'

const NEW_CONTEXT = 'new-context'

/** The context a page is kept in, in its header: a key that opens the contexts to move the page to, or to start a new one. */
const props = defineProps<{ contexts: readonly SectionContext[]; sections: readonly ProjectSection[]; currentId: SectionContextId }>()

const emit = defineEmits<{ move: [id: SectionContextId]; createContext: [title: string] }>()

const open = ref(false)
const naming = ref(false)
const draft = ref('')
const input = ref<HTMLInputElement | null>(null)

const current = computed(() => props.contexts.find((context) => context.id === props.currentId))
const entries = computed<readonly PopoverMenuEntry[]>(() => [
  { kind: 'heading', label: 'Move to context' },
  ...props.contexts.map((context) => ({
    kind: 'item' as const,
    id: context.id,
    label: context.title,
    current: context.id === props.currentId,
    hint: pagesLabel(context.id)
  })),
  { kind: 'separator' },
  { kind: 'item', id: NEW_CONTEXT, label: 'New context…' }
])

function pagesLabel(contextId: SectionContextId): string {
  const count = props.sections.filter((section) => section.contextId === contextId).length
  return `${count} ${count === 1 ? 'page' : 'pages'}`
}

async function choose(id: string): Promise<void> {
  if (id !== NEW_CONTEXT) {
    if (id !== props.currentId) emit('move', id as SectionContextId)
    return
  }
  naming.value = true
  await nextTick()
  input.value?.focus()
}

function submit(): void {
  const wanted = draft.value.trim()
  naming.value = false
  draft.value = ''
  if (wanted !== '') emit('createContext', wanted)
}

function cancel(): void {
  naming.value = false
  draft.value = ''
}
</script>

<template>
  <form v-if="naming" class="w-44" data-testid="section-context-new-form" @submit.prevent="submit">
    <label class="sr-only" for="section-context-new">New context</label>
    <input
      id="section-context-new"
      ref="input"
      v-model="draft"
      maxlength="200"
      autocomplete="off"
      placeholder="New context, then Enter"
      class="field text-text h-7 w-full rounded-control px-2.5 text-small"
      data-testid="section-context-new-title"
      @keydown.esc.stop.prevent="cancel"
      @blur="cancel"
    />
  </form>
  <PopoverMenu v-else v-model:open="open" :entries="entries" label="Move to context" align="right" @pick="choose">
    <template #trigger="{ toggle }">
      <AppButton
        variant="secondary"
        size="sm"
        title="Move this page to another context"
        aria-haspopup="menu"
        :aria-expanded="open"
        data-popover-trigger
        data-testid="section-context"
        @click="toggle"
      >
        <svg class="size-3.5 text-text-subtle" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path d="M2 4.6A1.1 1.1 0 0 1 3.1 3.5h2.7l1.4 1.6h5.7A1.1 1.1 0 0 1 14 6.2v5.9a1.1 1.1 0 0 1-1.1 1.1H3.1A1.1 1.1 0 0 1 2 12.1Z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
        </svg>
        <span class="font-normal text-text-subtle">Context</span>
        <span>{{ current?.title ?? '' }}</span>
        <svg class="size-3 text-text-subtle" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path d="m4 6.5 4 4 4-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </AppButton>
    </template>
  </PopoverMenu>
</template>
