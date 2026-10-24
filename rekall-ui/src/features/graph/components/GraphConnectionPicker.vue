<script setup lang="ts">
import ConnectionKindIcon from '@/features/connection/components/ConnectionKindIcon.vue'
import type { ProjectConnection } from '@/features/connection/model/connection'
import type { ConnectionId } from '@/model/branded'

/**
 * The connection a step reaches, picked among the ones the project binds: the binding is what puts
 * it in a session's context, so nothing else can be chosen. A connection without a credential is
 * flagged, since a session on it would have to stop and report.
 */
const props = defineProps<{
  modelValue: ConnectionId | null
  bindings: readonly ProjectConnection[]
  name: string
}>()

const emit = defineEmits<{ 'update:modelValue': [value: ConnectionId | null] }>()

function pick(value: ConnectionId | null): void {
  if (value !== props.modelValue) emit('update:modelValue', value)
}
</script>

<template>
  <div role="radiogroup" :aria-label="`Connection for ${name}`" class="flex flex-col gap-1.5" data-testid="graph-connection-picker">
    <button
      v-for="binding in bindings"
      :key="binding.id"
      type="button"
      role="radio"
      :aria-checked="modelValue === binding.connectionId"
      class="focus-ring flex cursor-pointer items-center gap-2.5 rounded-control border px-2.5 py-2 text-left transition-colors"
      :class="modelValue === binding.connectionId ? 'border-accent/60 bg-accent-soft' : 'border-border hover:bg-surface-raised'"
      :data-testid="`graph-connection-${binding.label}`"
      @click="pick(binding.connectionId)"
    >
      <ConnectionKindIcon :kind="binding.kind" :size="16" />
      <span class="min-w-0 flex-1">
        <span class="block truncate text-small font-medium text-text">{{ binding.title }}</span>
        <span class="block truncate font-mono text-micro text-text-muted">
          {{ binding.label }}<template v-if="binding.target"> · {{ binding.target }}</template>
        </span>
      </span>
      <span v-if="!binding.credentialSetAt" class="shrink-0 text-micro text-warn" title="A session will stop and report it">
        no credential
      </span>
    </button>
    <button
      type="button"
      role="radio"
      :aria-checked="modelValue === null"
      class="focus-ring flex cursor-pointer items-center gap-2.5 rounded-control border px-2.5 py-2 text-left text-small transition-colors"
      :class="modelValue === null ? 'border-accent/60 bg-accent-soft text-text' : 'border-border text-text-muted hover:bg-surface-raised'"
      data-testid="graph-connection-none"
      @click="pick(null)"
    >
      <svg class="size-4 shrink-0 text-text-subtle" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <circle cx="8" cy="8" r="5.25" stroke="currentColor" stroke-width="1.3" stroke-dasharray="2.4 2.2" />
      </svg>
      Reaches nothing outside the repository
    </button>
    <p v-if="!bindings.length" class="text-caption leading-relaxed text-text-subtle">
      This project binds no connection yet. Add one under <strong class="font-medium text-text-muted">Connections</strong>
      in the project's settings, then pick it here.
    </p>
  </div>
</template>
