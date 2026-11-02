<script setup lang="ts">
import ConnectionKindIcon from '@/features/connection/components/ConnectionKindIcon.vue'
import { KIND_ORDER } from '@/features/connection/lib/connection-order'
import { CONNECTION_KIND_LABEL, type ConnectionKind } from '@/features/connection/model/connection'

/** The four kinds as one choice, in the order every list uses. Fixed once the connection exists. */
defineProps<{ modelValue: ConnectionKind }>()
const emit = defineEmits<{ 'update:modelValue': [kind: ConnectionKind] }>()
</script>

<template>
  <fieldset class="mb-4">
    <legend class="mb-1.5 text-small font-medium text-text-muted">Kind</legend>
    <div class="grid grid-cols-2 gap-1.5 sm:grid-cols-4">
      <label
        v-for="kind in KIND_ORDER"
        :key="kind"
        class="has-[:focus-visible]:ring-2 has-[:focus-visible]:ring-accent flex min-h-11 cursor-pointer items-center gap-2 rounded-control border px-3 text-small transition-colors"
        :class="modelValue === kind ? 'border-accent bg-accent-soft text-accent' : 'border-border-strong bg-canvas text-text-muted hover:text-text'"
      >
        <input
          type="radio"
          name="connection-kind"
          class="sr-only"
          :value="kind"
          :checked="modelValue === kind"
          :data-testid="`kind-${kind}`"
          @change="emit('update:modelValue', kind)"
        />
        <ConnectionKindIcon :kind="kind" :size="16" />
        {{ CONNECTION_KIND_LABEL[kind] }}
      </label>
    </div>
  </fieldset>
</template>
