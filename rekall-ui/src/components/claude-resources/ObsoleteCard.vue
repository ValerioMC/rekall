<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'
import MemoryTypeChip from './MemoryTypeChip.vue'
import { COVERING_LABEL, type ObsoleteMemory } from '@/model/memory-analysis'

/** A memory that a document Claude loads anyway already repeats. */
defineProps<{ finding: ObsoleteMemory; groupName: string }>()
defineEmits<{ open: [path: string] }>()
</script>

<template>
  <article class="rounded-card border border-border bg-surface p-4" data-testid="obsolete-card">
    <header class="flex items-start gap-3">
      <div class="min-w-0 flex-1">
        <div class="flex min-w-0 items-center gap-2">
          <h4 class="truncate text-body font-semibold text-text">{{ finding.memory.name }}</h4>
          <MemoryTypeChip :type="finding.memory.memoryType" />
        </div>
        <p class="mt-0.5 truncate font-mono text-micro text-text-subtle">{{ groupName }}</p>
      </div>
      <div class="w-24 shrink-0 text-right" :title="`${finding.overlapPercent}% of this memory's phrasing is in the document`">
        <span class="font-mono text-caption tabular-nums text-text">{{ finding.overlapPercent }}%</span>
        <span class="text-micro text-text-subtle"> repeated</span>
        <div class="mt-1 h-1 overflow-hidden rounded-full bg-surface-raised" role="presentation">
          <div class="h-full rounded-full bg-warn" :style="{ width: `${finding.overlapPercent}%` }" />
        </div>
      </div>
    </header>

    <p class="mt-3 text-small text-text-muted">
      Already said in
      <span class="font-medium text-text">{{ finding.coveredBy.label }}</span>
      <span class="text-text-subtle"> · {{ COVERING_LABEL[finding.coveredBy.kind] }}</span>
    </p>
    <ul v-if="finding.sharedPhrases.length > 0" class="mt-2 flex flex-wrap gap-1" aria-label="Phrases both share">
      <li v-for="phrase in finding.sharedPhrases" :key="phrase" class="rounded-key bg-surface-raised px-1.5 py-px font-mono text-micro text-text-muted">{{ phrase }}</li>
    </ul>

    <footer class="mt-3 flex flex-wrap items-center gap-2">
      <AppButton size="sm" variant="secondary" data-testid="review-open-memory" @click="$emit('open', finding.memory.path)">Open the memory</AppButton>
      <AppButton v-if="finding.coveredBy.resourcePath" size="sm" variant="quiet" @click="$emit('open', finding.coveredBy.resourcePath)">
        Open {{ finding.coveredBy.label }}
      </AppButton>
      <span v-else class="min-w-0 truncate font-mono text-micro text-text-subtle" :title="finding.coveredBy.location">{{ finding.coveredBy.location }}</span>
    </footer>
  </article>
</template>
