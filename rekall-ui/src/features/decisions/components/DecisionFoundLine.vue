<script setup lang="ts">
import { computed } from 'vue'
import DecisionFoundText from '@/features/decisions/components/DecisionFoundText.vue'
import { excerptAround, type PlaceFound } from '@/features/decisions/lib/decision-search'

/**
 * The line under a row for matches the row does not show: it quotes the first place when the row
 * shows no match, and otherwise only names the places.
 */
const props = defineProps<{ places: readonly PlaceFound[]; inRow: boolean; query: string }>()

const quoted = computed(() => (props.inRow ? undefined : props.places[0]))
const named = computed(() => names(quoted.value === undefined ? props.places : props.places.slice(1)))

function names(places: readonly PlaceFound[]): string {
  return places.map((found) => found.place.toLowerCase()).join(', ')
}
</script>

<template>
  <span class="col-[2/-1] flex min-w-0 items-baseline gap-2.5 text-small text-text-muted" data-testid="decision-found">
    <template v-if="quoted">
      <b class="shrink-0 text-caption font-medium text-text-subtle">{{ quoted.place }}</b>
      <span class="min-w-0 truncate pb-1" data-testid="decision-excerpt">
        <DecisionFoundText :text="excerptAround(quoted.text, query)" :query="query" />
      </span>
      <span v-if="named" class="shrink-0 text-caption text-text-subtle">and in {{ named }}</span>
    </template>
    <span v-else class="text-caption text-text-subtle">Also in {{ named }}</span>
  </span>
</template>
