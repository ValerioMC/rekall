<script setup lang="ts">
import { computed } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppSplitButton, { type SplitButtonOption } from '@/components/ui/AppSplitButton.vue'
import StepSeal from '@/features/steps/components/StepSeal.vue'
import { useDescriptionRewrite } from '@/features/catalog/composables/useDescriptionRewrite'
import type { RewriteStyle } from '@/features/claude/model/terminal'
import type { TaskId } from '@/model/branded'

/**
 * Asks a session to rewrite the description: Fix wording on the key, Restructure in its menu. While
 * it runs the key becomes the lit seal Run here and Run revision show, and once the new text is in
 * an Undo puts the old one back. Rekall keeps no history, so that text is held in this window only.
 */
const props = defineProps<{ taskId: TaskId; anchor: string; folder: string | null }>()

const RESTRUCTURE_OPTIONS: readonly SplitButtonOption<RewriteStyle>[] = [
  {
    id: 'RESTRUCTURE',
    label: 'Restructure',
    hint: "Lay the same content out under Goal, Problem, Solution and so on, or under the project's Task template page"
  }
]

const { phase, blocker, undoable, start, undo } = useDescriptionRewrite(
  () => props.taskId,
  () => props.anchor,
  () => props.folder
)

const hint = computed(() => blocker.value ?? 'A session corrects the grammar, spelling and wording and keeps everything else')
const busy = computed(() => phase.value !== 'idle')
const caption = computed(() => (phase.value === 'launching' ? 'Starting' : phase.value === 'working' ? 'Rewriting' : 'Rewritten'))
</script>

<template>
  <div class="flex items-center gap-1.5" data-testid="description-rewrite">
    <AppButton
      v-if="undoable && !busy"
      size="sm"
      variant="quiet"
      title="Puts the earlier text back. Rekall keeps no history: it is held in this window only, and gone once you edit, leave the task or close the console"
      data-testid="description-rewrite-undo"
      @click="undo"
    >
      Undo
    </AppButton>

    <span
      v-if="busy"
      class="key-lit inline-flex h-7 shrink-0 cursor-default items-center gap-1.5 rounded-key px-2.5 text-small font-medium"
      role="status"
      :data-phase="phase"
      data-testid="description-rewrite-busy"
    >
      <span class="grid size-4 place-items-center" aria-hidden="true">
        <svg v-if="phase === 'launching'" class="size-4" viewBox="0 0 16 16" fill="none">
          <circle class="rewrite-ring" cx="8" cy="8" r="6.4" pathLength="100" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
        </svg>
        <StepSeal v-else :state="phase === 'working' ? 'RUNNING' : 'DONE'" :arriving="phase === 'done'" />
      </span>
      {{ caption }}
    </span>

    <AppSplitButton
      v-else
      size="sm"
      variant="secondary"
      :options="RESTRUCTURE_OPTIONS"
      menu-label="Other ways to rewrite"
      :disabled="blocker !== null"
      :title="hint"
      data-testid="description-rewrite-fix"
      @click="start('FIX')"
      @choose="start"
    >
      Fix wording
    </AppSplitButton>
  </div>
</template>

<style scoped>
.rewrite-ring {
  stroke-dasharray: 28 72;
  opacity: 0.8;
  transform-box: fill-box;
  transform-origin: center;
  animation: dial-sweep 800ms linear infinite;
}
</style>
