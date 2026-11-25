<script setup lang="ts">
import { computed, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import CopyAnchorButton from '@/components/console/CopyAnchorButton.vue'
import ExportRow from '@/features/export/components/ExportRow.vue'
import { identityHue } from '@/common/identity'
import type { ExportProjectGroup } from '@/features/export/lib/export-grouping'
import type { ProjectExportId } from '@/model/branded'

/** One project's exports under its company: a head that folds, counts and clears them, and the rows. */
const props = defineProps<{ group: ExportProjectGroup; forceOpen: boolean }>()
const emit = defineEmits<{ delete: [id: ProjectExportId]; clear: [] }>()

const open = ref(true)
const expanded = computed(() => props.forceOpen || open.value)
</script>

<template>
  <section :aria-label="group.projectTitle" :data-testid="`export-project-${group.projectId}`">
    <div class="flex items-center gap-2 px-3 py-2">
      <button
        type="button"
        class="focus-ring flex min-w-0 flex-1 cursor-pointer items-center gap-2 rounded-key text-left"
        :aria-expanded="expanded"
        :data-testid="`export-project-toggle-${group.projectId}`"
        @click="open = !open"
      >
        <svg class="size-3 shrink-0 text-text-subtle transition-transform duration-(--motion-quick)" :class="expanded ? '' : '-rotate-90'" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path d="m3 4.5 3 3 3-3" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        <span class="size-2 shrink-0 rounded-full" :style="{ backgroundColor: identityHue(group.projectId).base }" aria-hidden="true" />
        <span class="min-w-0 truncate text-body font-semibold text-text">{{ group.projectTitle }}</span>
      </button>
      <span class="anchor-chip shrink-0 px-1.5 py-px text-micro">project:{{ group.projectLabel }}</span>
      <CopyAnchorButton :anchor="`project:${group.projectLabel}`" />
      <span class="shrink-0 text-caption tabular-nums text-text-subtle">{{ group.entries.length }}</span>
      <span v-if="group.failedCount > 0" class="shrink-0 text-caption tabular-nums text-danger" data-testid="export-project-failed">{{ group.failedCount }} failed</span>
      <AppButton variant="danger-quiet" size="sm" data-testid="export-project-clear" @click="emit('clear')">Delete all</AppButton>
    </div>
    <ul v-if="expanded" class="m-0 list-none rounded-control border border-border bg-surface p-0">
      <ExportRow v-for="entry in group.entries" :key="entry.id" :entry="entry" @delete="emit('delete', entry.id)" />
    </ul>
  </section>
</template>
