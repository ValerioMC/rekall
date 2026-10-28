<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import SkillMark from '@/features/skills/components/SkillMark.vue'
import TagBadge from '@/components/ui/TagBadge.vue'
import EpicChip from '@/features/catalog/components/epic/EpicChip.vue'
import TaskMark from '@/features/catalog/components/TaskMark.vue'
import TokenCount from '@/components/ui/TokenCount.vue'
import { identityHue } from '@/common/identity'
import { TASK_STATUS_LABEL } from '@/features/catalog/model/catalog'
import type { Task } from '@/features/catalog/model/catalog'
import { taskSkills } from '@/features/skills/model/skill-assignment'
import { TASK_MARK_LABEL, taskMark } from '@/features/catalog/model/task-mark'
import { useConsoleStore } from '@/stores/console.store'
import { useTokenUsageStore } from '@/features/tokens/store/tokenUsage.store'

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

const { steps, tasks, taskSkills: ownedSkills, tagSkills } = storeToRefs(useConsoleStore())
const tokenUsage = useTokenUsageStore()
const tokens = computed(() => tokenUsage.ofTask(props.task.id))

const epic = computed(() => (props.task.epicId ? tasks.value.find((task) => task.id === props.task.epicId) ?? null : null))

const skills = computed(() => taskSkills(props.task.id, props.task.tagId, ownedSkills.value, tagSkills.value))
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
      class="focus-ring flex w-full items-start gap-2.5 rounded-control px-2.5 py-2 text-left transition-colors"
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
          <span class="min-w-0 truncate text-body font-medium leading-[1.35]">{{ task.title }}</span>
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
            <span class="shrink-0 truncate text-micro text-text-subtle">{{ task.companyName }}</span>
            <span class="shrink-0 text-micro text-text-subtle" aria-hidden="true">/</span>
          </template>
          <span
            class="anchor-chip min-w-0 truncate px-1.5 py-px text-micro leading-[15px]"
            :class="epic && 'max-w-[50%] flex-1 basis-0'"
            data-testid="task-anchor"
          >
            {{ task.label }}
          </span>
          <EpicChip v-if="epic" :title="epic.title" />
        </span>

        <!-- What the task carries, on a line of its own so the anchor and the epic share the one
             above. It stops short of the edit control that appears in the corner under the pointer. -->
        <span class="mt-1.5 flex min-w-0 items-center gap-2.5 pr-7 font-mono text-micro text-text-subtle">
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
            <span class="size-[7px] rounded-key border border-dashed border-current" aria-hidden="true" />
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
          <SkillMark :skills="skills" />
          <TokenCount :totals="tokens" />
        </span>
      </span>
    </button>

    <button
      data-testid="edit-task"
      class="focus-ring absolute bottom-[5px] right-1.5 grid size-6 translate-x-1 place-items-center rounded-control border border-border-strong bg-surface-hover text-text-subtle opacity-0 shadow-lift transition-all hover:text-accent focus-visible:translate-x-0 focus-visible:opacity-100 group-hover/task:translate-x-0 group-hover/task:opacity-100"
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
