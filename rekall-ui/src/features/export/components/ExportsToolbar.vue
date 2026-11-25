<script setup lang="ts">
import { computed } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppInput from '@/components/ui/AppInput.vue'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'
import { formatTimeOfDay } from '@/common/format/app-locale'
import type { ExportFilter, ExportFormatFilter, ExportStatusFilter } from '@/features/export/lib/export-filter'

/** Search and the status and format filters over the loaded exports, when it last loaded, and the clear of every failed one. */
const props = defineProps<{ counts: Readonly<{ all: number; done: number; failed: number }>; loadedAt: Date | null }>()
const filter = defineModel<ExportFilter>({ required: true })
const emit = defineEmits<{ clearFailed: [] }>()

const statusOptions = computed(() => [
  { value: 'all', label: 'All', count: props.counts.all },
  { value: 'done', label: 'Done', count: props.counts.done },
  { value: 'failed', label: 'Failed', count: props.counts.failed }
])
const formatOptions = [
  { value: 'all', label: 'All' },
  { value: 'PDF', label: 'PDF' },
  { value: 'DOCX', label: 'Word' }
] as const

const query = computed({ get: () => filter.value.query, set: (query: string) => (filter.value = { ...filter.value, query }) })
const status = computed({ get: () => filter.value.status, set: (status: ExportStatusFilter) => (filter.value = { ...filter.value, status }) })
const format = computed({ get: () => filter.value.format, set: (format: ExportFormatFilter) => (filter.value = { ...filter.value, format }) })
</script>

<template>
  <div class="flex flex-wrap items-center gap-x-3 gap-y-2" data-testid="exports-toolbar">
    <div class="w-64"><AppInput v-model="query" type="search" placeholder="Search what was exported" /></div>
    <SegmentedControl v-model="status" :options="statusOptions" label="Status" />
    <SegmentedControl v-model="format" :options="formatOptions" label="Format" />
    <span class="ml-auto flex items-center gap-3">
      <span v-if="loadedAt" class="text-caption text-text-subtle" data-testid="exports-updated">Updated {{ formatTimeOfDay(loadedAt) }}</span>
      <AppButton v-if="counts.failed > 0" variant="danger-quiet" size="sm" data-testid="export-clear-failed" @click="emit('clearFailed')">
        Delete failed {{ counts.failed }}
      </AppButton>
    </span>
  </div>
</template>
