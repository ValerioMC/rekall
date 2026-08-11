<script setup lang="ts">
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import { typeLabel } from '@/model/mappers'
import type { EntityField } from '@/model/schema'
import type { FieldId } from '@/model/branded'

defineProps<{ fields: readonly EntityField[]; displayFieldId: FieldId | null }>()

const emit = defineEmits<{ remove: [id: FieldId]; setDisplay: [id: FieldId] }>()
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
            v-for="header in ['Column', 'Type', 'Description', 'Required', 'Identifies', '']"
            :key="header"
            class="px-4 py-2.5 text-left text-[10px] font-semibold uppercase tracking-[0.07em] text-text-subtle"
          >
            {{ header }}
          </th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="field in fields"
          :key="field.id"
          data-testid="field-row"
          class="h-(--spacing-row) border-b border-border/60 transition-colors last:border-0 hover:bg-surface-raised"
        >
          <td class="px-4 py-2">
            <code class="text-text">{{ field.columnName }}</code>
          </td>
          <td class="px-4 py-2">
            <AppBadge>{{ typeLabel(field.type) }}</AppBadge>
            <span v-if="field.type === 'ENUM'" class="ml-2 font-mono text-[11.5px] text-text-subtle">
              {{ field.enumValues.join(' | ') }}
            </span>
          </td>
          <td class="px-4 py-2 text-text-muted">{{ field.description }}</td>
          <td class="px-4 py-2">
            <AppBadge v-if="!field.nullable" tone="warn">required</AppBadge>
            <span v-else class="text-[12px] text-text-subtle">optional</span>
          </td>
          <td class="px-4 py-2">
            <AppBadge v-if="displayFieldId === field.id" data-testid="identifier-badge" tone="accent" dot>
              identifier
            </AppBadge>
            <AppButton
              v-else
              variant="ghost"
              size="sm"
              data-testid="set-identifier"
              @click="emit('setDisplay', field.id)"
            >
              set
            </AppButton>
          </td>
          <td class="px-4 py-2 text-right">
            <AppButton variant="ghost" size="sm" @click="emit('remove', field.id)">remove</AppButton>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
