<script setup lang="ts">
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import CompanyMark from '@/components/ui/CompanyMark.vue'
import SegmentedControl, { type SegmentOption } from '@/components/ui/SegmentedControl.vue'
import type { ProjectStatusChoice } from '@/features/catalog/model/project-status'
import type { Company } from '@/features/catalog/model/catalog'

/** The line under the sub-navigation: the status segments, the company the list is pinned to and what is on show. */
defineProps<{
  options: readonly SegmentOption<ProjectStatusChoice>[]
  company: Company | null
  companyPinned: boolean
  total: number
  reviewing: number
}>()

const status = defineModel<ProjectStatusChoice>({ required: true })

const emit = defineEmits<{ clearCompany: [] }>()
</script>

<template>
  <div class="flex flex-wrap items-center gap-2.5">
    <SegmentedControl v-model="status" :options="options" label="Status" />

    <button
      v-if="companyPinned"
      type="button"
      class="focus-ring inline-flex h-7 items-center gap-2 rounded-control border border-border-strong bg-surface-raised pl-1 pr-2 text-small text-text transition-colors hover:bg-surface-hover"
      data-testid="company-filter"
      @click="emit('clearCompany')"
    >
      <CompanyMark
        v-if="company"
        :company-id="company.id"
        :name="company.name"
        :image-version="company.logoImageVersion"
        :size="20"
      />
      <span>{{ company?.name ?? 'one company' }}</span>
      <CloseGlyph small />
    </button>

    <span class="ml-auto font-mono text-caption tabular-nums text-text-subtle">
      <span class="text-text-muted">{{ total }}</span> project{{ total === 1 ? '' : 's' }}
      <template v-if="reviewing > 0">
        · <span class="text-text-muted">{{ reviewing }}</span> to review
      </template>
    </span>
  </div>
</template>
