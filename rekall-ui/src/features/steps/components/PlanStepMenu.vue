<script setup lang="ts">
import { computed, ref } from 'vue'
import PopoverMenu, { type PopoverMenuEntry } from '@/components/ui/PopoverMenu.vue'
import type { SessionPlace, StepPlaces } from '@/features/steps/lib/step-places'

/**
 * What a person does to an open step without dragging it: move it a place, choose the session it
 * runs in, give it a session of its own, return it to draft or delete it. A session that cannot
 * take it says why.
 */
const props = defineProps<{ title: string; number: number; places: StepPlaces }>()

const emit = defineEmits<{
  'to-draft': []
  remove: []
  nudge: [by: number]
  'run-in': [sessionNumber: number]
  alone: [where: 'before-own-session' | 'end']
}>()

const open = ref(false)

const SESSION_PREFIX = 'session:'

const currentSessionNumber = computed(() => props.places.sessions.find((session) => session.current)?.number ?? 0)

function sessionHint(place: SessionPlace): string | undefined {
  if (place.current) return undefined
  if (!place.takes) return 'Started, it takes no more steps'
  return place.number < currentSessionNumber.value ? 'Joins it at the end' : 'Joins it at the start'
}

const sessionEntries = computed<PopoverMenuEntry[]>(() =>
  props.places.sessions.map((place) => ({
    kind: 'item',
    id: `${SESSION_PREFIX}${place.number}`,
    label: `Session ${place.number}`,
    current: place.current,
    disabled: !place.current && !place.takes,
    hint: sessionHint(place)
  }))
)

const aloneEntries = computed<PopoverMenuEntry[]>(() => [
  ...(props.places.aloneBeforeSession === null
    ? []
    : [{ kind: 'item' as const, id: 'alone-before', label: `New session before Session ${props.places.aloneBeforeSession}` }]),
  ...(props.places.aloneAtEnd ? [{ kind: 'item' as const, id: 'alone-end', label: 'New session at the end' }] : [])
])

const ENTRIES = computed<readonly PopoverMenuEntry[]>(() => [
  {
    kind: 'item',
    id: 'up',
    label: 'Move up',
    shortcut: '⌥ ↑',
    disabled: !props.places.canMoveUp,
    hint: props.places.canMoveUp ? undefined : 'Nothing open above it'
  },
  {
    kind: 'item',
    id: 'down',
    label: 'Move down',
    shortcut: '⌥ ↓',
    disabled: !props.places.canMoveDown,
    hint: props.places.canMoveDown ? undefined : 'Nothing open below it'
  },
  { kind: 'separator' },
  { kind: 'heading', label: 'Run in' },
  ...sessionEntries.value,
  ...aloneEntries.value,
  { kind: 'separator' },
  { kind: 'item', id: 'to-draft', label: 'Return to draft', hint: 'Takes it off the plan until you promote it again.' },
  { kind: 'item', id: 'remove', label: 'Delete step…', hint: 'Removes it for good. Asks first.', tone: 'danger' }
])

function pick(id: string): void {
  if (id === 'to-draft') emit('to-draft')
  if (id === 'remove') emit('remove')
  if (id === 'up') emit('nudge', -1)
  if (id === 'down') emit('nudge', 1)
  if (id === 'alone-before') emit('alone', 'before-own-session')
  if (id === 'alone-end') emit('alone', 'end')
  if (id.startsWith(SESSION_PREFIX)) emit('run-in', Number(id.slice(SESSION_PREFIX.length)))
}
</script>

<template>
  <PopoverMenu v-model:open="open" :entries="ENTRIES" :label="`Actions for step ${number}`" align="right" @pick="pick">
    <template #trigger="{ toggle }">
      <button
        type="button"
        class="focus-ring grid size-7 cursor-pointer place-items-center rounded-control text-text-subtle opacity-0 transition-colors duration-(--motion-quick) group-focus-within/step:opacity-100 group-hover/step:opacity-100 hover:bg-surface-raised hover:text-text aria-expanded:bg-surface-raised aria-expanded:text-text aria-expanded:opacity-100 group-aria-[current=true]/step:opacity-100"
        :aria-label="`More actions for ${title}`"
        :aria-expanded="open"
        aria-haspopup="menu"
        data-popover-trigger
        data-testid="step-more"
        @click.stop="toggle"
      >
        <svg class="size-4" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
          <circle cx="3.5" cy="8" r="1.3" />
          <circle cx="8" cy="8" r="1.3" />
          <circle cx="12.5" cy="8" r="1.3" />
        </svg>
      </button>
    </template>
  </PopoverMenu>
</template>
