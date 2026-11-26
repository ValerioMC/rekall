<script setup lang="ts">
import { ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import SessionProviderMenu from '@/features/claude/components/SessionProviderMenu.vue'
import { usePlanTask } from '@/features/claude/composables/usePlanTask'
import { useAnchoredMenu } from '@/composables/useAnchoredMenu'
import type { SessionProvider } from '@/features/claude/model/session-provider'
import type { TaskId } from '@/model/branded'

const props = defineProps<{ taskId: TaskId; folder: string | null }>()

const refining = usePlanTask(
  () => props.taskId,
  () => props.folder,
  "Set this project's folder on its page to refine from it"
)
const button = ref<InstanceType<typeof AppButton> | null>(null)
const { open, menu, placement, toggle, close, handleKeydown } = useAnchoredMenu(() => button.value?.$el ?? null, 240)

function refine(provider: SessionProvider): void {
  if (refining.isRunning.value) return
  close(true)
  refining.start('REFINE', provider)
}
</script>

<template>
  <AppButton
    ref="button"
    variant="primary"
    size="sm"
    title="Have a session go through this task with you and record each decision here"
    data-testid="decision-refine"
    aria-haspopup="menu"
    :aria-expanded="open"
    :aria-disabled="refining.isRunning.value"
    :aria-busy="refining.isRunning.value"
    @click="refining.isRunning.value ? undefined : toggle()"
  >
    Refine
    <svg class="size-3.5 shrink-0" viewBox="0 0 16 16" fill="none" aria-hidden="true">
      <path d="m4 6 4 4 4-4" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
  </AppButton>

  <Teleport to="body">
    <div
      v-if="open"
      ref="menu"
      role="menu"
      aria-label="Refine"
      class="fixed z-(--z-popover) overflow-y-auto rounded-card border border-border-strong bg-surface-raised p-1 shadow-lift"
      :style="placement"
      data-anchored-menu
      @keydown="handleKeydown"
    >
      <SessionProviderMenu :task-id="taskId" :busy="refining.isRunning.value" @choose="refine" />
    </div>
  </Teleport>
</template>
