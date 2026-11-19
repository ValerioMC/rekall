<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import SegmentedControl, { type SegmentOption } from '@/components/ui/SegmentedControl.vue'
import WorkViewMenu from '@/features/work/components/WorkViewMenu.vue'
import { useWorkLayout, type WorkLayout } from '@/features/work/composables/useWorkLayout'
import { useWorkStore } from '@/features/work/store/work.store'
import { useConsoleStore } from '@/stores/console.store'

/** The filter, the layout (Work only) and the view menu: what a person adjusts about the list, apart from which project it shows. */
const LAYOUT_OPTIONS: readonly SegmentOption<WorkLayout>[] = [
  { value: 'ledger', label: 'Ledger' },
  { value: 'board', label: 'Board' }
]

defineProps<{ choosesLayout: boolean }>()

const { tasks } = storeToRefs(useConsoleStore())
const { query } = storeToRefs(useWorkStore())
const layout = useWorkLayout()

const isLedger = computed(() => layout.value === 'ledger')
const hasQuickTasks = computed(() => tasks.value.some((task) => task.quick))
</script>

<template>
  <input
    v-model="query"
    type="text"
    autocomplete="off"
    spellcheck="false"
    placeholder="Filter by title or anchor"
    aria-label="Filter the work"
    class="field h-8 w-60 rounded-control px-3 text-small text-text"
    data-testid="work-filter"
  />
  <SegmentedControl v-if="choosesLayout" v-model="layout" :options="LAYOUT_OPTIONS" label="Layout" />
  <WorkViewMenu v-if="isLedger || hasQuickTasks" :shows-density="isLedger" :shows-quick-tasks="hasQuickTasks" />
</template>
