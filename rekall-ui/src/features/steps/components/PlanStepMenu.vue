<script setup lang="ts">
import { computed, ref } from 'vue'
import PopoverMenu, { type PopoverMenuEntry } from '@/components/ui/PopoverMenu.vue'

/**
 * What a person rarely does to an open step, behind one key and said in words: take it off the
 * plan until it is promoted again, or delete it for good. Everything else a row does has a place of
 * its own, so nothing here is a click whose effect has to be guessed.
 */
defineProps<{ title: string; number: number }>()

const emit = defineEmits<{ 'to-draft': []; remove: [] }>()

const open = ref(false)

const ENTRIES = computed<readonly PopoverMenuEntry[]>(() => [
  { kind: 'item', id: 'to-draft', label: 'Return to draft', hint: 'Takes it off the plan until you promote it again.' },
  { kind: 'separator' },
  { kind: 'item', id: 'remove', label: 'Delete step…', hint: 'Removes it for good. Asks first.', tone: 'danger' }
])

function pick(id: string): void {
  if (id === 'to-draft') emit('to-draft')
  if (id === 'remove') emit('remove')
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
