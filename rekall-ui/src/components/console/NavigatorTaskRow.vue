<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import TagBadge from '@/components/ui/TagBadge.vue'
import TaskMark from '@/components/console/TaskMark.vue'
import { identityHue } from '@/common/identity'
import { TASK_STATUS_LABEL } from '@/model/catalog'
import type { Task } from '@/model/catalog'
import { TASK_MARK_LABEL, taskMark } from '@/model/task-mark'
import { useConsoleStore } from '@/stores/console.store'

const props = withDefaults(
  defineProps<{
    task: Task
    selected: boolean
    running: boolean
    showContext: boolean
    showCompany: boolean
    filed?: boolean
  }>(),
  { filed: false }
)

defineEmits<{ select: []; edit: [] }>()

const { steps } = storeToRefs(useConsoleStore())

const mark = computed(() => taskMark(props.task, steps.value, props.running))

// The selected row's line and gradient wash take the task's own project colour, so a row
// reads as belonging to that project rather than to a single fixed "selected" hue.
const selectTint = computed(() => identityHue(props.task.projectId))
const markLabel = computed(() => TASK_MARK_LABEL[mark.value.state] || TASK_STATUS_LABEL[props.task.status])

const awaitingReview = computed(
  () => props.task.reviewActive && props.task.stepCount === 0 && props.task.reviewState === 'CLAIMED'
)

// A short pulse on the marker the moment this row becomes the selected one, not while it stays
// selected: the settle is the arrival, and a row that is already current has nothing left to say.
const justSelected = ref(false)

watch(
  () => props.selected,
  (selected, wasSelected) => {
    if (!selected || wasSelected) return
    justSelected.value = false
    requestAnimationFrame(() => {
      justSelected.value = true
      setTimeout(() => (justSelected.value = false), 340)
    })
  }
)
</script>

<template>
  <div class="group/task relative" :class="filed && 'filed-row'">
    <button
      data-testid="task-row"
      class="focus-ring flex w-full items-start gap-2.5 rounded-[var(--radius-control)] px-2.5 py-2 text-left transition-colors"
      :class="selected ? 'selected-row text-text' : 'text-text-muted hover:bg-surface-raised hover:text-text'"
      :style="selected ? { '--select-tint': selectTint.base, '--select-tint-soft': selectTint.soft } : undefined"
      :aria-current="selected"
      @click="$emit('select')"
    >
      <span class="relative mt-[3px] grid size-3.5 shrink-0 place-items-center" :title="markLabel">
        <span
          class="absolute size-3.5 rounded-full transition-shadow"
          :class="[
            selected && mark.state === 'RESTING' && 'ring-1 ring-inset ring-accent/45',
            justSelected && 'settle'
          ]"
          aria-hidden="true"
        >
          <TaskMark
            :state="mark.state"
            :status="task.status"
            :accepted="mark.accepted"
            :selected="selected"
          />
        </span>
        <span class="sr-only">{{ markLabel }}.</span>
      </span>

      <span class="min-w-0 flex-1">
        <span class="flex items-center gap-1.5">
          <span class="min-w-0 truncate text-[13px] font-medium leading-[1.35]">{{ task.title }}</span>
          <TagBadge
            v-if="task.tagId && task.tagName && task.tagIcon && task.tagColor"
            class="ml-auto shrink-0"
            :name="task.tagName"
            :icon="task.tagIcon"
            :color="task.tagColor"
          />
        </span>
        <span class="mt-1 flex min-w-0 items-center gap-1.5">
          <template v-if="showContext && showCompany">
            <span class="shrink-0 truncate text-[10px] text-text-subtle">{{ task.companyName }}</span>
            <span class="shrink-0 text-[10px] text-text-subtle" aria-hidden="true">/</span>
          </template>
          <span class="anchor-chip min-w-0 truncate px-1.5 py-px text-[9.5px] leading-[15px]">
            {{ task.label }}
          </span>

          <!-- What the task carries, on the anchor's line so the title keeps the whole width above.
               It steps aside for the edit control under the pointer rather than sitting beneath it. -->
          <span
            class="ml-auto flex shrink-0 items-center gap-2 pl-1 font-mono text-[10px] text-text-subtle transition-opacity duration-100 group-hover/task:opacity-0 group-focus-within/task:opacity-0"
          >
            <span
              v-if="task.stepCount > 0"
              class="tabular-nums"
              :class="task.stepsDone === task.stepCount ? 'text-safe' : 'text-text-muted'"
              :title="`${task.stepsDone} of ${task.stepCount} steps done`"
              data-testid="task-steps"
            >
              {{ task.stepsDone }}/{{ task.stepCount }}
            </span>
            <span
              v-if="task.draftStepCount > 0"
              class="inline-flex items-center gap-[3px] tabular-nums"
              :title="`${task.draftStepCount} step${task.draftStepCount > 1 ? 's' : ''} in draft`"
              data-testid="task-drafts"
            >
              <span class="size-[7px] rounded-[2px] border border-dashed border-current" aria-hidden="true" />
              {{ task.draftStepCount }}
            </span>
            <span
              v-if="awaitingReview && task.draftStepCount === 0"
              class="size-1.5 rounded-full bg-accent"
              title="Awaiting your review"
              data-testid="task-review-dot"
            />
            <svg v-if="task.hasWrapup" class="size-2 text-text-muted" viewBox="0 0 12 12" role="img">
              <title>Has a wrapup</title>
              <path d="M6 1.2 10.8 6 6 10.8 1.2 6z" fill="currentColor" />
            </svg>
            <span
              v-if="task.documentCount > 0"
              class="inline-flex items-center gap-[3px] tabular-nums"
              :title="`${task.documentCount} note${task.documentCount > 1 ? 's' : ''}`"
            >
              <svg class="size-2" viewBox="0 0 12 12" fill="none" aria-hidden="true">
                <path d="M3 1.5h6v9H3z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" />
              </svg>
              {{ task.documentCount }}
            </span>
          </span>
        </span>
      </span>
    </button>

    <button
      data-testid="edit-task"
      class="focus-ring absolute bottom-[5px] right-1.5 grid size-6 translate-x-1 place-items-center rounded-md border border-border-strong bg-surface-hover text-text-subtle opacity-0 shadow-lift transition-all hover:text-accent focus-visible:translate-x-0 focus-visible:opacity-100 group-hover/task:translate-x-0 group-hover/task:opacity-100"
      :aria-label="`Edit ${task.title}`"
      @click.stop="$emit('edit')"
    >
      <svg class="size-3.5" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path
          d="M4 20h4L20 8l-4-4L4 16v4z"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linejoin="round"
        />
      </svg>
    </button>
  </div>
</template>
