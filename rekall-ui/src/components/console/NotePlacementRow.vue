<script setup lang="ts">
import { computed, ref } from 'vue'
import { TASK_STATUS_COLOR, TASK_STATUS_LABEL } from '@/model/catalog'
import type { Task } from '@/model/catalog'

/**
 * One task in a note-side list, in one of two modes.
 *
 * Picking (`removable` off): the whole row is a button with a tick that says whether the note
 * is (or will be) on the task, and a click flips it. The new-note composer uses this.
 *
 * Placed (`removable` on): the row is where the note already is, and it does not react to a
 * click on its body. The only way off is the `remove` control at its right end, always visible
 * so the list reads as a list of removable placements, and under the pointer it colours the
 * whole row with what it is about to do. When the task is the only one the note is on
 * (`locked`), the same control is a bin rather than a cross and says "Delete note": taking the
 * note off its last task is deleting it, and the parent asks before doing so.
 */
const props = withDefaults(
  defineProps<{
    task: Task
    attached: boolean
    walkIndex: number
    highlighted?: boolean
    busy?: boolean
    locked?: boolean
    openable?: boolean
    removable?: boolean
  }>(),
  { highlighted: false, busy: false, locked: false, openable: false, removable: false }
)

const emit = defineEmits<{ toggle: []; remove: []; open: []; hover: [] }>()

/** True while the pointer or focus is on the remove control: the row shows what it would lose. */
const armed = ref(false)

const pickTitle = computed(() =>
  props.attached ? `Take this note off ${props.task.title}` : `Put this note on ${props.task.title}`
)

const removeLabel = computed(() => (props.locked ? 'Delete note' : 'Take off'))

const removeTitle = computed(() =>
  props.locked
    ? `${props.task.title} is the only task this note is on: taking it off deletes the note`
    : `Take this note off ${props.task.title}`
)
</script>

<template>
  <div
    class="group/row relative rounded-[var(--radius-control)] transition-colors duration-150"
    :class="[removable && armed && (locked ? 'placement-armed-delete' : 'placement-armed'), busy && 'opacity-60']"
    :data-armed="removable && armed ? (locked ? 'delete' : 'remove') : undefined"
  >
    <component
      :is="removable ? 'div' : 'button'"
      :type="removable ? undefined : 'button'"
      class="flex w-full items-center gap-2.5 rounded-[var(--radius-control)] py-1.5 pl-2 text-left transition-colors"
      :class="[
        removable ? (openable ? 'pr-[68px]' : 'pr-11') : 'pr-2',
        !removable && (highlighted ? 'bg-surface-raised' : 'hover:bg-surface-raised'),
        !removable && 'focus-ring cursor-pointer'
      ]"
      :aria-pressed="removable ? undefined : attached"
      :title="removable ? undefined : pickTitle"
      :data-walk-index="walkIndex"
      :data-attached="attached"
      data-testid="note-placement-row"
      @mousemove="!removable && emit('hover')"
      @click="!removable && emit('toggle')"
    >
      <span
        v-if="!removable"
        class="grid size-4 shrink-0 place-items-center rounded-[5px] border transition-colors"
        :class="
          attached
            ? 'border-accent-deep bg-accent-soft text-accent'
            : 'border-border-strong text-transparent group-hover/row:border-accent/60'
        "
        aria-hidden="true"
      >
        <svg class="size-3" viewBox="0 0 24 24" fill="none">
          <path
            d="M5 12.5l4.5 4.5L19 7.5"
            stroke="currentColor"
            stroke-width="2.6"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </span>
      <span
        v-else
        class="size-[7px] shrink-0 rounded-full transition-colors"
        :class="armed ? 'bg-danger' : TASK_STATUS_COLOR[task.status]"
        :title="TASK_STATUS_LABEL[task.status]"
        aria-hidden="true"
      />

      <span class="min-w-0 flex-1">
        <span
          class="block truncate text-[12.5px] leading-[1.35] transition-[text-decoration-color] duration-150"
          :class="[
            attached ? 'text-text' : 'text-text-muted',
            removable && armed && 'line-through decoration-danger/70'
          ]"
        >
          {{ task.title }}
        </span>
        <span class="mt-0.5 flex items-center gap-1.5">
          <span
            v-if="!removable"
            class="size-[6px] shrink-0 rounded-full"
            :class="TASK_STATUS_COLOR[task.status]"
            :title="TASK_STATUS_LABEL[task.status]"
          />
          <span class="anchor-chip truncate px-1.5 py-px text-[9.5px] leading-[15px]">
            {{ task.label }}
          </span>
          <span v-if="locked" class="truncate text-[10px] text-accent/80">only here</span>
        </span>
      </span>
    </component>

    <!--
      The placed row's controls sit outside its body so a click on the title does nothing.
      The remove control carries its own label when armed, so what a click will do is written
      on the row before it happens.
    -->
    <span
      v-if="removable"
      class="absolute inset-y-0 right-1.5 flex items-center gap-0.5"
    >
      <button
        type="button"
        class="focus-ring placement-remove flex h-6 items-center gap-1 rounded-full border pr-1.5 pl-1 font-mono text-[10px] transition-[color,background-color,border-color] duration-150"
        :class="
          armed
            ? 'border-danger/60 bg-danger-soft text-danger'
            : 'border-border-strong text-text-subtle'
        "
        :title="removeTitle"
        :aria-label="locked ? `Delete this note, ${task.title} is its only task` : `Take this note off ${task.title}`"
        :disabled="busy"
        :data-testid="locked ? 'note-placement-delete' : 'note-placement-remove'"
        @mouseenter="armed = true"
        @mouseleave="armed = false"
        @focus="armed = true"
        @blur="armed = false"
        @click.stop="emit('remove')"
      >
        <svg v-if="locked" class="size-3 shrink-0" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path d="M2.5 3.5h7M4.5 3.5V2.5h3v1M3.5 3.5l.5 6h4l.5-6" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        <svg v-else class="size-3 shrink-0" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path d="M3 3l6 6M9 3l-6 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
        <span
          class="placement-remove-label overflow-hidden whitespace-nowrap transition-[max-width,opacity] duration-150"
          :class="armed ? 'max-w-[80px] opacity-100' : 'max-w-0 opacity-0'"
          aria-hidden="true"
        >
          {{ removeLabel }}
        </span>
      </button>

      <button
        v-if="openable && attached"
        type="button"
        class="focus-ring grid size-6 place-items-center rounded-md text-text-subtle opacity-0 transition-opacity hover:bg-surface-hover hover:text-accent focus-visible:opacity-100 group-hover/row:opacity-100"
        :title="`Open ${task.title} on the Tasks side`"
        :aria-label="`Open ${task.title}`"
        data-testid="note-placement-open"
        @click.stop="emit('open')"
      >
        <svg class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path
            d="M2.5 6h7M6.5 3l3 3-3 3"
            stroke="currentColor"
            stroke-width="1.4"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </button>
    </span>
  </div>
</template>

<style scoped>
/*
 * Arming the remove control colours the row it would empty: a coral wash and a hairline on the
 * left that overrides the project's identity rule, so the eye is told which row is about to go
 * before the click. Deleting the note is the same wash one shade heavier.
 */
.placement-armed {
  background-color: var(--color-danger-soft);
  box-shadow: inset 2px 0 0 var(--color-danger);
}

.placement-armed-delete {
  background-color: color-mix(in srgb, var(--color-danger) 16%, transparent);
  box-shadow: inset 2px 0 0 var(--color-danger);
}

@media (prefers-reduced-motion: reduce) {
  .placement-remove,
  .placement-remove-label {
    transition: none;
  }
}
</style>
