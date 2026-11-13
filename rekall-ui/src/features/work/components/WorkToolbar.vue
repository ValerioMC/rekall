<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import SegmentedControl, { type SegmentOption } from '@/components/ui/SegmentedControl.vue'
import SubNav, { type SubNavItem } from '@/components/ui/SubNav.vue'
import QuickTaskFilter from '@/features/catalog/components/QuickTaskFilter.vue'
import ScopePicker from '@/features/catalog/components/ScopePicker.vue'
import { useWorkLayout, type WorkLayout } from '@/features/work/composables/useWorkLayout'
import { useConsoleStore } from '@/stores/console.store'
import { useWorkStore } from '@/features/work/store/work.store'
import { WORK_VIEWS, WORK_VIEW_LABEL, type WorkDensity, type WorkView } from '@/features/work/model/work-view'

defineEmits<{ newTask: [] }>()

const store = useConsoleStore()
const work = useWorkStore()
const { tasks, scopedProjects } = storeToRefs(store)
const { view, density, query, counts } = storeToRefs(work)

const layout = useWorkLayout()

const LAYOUT_OPTIONS: readonly SegmentOption<WorkLayout>[] = [
  { value: 'ledger', label: 'Ledger' },
  { value: 'board', label: 'Board' }
]

const hasQuickTasks = computed(() => tasks.value.some((task) => task.quick))
const canCreate = computed(() => scopedProjects.value.length > 0)

const DENSITY_OPTIONS: readonly SegmentOption<WorkDensity>[] = [
  { value: 'comfortable', label: 'Comfortable' },
  { value: 'compact', label: 'Compact' }
]

const tabs = computed<SubNavItem<WorkView>[]>(() =>
  WORK_VIEWS.map((id) => ({ id, label: WORK_VIEW_LABEL[id], count: counts.value[id], hot: id === 'needs' }))
)
</script>

<template>
  <div data-testid="work-toolbar">
    <div class="flex flex-wrap items-center gap-3 pb-3.5 pt-5.5">
      <h1 class="text-heading font-medium tracking-[-0.005em] text-text">Work</h1>
      <div class="w-72"><ScopePicker /></div>
      <input
        v-model="query"
        type="text"
        autocomplete="off"
        spellcheck="false"
        placeholder="Filter by title or anchor"
        aria-label="Filter the work"
        class="field h-9 w-70 rounded-control px-3 text-body text-text"
        data-testid="work-filter"
      />
      <QuickTaskFilter v-if="hasQuickTasks" />
      <span class="flex-1" />
      <SegmentedControl v-model="layout" :options="LAYOUT_OPTIONS" label="Layout" />
      <SegmentedControl v-if="layout === 'ledger'" v-model="density" :options="DENSITY_OPTIONS" label="Density" />
      <AppButton v-if="canCreate" variant="primary" data-testid="new-task" @click="$emit('newTask')">
        <span aria-hidden="true">+</span> New task
      </AppButton>
    </div>
    <SubNav v-if="layout === 'ledger'" v-model="view" :items="tabs" label="Work views" />
  </div>
</template>
