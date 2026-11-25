<script setup lang="ts">
import CompanyMark from '@/components/ui/CompanyMark.vue'
import ProjectCompositionBar from '@/features/catalog/components/ProjectCompositionBar.vue'
import { plural } from '@/features/connection/lib/connection-impact'
import type { ProjectComposition } from '@/features/catalog/lib/project-composition'
import type { Company } from '@/features/catalog/model/catalog'

/**
 * One company in the list beside the detail: its size, where its work stands and what waits for review,
 * so the next company to open is visible without opening it. Beside a narrow detail it becomes a strip card.
 */
defineProps<{
  company: Company
  composition: ProjectComposition
  reviewing: number
  selected: boolean
}>()

const emit = defineEmits<{ select: [] }>()
</script>

<template>
  <button
    type="button"
    class="focus-ring grid w-full grid-cols-[32px_minmax(0,1fr)] items-center gap-x-3 gap-y-2 rounded-control p-3 text-left transition-colors @max-[56rem]:w-57 @max-[56rem]:shrink-0"
    :class="
      selected
        ? 'bg-accent-soft ring-1 ring-inset ring-accent/20'
        : 'hover:bg-surface-raised @max-[56rem]:border @max-[56rem]:border-border @max-[56rem]:bg-surface'
    "
    :aria-current="selected ? 'true' : undefined"
    data-testid="company-card"
    @click="emit('select')"
  >
    <CompanyMark :company-id="company.id" :name="company.name" :image-version="company.logoImageVersion" :size="32" />
    <span class="min-w-0">
      <span class="block truncate text-body font-semibold text-text">{{ company.name }}</span>
      <span class="flex items-center whitespace-nowrap text-caption text-text-subtle">
        {{ plural(company.projectCount, 'project') }}
        <span class="@max-[56rem]:hidden">&nbsp;· {{ plural(company.taskCount, 'task') }}</span>
        <span v-if="reviewing > 0" class="ml-auto font-medium text-accent" data-testid="company-needs">
          {{ reviewing }} to review
        </span>
      </span>
    </span>
    <ProjectCompositionBar class="col-start-2" :composition="composition" />
  </button>
</template>
