<script setup lang="ts">
import { computed, toRef } from 'vue'
import StepSeal from '@/features/steps/components/StepSeal.vue'
import SkillMark from '@/features/skills/components/SkillMark.vue'
import { useStepSkills } from '@/features/skills/composables/useStepSkills'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStep } from '@/features/steps/model/task-step'
import StepRowControls from './StepRowControls.vue'

/**
 * One draft on the shelf below the checklist, a line that selects it: its title, whether a detail
 * is written, and what it will carry (skills, a new session, routes). It is written in the pane
 * beside the list, never here. Promote waits for a detail, as it does in the pane.
 */
const props = defineProps<{
  step: TaskStep
  task: Task
  selected: boolean
  isFirst: boolean
  isLast: boolean
  promoting: boolean
}>()

defineEmits<{ select: []; promote: []; move: [by: number]; remove: [] }>()

const { effective: skills } = useStepSkills(toRef(props, 'step'), toRef(props, 'task'))
const hasDetail = computed(() => Boolean(props.step.bodyMarkdown?.trim()))
const routeCount = computed(() => props.step.links.length)
</script>

<template>
  <li
    class="group/draft relative grid min-w-0 cursor-pointer grid-cols-[22px_minmax(0,1fr)_auto] items-start gap-x-2.5 rounded-control py-2 pl-2 pr-1.5 transition-colors duration-(--motion-quick)"
    :class="selected ? 'selected-row' : 'hover:bg-surface-hover'"
    :aria-current="selected ? 'true' : undefined"
    data-testid="draft-row"
    data-seal-host
    :data-step-id="step.id"
    @click="$emit('select')"
  >
    <span class="mt-px size-[22px] shrink-0">
      <StepSeal state="DRAFT" />
    </span>

    <span class="flex min-w-0 flex-col gap-0.5">
      <button
        type="button"
        class="focus-ring line-clamp-2 rounded-key text-left text-body leading-snug text-text"
        data-testid="draft-title"
        @click.stop="$emit('select')"
      >
        {{ step.title }}
      </button>
      <span class="flex flex-wrap items-center gap-x-2.5 gap-y-1 text-caption text-text-subtle">
        <span v-if="hasDetail" class="flex items-center gap-1" data-testid="draft-has-detail">
          <svg class="size-2.5" viewBox="0 0 12 12" fill="none" aria-hidden="true">
            <path d="M1.6 3h8.8M1.6 6h8.8M1.6 9h5.4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
          </svg>
          Detail written
        </span>
        <span v-else>No detail yet</span>
        <SkillMark :skills="skills" />
        <span v-if="routeCount" data-testid="draft-routes">{{ routeCount }} {{ routeCount === 1 ? 'route' : 'routes' }}</span>
        <span
          v-if="step.startsSession"
          class="rounded-key border border-accent/40 px-1.5 py-px text-micro font-semibold tracking-[0.02em] text-accent"
          data-testid="draft-new-session"
        >
          new session
        </span>
      </span>
    </span>

    <span class="flex shrink-0 items-center gap-0.5" @click.stop>
      <button
        class="key-quiet rounded-key focus-ring group/promote inline-flex h-7 shrink-0 items-center gap-1.5 px-2.5 text-caption font-medium disabled:cursor-not-allowed disabled:opacity-40"
        :class="promoting && 'settle'"
        :disabled="promoting || !hasDetail"
        :title="hasDetail ? `Put ${step.title} on the checklist` : 'Write the detail to promote it'"
        data-testid="draft-promote"
        @click="$emit('promote')"
      >
        <svg
          class="size-3 transition-transform duration-(--motion-move) group-hover/promote:-translate-y-px"
          viewBox="0 0 12 12"
          fill="none"
          aria-hidden="true"
        >
          <path
            d="M6 9.8V4M3.4 6.4 6 3.8l2.6 2.6M2.4 1.8h7.2"
            stroke="currentColor"
            stroke-width="1.3"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        Promote
      </button>
      <span class="flex items-center gap-0.5 opacity-0 transition-opacity focus-within:opacity-100 group-hover/draft:opacity-100">
        <StepRowControls
          :title="step.title"
          :can-move-up="!isFirst"
          :can-move-down="!isLast"
          testid-prefix="draft"
          @up="$emit('move', -1)"
          @down="$emit('move', 1)"
          @remove="$emit('remove')"
        />
      </span>
    </span>
  </li>
</template>
