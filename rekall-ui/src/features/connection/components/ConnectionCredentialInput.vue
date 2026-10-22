<script setup lang="ts">
import KubeconfigImport from '@/features/connection/components/KubeconfigImport.vue'
import type { CredentialDraft } from '@/features/connection/model/connection'

/** Where a credential is typed or pasted. It only ever goes out: nothing here reads one back. */
const props = defineProps<{ modelValue: CredentialDraft; id?: string }>()
const emit = defineEmits<{ 'update:modelValue': [value: CredentialDraft] }>()

function setToken(token: string): void {
  if (props.modelValue.kind === 'GITLAB') emit('update:modelValue', { kind: 'GITLAB', token })
  if (props.modelValue.kind === 'JIRA') emit('update:modelValue', { ...props.modelValue, token })
}

function setEmail(email: string): void {
  if (props.modelValue.kind === 'JIRA') emit('update:modelValue', { ...props.modelValue, email })
}
</script>

<template>
  <input
    v-if="modelValue.kind === 'GITLAB'"
    :id="id"
    :value="modelValue.token"
    type="password"
    autocomplete="off"
    spellcheck="false"
    data-testid="gitlab-token"
    class="field text-text h-(--spacing-control) w-full rounded-control px-3 font-mono text-body"
    placeholder="glpat-…"
    @input="setToken(($event.target as HTMLInputElement).value)"
  />
  <div v-else-if="modelValue.kind === 'JIRA'" class="flex flex-col gap-2">
    <input
      :id="id"
      :value="modelValue.email"
      type="email"
      autocomplete="off"
      spellcheck="false"
      aria-label="Atlassian account email"
      data-testid="jira-email"
      class="field text-text h-(--spacing-control) w-full rounded-control px-3 text-body"
      placeholder="you@example.com"
      @input="setEmail(($event.target as HTMLInputElement).value)"
    />
    <input
      :value="modelValue.token"
      type="password"
      autocomplete="off"
      spellcheck="false"
      aria-label="Atlassian API token"
      data-testid="jira-token"
      class="field text-text h-(--spacing-control) w-full rounded-control px-3 font-mono text-body"
      placeholder="API token"
      @input="setToken(($event.target as HTMLInputElement).value)"
    />
  </div>
  <KubeconfigImport
    v-else
    :model-value="modelValue"
    @update:model-value="emit('update:modelValue', $event)"
  />
</template>
