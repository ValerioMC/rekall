<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import ToolInputView from './ToolInputView.vue'
import { describePermission } from '@/features/chat/lib/describe-permission'
import type { PendingPermission } from '@/features/chat/model/timeline'

/**
 * A tool call waiting for the person's answer. It never answers, expires or dismisses itself:
 * only the resolution event removes it. Focus lands on the card, not on a key, so Enter cannot
 * answer by accident; Y and N answer while focus is inside it.
 */
const props = defineProps<{ request: PendingPermission; busy: boolean; error: string | null }>()

const emit = defineEmits<{ allow: []; deny: [] }>()

const INPUT_LINE_LIMIT = 12

const card = ref<HTMLElement | null>(null)
const sentence = computed(() => describePermission(props.request.input))

function onKeydown(event: KeyboardEvent): void {
  if (props.busy || event.metaKey || event.ctrlKey || event.altKey) return
  const key = event.key.toLowerCase()
  if (key !== 'y' && key !== 'n') return
  event.preventDefault()
  if (key === 'y') emit('allow')
  else emit('deny')
}

function focus(): void {
  card.value?.focus()
}

onMounted(focus)

defineExpose({ focus })
</script>

<template>
  <div
    ref="card"
    class="focus-ring relative mx-3 mb-2 flex flex-col gap-2 overflow-hidden rounded-card border border-warn/50 bg-warn-soft py-3 pr-3 pl-4"
    role="group"
    aria-label="Permission needed"
    tabindex="-1"
    data-testid="chat-approval"
    @keydown="onKeydown"
  >
    <span class="absolute inset-y-0 left-0 w-[3px] bg-warn" aria-hidden="true" />

    <p class="flex items-center gap-1.5 text-micro font-semibold text-warn">
      <span class="relative grid size-2 place-items-center" aria-hidden="true">
        <span class="absolute inline-flex size-2 animate-ping rounded-full bg-current/50" />
        <span class="relative inline-flex size-1.5 rounded-full bg-current" />
      </span>
      Waiting for you
    </p>
    <p class="text-body font-semibold break-words text-text" aria-live="assertive" data-testid="chat-approval-sentence">{{ sentence }}</p>

    <div class="flex flex-col gap-2">
      <ToolInputView :input="request.input" :line-limit="INPUT_LINE_LIMIT" />
    </div>

    <p v-if="error" class="text-caption text-danger" role="alert" data-testid="chat-approval-error">{{ error }}</p>

    <div class="flex flex-wrap gap-2 max-[420px]:flex-col">
      <AppButton
        variant="primary"
        :loading="busy"
        :disabled="busy"
        aria-keyshortcuts="Y"
        data-testid="chat-allow"
        @click="emit('allow')"
      >
        Allow <kbd class="text-micro opacity-70">Y</kbd>
      </AppButton>
      <AppButton
        variant="danger"
        :disabled="busy"
        aria-keyshortcuts="N"
        data-testid="chat-deny"
        @click="emit('deny')"
      >
        Deny <kbd class="text-micro opacity-70">N</kbd>
      </AppButton>
    </div>
  </div>
</template>
