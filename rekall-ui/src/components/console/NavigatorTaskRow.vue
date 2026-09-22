<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import TagBadge from '@/components/ui/TagBadge.vue'
import { TASK_STATUS_COLOR, TASK_STATUS_RING } from '@/model/catalog'
import type { Task } from '@/model/catalog'

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

// The ping fires for a running timer or for a stepless task a live session is attached to.
const isRunning = computed(
  () =>
    props.running ||
    (props.task.reviewActive && props.task.stepCount === 0 && props.task.reviewState === 'RUNNING')
)

const awaitingReview = computed(
  () =>
    props.task.reviewActive &&
    props.task.stepCount === 0 &&
    props.task.reviewState === 'CLAIMED'
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
      :aria-current="selected"
      @click="$emit('select')"
    >
      <span class="relative mt-[3px] grid size-3.5 shrink-0 place-items-center" aria-hidden="true">
        <span
          class="absolute size-3.5 rounded-full transition-shadow"
          :class="[
            isRunning ? 'bg-accent/20' : TASK_STATUS_RING[task.status],
            selected && 'ring-1 ring-inset ring-accent/45',
            justSelected && 'settle'
          ]"
        />
        <template v-if="isRunning">
          <span class="absolute inline-flex size-3.5 animate-ping rounded-full bg-accent/55" />
          <span class="relative inline-flex size-2 rounded-full bg-accent" />
        </template>
        <span
          v-else
          class="relative rounded-full transition-all"
          :class="[TASK_STATUS_COLOR[task.status], selected ? 'size-2' : 'size-[7px]']"
        />
      </span>

      <span class="min-w-0 flex-1">
        <span class="flex items-center gap-1.5">
          <span class="min-w-0 truncate text-[13px] font-medium leading-[1.35]">{{ task.title }}</span>
          <TagBadge
            v-if="task.tagId && task.tagName && task.tagIcon && task.tagColor"
            class="shrink-0"
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
