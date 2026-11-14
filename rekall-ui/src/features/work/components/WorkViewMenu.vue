<script setup lang="ts">
import { ref } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import SegmentedControl, { type SegmentOption } from '@/components/ui/SegmentedControl.vue'
import QuickTaskFilter from '@/features/catalog/components/QuickTaskFilter.vue'
import type { WorkDensity } from '@/features/work/model/work-view'
import { useWorkStore } from '@/features/work/store/work.store'

/** What changes how the ledger is drawn, not what it lists: row density, and whether quick tasks show. */
defineProps<{ showsDensity: boolean; showsQuickTasks: boolean }>()

const DENSITY_OPTIONS: readonly SegmentOption<WorkDensity>[] = [
  { value: 'comfortable', label: 'Comfortable' },
  { value: 'compact', label: 'Compact' }
]

const { density } = storeToRefs(useWorkStore())
const isOpen = ref(false)

function onKeydown(event: KeyboardEvent): void {
  if (event.key !== 'Escape') return
  event.stopPropagation()
  isOpen.value = false
}
</script>

<template>
  <div class="relative" data-testid="work-view-menu" @keydown="onKeydown">
    <AppButton variant="ghost" aria-haspopup="true" :aria-expanded="isOpen" data-testid="work-view-trigger" @click="isOpen = !isOpen">
      <svg class="size-4" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <path d="M2.5 5h6M11.5 5h2M2.5 11h2M7.5 11h6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        <circle cx="10" cy="5" r="1.5" stroke="currentColor" stroke-width="1.5" />
        <circle cx="6" cy="11" r="1.5" stroke="currentColor" stroke-width="1.5" />
      </svg>
      View
    </AppButton>

    <div
      v-if="isOpen"
      class="rise absolute right-0 top-[calc(100%+6px)] z-(--z-overlay) flex w-64 flex-col gap-4 rounded-card border border-border-strong bg-surface-raised p-4 shadow-(--shadow-lift)"
      role="group"
      aria-label="View options"
      data-testid="work-view-panel"
    >
      <div v-if="showsDensity" class="flex flex-col gap-2">
        <p class="text-caption font-medium text-text-subtle">Row density</p>
        <SegmentedControl v-model="density" :options="DENSITY_OPTIONS" label="Density" />
      </div>
      <QuickTaskFilter v-if="showsQuickTasks" />
    </div>

    <button v-if="isOpen" type="button" class="fixed inset-0 z-(--z-sticky) cursor-default" tabindex="-1" aria-hidden="true" @click="isOpen = false" />
  </div>
</template>
