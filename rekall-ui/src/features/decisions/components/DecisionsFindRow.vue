<script setup lang="ts">
import { computed, ref } from 'vue'
import AppInput from '@/components/ui/AppInput.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'

export type DecisionsView = 'ledger' | 'read'

/**
 * How the decisions are looked at: the search over the ledger with how many it found, and the
 * switch to what a session reads. `found` is `null` while nothing is searched.
 */
const props = defineProps<{ found: number | null; total: number }>()
const view = defineModel<DecisionsView>('view', { required: true })
const query = defineModel<string>('query', { required: true })

const field = ref<InstanceType<typeof AppInput> | null>(null)
const count = computed(() => (props.found === null ? '' : `${props.found} of ${props.total} decision${props.total === 1 ? '' : 's'}`))

function clear(): void {
  query.value = ''
  ;(field.value?.$el as HTMLInputElement | undefined)?.focus()
}

defineExpose({ clear })
</script>

<template>
  <div class="flex flex-none items-center gap-3.5 px-6 py-2.5" data-testid="decisions-find-row">
    <template v-if="view === 'ledger'">
      <div class="relative min-w-0 flex-[0_1_27.5rem]" role="search">
        <svg class="pointer-events-none absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-text-subtle" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" aria-hidden="true">
          <circle cx="7" cy="7" r="4.5" />
          <path d="m10.5 10.5 3 3" />
        </svg>
        <AppInput ref="field" v-model="query" class="pl-8 pr-8" placeholder="Search decisions" aria-label="Search decisions" autocomplete="off" spellcheck="false" data-testid="decision-search" />
        <button
          v-if="query"
          type="button"
          class="focus-ring absolute right-1 top-1/2 grid size-6 -translate-y-1/2 cursor-pointer place-items-center rounded-key text-text-subtle transition-colors duration-(--motion-quick) hover:bg-surface-raised hover:text-text"
          aria-label="Clear search"
          data-testid="decision-search-clear"
          @click="clear"
        >
          <CloseGlyph />
        </button>
      </div>
      <p class="whitespace-nowrap text-small tabular-nums text-text-muted" role="status" data-testid="decision-search-count">{{ count }}</p>
    </template>
    <span class="flex-1" />
    <SegmentedControl
      v-model="view"
      label="Show the decisions as"
      size="sm"
      :options="[
        { value: 'ledger', label: 'Ledger' },
        { value: 'read', label: 'As the session reads it' }
      ]"
    />
  </div>
</template>
