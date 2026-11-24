<script setup lang="ts">
import { computed, ref, toRef } from 'vue'
import PopoverMenu, { type PopoverMenuEntry } from '@/components/ui/PopoverMenu.vue'
import StepSeal from '@/features/steps/components/StepSeal.vue'
import SkillMark from '@/features/skills/components/SkillMark.vue'
import { useStepSkills } from '@/features/skills/composables/useStepSkills'
import type { Task } from '@/features/catalog/model/catalog'
import type { SessionPlace } from '@/features/steps/lib/step-places'
import type { TaskStep } from '@/features/steps/model/task-step'
import StepRowControls from './StepRowControls.vue'

/**
 * One draft on the shelf below the checklist, a line that selects it: its title, whether a detail
 * is written, and what it will carry (skills, a new session, routes). It is written in the pane
 * beside the list, never here. Promote waits for a detail, as it does in the pane, and so does
 * placing the draft: its handle drags it onto the plan and the arrow beside Promote lists where.
 */
const props = defineProps<{
  step: TaskStep
  task: Task
  selected: boolean
  isFirst: boolean
  isLast: boolean
  promoting: boolean
  sessions: readonly SessionPlace[]
  dragging: boolean
}>()

const emit = defineEmits<{
  select: []
  promote: []
  'promote-into': [sessionNumber: number]
  'promote-alone': []
  move: [by: number]
  remove: []
  'drag-start': []
  'drag-end': []
}>()

const { effective: skills } = useStepSkills(toRef(props, 'step'), toRef(props, 'task'))
const hasDetail = computed(() => Boolean(props.step.bodyMarkdown?.trim()))
const routeCount = computed(() => props.step.links.length)

const placing = ref(false)
const handleHeld = ref(false)

const SESSION_PREFIX = 'session:'

const PLACES = computed<readonly PopoverMenuEntry[]>(() => [
  { kind: 'heading', label: 'Promote into' },
  ...props.sessions.map((session) => ({
    kind: 'item' as const,
    id: `${SESSION_PREFIX}${session.number}`,
    label: `Session ${session.number}`,
    disabled: !session.takes,
    hint: session.takes ? 'Joins it at the end' : 'Started, it takes no more steps'
  })),
  { kind: 'item', id: 'alone', label: 'A new session at the end' }
])

function pickPlace(id: string): void {
  if (id === 'alone') emit('promote-alone')
  else emit('promote-into', Number(id.slice(SESSION_PREFIX.length)))
}

function beginDrag(event: DragEvent): void {
  if (!handleHeld.value || !hasDetail.value || !event.dataTransfer) {
    event.preventDefault()
    return
  }
  event.dataTransfer.effectAllowed = 'move'
  event.dataTransfer.setData('text/plain', props.step.id)
  emit('drag-start')
}

function endDrag(): void {
  handleHeld.value = false
  emit('drag-end')
}
</script>

<template>
  <li
    class="group/draft relative grid min-w-0 cursor-pointer grid-cols-[22px_minmax(0,1fr)_auto] items-start gap-x-2.5 rounded-control py-2 pl-2 pr-1.5 transition-colors duration-(--motion-quick)"
    :class="[selected ? 'selected-row' : 'hover:bg-surface-hover', dragging && 'opacity-40']"
    :draggable="handleHeld"
    :aria-current="selected ? 'true' : undefined"
    data-testid="draft-row"
    data-seal-host
    :data-step-id="step.id"
    @click="$emit('select')"
    @dragstart="beginDrag"
    @dragend="endDrag"
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
      <PopoverMenu v-model:open="placing" :entries="PLACES" :label="`Where to promote ${step.title}`" align="right" @pick="pickPlace">
        <template #trigger="{ toggle }">
          <button
            type="button"
            class="key-quiet rounded-key focus-ring grid size-7 cursor-pointer place-items-center disabled:cursor-not-allowed disabled:opacity-40"
            :disabled="promoting || !hasDetail"
            :aria-label="`Choose where to promote ${step.title}`"
            :aria-expanded="placing"
            aria-haspopup="menu"
            data-popover-trigger
            data-testid="draft-promote-into"
            @click="toggle"
          >
            <svg class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
              <path d="M2.5 4.5 6 8l3.5-3.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </button>
        </template>
      </PopoverMenu>
      <button
        v-if="hasDetail"
        type="button"
        class="focus-ring grid size-6 cursor-grab place-items-center rounded-key text-text-subtle opacity-0 transition-colors hover:bg-surface-hover hover:text-text focus-visible:opacity-100 group-hover/draft:opacity-100 active:cursor-grabbing"
        :aria-label="`Drag ${step.title} onto the plan to promote it there`"
        data-testid="draft-handle"
        @pointerdown="handleHeld = true"
        @pointerup="handleHeld = false"
      >
        <svg class="h-3.5 w-2.5" viewBox="0 0 10 14" fill="currentColor" aria-hidden="true">
          <circle cx="2.5" cy="2.5" r="1.2" />
          <circle cx="7.5" cy="2.5" r="1.2" />
          <circle cx="2.5" cy="7" r="1.2" />
          <circle cx="7.5" cy="7" r="1.2" />
          <circle cx="2.5" cy="11.5" r="1.2" />
          <circle cx="7.5" cy="11.5" r="1.2" />
        </svg>
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
