<script setup lang="ts">
import AppBadge from '@/components/ui/AppBadge.vue'
import { previewFields, toCellText } from '@/model/mappers'
import type { Entity } from '@/model/schema'
import type { EntityRecord } from '@/model/records'
import { isResolvedReference } from '@/model/records'

const props = defineProps<{ entity: Entity; records: readonly EntityRecord[] }>()

const columns = previewFields(props.entity.fields)

function formatDate(value: Date): string {
  return value.toLocaleDateString(undefined, { day: '2-digit', month: 'short', year: 'numeric' })
}
</script>

<template>
  <div class="overflow-x-auto rounded-[var(--radius-card)] border border-border bg-surface">
    <table class="w-full text-[13px]">
      <!--
        Not sticky. These tables hold a handful of rows, so a pinned header buys nothing and,
        inside the horizontal scroll wrapper, it overlays the first row and swallows its clicks.
      -->
      <thead>
        <tr class="border-b border-border">
          <th
            class="px-4 py-2.5 text-left text-[10px] font-semibold uppercase tracking-[0.07em] text-text-subtle"
          >
            {{ entity.label }}
          </th>
          <th
            v-for="column in columns"
            :key="column.id"
            class="px-4 py-2.5 text-left text-[10px] font-semibold uppercase tracking-[0.07em] text-text-subtle"
          >
            {{ column.label }}
          </th>
          <th
            class="px-4 py-2.5 text-right text-[10px] font-semibold uppercase tracking-[0.07em] text-text-subtle"
          >
            Updated
          </th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="record in records"
          :key="record.id"
          data-testid="record-row"
          class="h-(--spacing-row) border-b border-border/60 transition-colors last:border-0 hover:bg-surface-raised"
        >
          <td class="px-4 py-2">
            <RouterLink
              :to="`/data/${entity.physicalName}/${record.id}`"
              data-testid="record-link"
              class="focus-ring font-medium text-text underline-offset-4 hover:text-accent hover:underline"
            >
              {{ record.label }}
            </RouterLink>
          </td>
          <td v-for="column in columns" :key="column.id" class="px-4 py-2 text-text-muted">
            <AppBadge v-if="isResolvedReference(record.values[column.columnName] ?? null)" tone="accent">
              {{ toCellText(record.values[column.columnName] ?? null) }}
            </AppBadge>
            <span v-else>{{ toCellText(record.values[column.columnName] ?? null) }}</span>
          </td>
          <td class="px-4 py-2 text-right text-[12px] text-text-subtle">
            {{ formatDate(record.updatedAt) }}
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
