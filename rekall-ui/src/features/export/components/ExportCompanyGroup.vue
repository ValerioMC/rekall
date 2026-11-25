<script setup lang="ts">
import { computed, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import CompanyMark from '@/components/ui/CompanyMark.vue'
import ExportProjectGroup from '@/features/export/components/ExportProjectGroup.vue'
import type { ExportCompanyGroup } from '@/features/export/lib/export-grouping'
import type { ExportProjectGroup as ProjectGroup } from '@/features/export/lib/export-grouping'
import type { ProjectExportId } from '@/model/branded'

/** One company's exports: a head that folds, counts and clears them, and its projects. */
const props = defineProps<{ group: ExportCompanyGroup; forceOpen: boolean }>()
const emit = defineEmits<{ delete: [id: ProjectExportId]; clearCompany: []; clearProject: [project: ProjectGroup] }>()

const open = ref(true)
const expanded = computed(() => props.forceOpen || open.value)
</script>

<template>
  <section :aria-label="group.companyName" :data-testid="`export-company-${group.companyId}`">
    <div class="flex items-center gap-2 border-b border-border pb-2">
      <button
        type="button"
        class="focus-ring flex min-w-0 flex-1 cursor-pointer items-center gap-2.5 rounded-key text-left"
        :aria-expanded="expanded"
        :data-testid="`export-company-toggle-${group.companyId}`"
        @click="open = !open"
      >
        <svg class="size-3.5 shrink-0 text-text-subtle transition-transform duration-(--motion-quick)" :class="expanded ? '' : '-rotate-90'" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path d="m3 4.5 3 3 3-3" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        <CompanyMark :company-id="group.companyId" :name="group.companyName" />
        <span class="min-w-0 truncate text-title font-semibold text-text">{{ group.companyName }}</span>
      </button>
      <span class="shrink-0 text-caption tabular-nums text-text-subtle">{{ group.count }} {{ group.count === 1 ? 'export' : 'exports' }}</span>
      <span v-if="group.failedCount > 0" class="shrink-0 text-caption tabular-nums text-danger">{{ group.failedCount }} failed</span>
      <AppButton variant="danger-quiet" size="sm" data-testid="export-company-clear" @click="emit('clearCompany')">Delete all</AppButton>
    </div>
    <div v-if="expanded" class="mt-2 flex flex-col gap-3">
      <ExportProjectGroup
        v-for="project in group.projects"
        :key="project.projectId"
        :group="project"
        :force-open="forceOpen"
        @delete="emit('delete', $event)"
        @clear="emit('clearProject', project)"
      />
    </div>
  </section>
</template>
