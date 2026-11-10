<script setup lang="ts">
import KubeconfigImport from '@/features/connection/components/KubeconfigImport.vue'
import type { CredentialDraft, GrafanaAuthentication, JiraDeployment } from '@/features/connection/model/connection'

/** Where a credential is typed or pasted. It only ever goes out: nothing here reads one back. */
const props = defineProps<{ modelValue: CredentialDraft; id?: string }>()
const emit = defineEmits<{ 'update:modelValue': [value: CredentialDraft] }>()

const JIRA_DEPLOYMENTS: readonly { value: JiraDeployment; label: string }[] = [
  { value: 'CLOUD', label: 'Jira Cloud' },
  { value: 'DATA_CENTER', label: 'Data Center' }
]

const GRAFANA_AUTHENTICATIONS: readonly { value: GrafanaAuthentication; label: string }[] = [
  { value: 'SERVICE_ACCOUNT', label: 'Service account token' },
  { value: 'PASSWORD', label: 'User and password' }
]

const TOKEN_PLACEHOLDER: Readonly<Record<'GITLAB' | 'GITHUB', string>> = {
  GITLAB: 'glpat-…',
  GITHUB: 'github_pat_…'
}

function setToken(token: string): void {
  if (props.modelValue.kind === 'GITLAB' || props.modelValue.kind === 'GITHUB') emit('update:modelValue', { kind: props.modelValue.kind, token })
  if (props.modelValue.kind === 'JIRA' || props.modelValue.kind === 'GRAFANA') emit('update:modelValue', { ...props.modelValue, token })
}

function setGrafanaUser(user: string): void {
  if (props.modelValue.kind === 'GRAFANA') emit('update:modelValue', { ...props.modelValue, user })
}

function setGrafanaAuthentication(authentication: GrafanaAuthentication): void {
  if (props.modelValue.kind === 'GRAFANA') emit('update:modelValue', { ...props.modelValue, authentication })
}

function setEmail(email: string): void {
  if (props.modelValue.kind === 'JIRA') emit('update:modelValue', { ...props.modelValue, email })
}

function setDeployment(deployment: JiraDeployment): void {
  if (props.modelValue.kind === 'JIRA') emit('update:modelValue', { ...props.modelValue, deployment })
}
</script>

<template>
  <input
    v-if="modelValue.kind === 'GITLAB' || modelValue.kind === 'GITHUB'"
    :id="id"
    :value="modelValue.token"
    type="password"
    autocomplete="off"
    spellcheck="false"
    :data-testid="`${modelValue.kind.toLowerCase()}-token`"
    class="field text-text h-(--spacing-control) w-full rounded-control px-3 font-mono text-body"
    :placeholder="TOKEN_PLACEHOLDER[modelValue.kind]"
    @input="setToken(($event.target as HTMLInputElement).value)"
  />
  <div v-else-if="modelValue.kind === 'GRAFANA'" class="flex flex-col gap-2">
    <div class="flex gap-0.5 self-start rounded-control bg-surface p-0.5" role="group" aria-label="Grafana sign-in">
      <button
        v-for="option in GRAFANA_AUTHENTICATIONS"
        :key="option.value"
        type="button"
        class="focus-ring h-6 rounded-control px-2.5 text-caption transition-colors"
        :class="modelValue.authentication === option.value ? 'bg-surface-raised text-text' : 'text-text-subtle hover:text-text'"
        :aria-pressed="modelValue.authentication === option.value"
        :data-testid="`grafana-authentication-${option.value}`"
        @click="setGrafanaAuthentication(option.value)"
      >
        {{ option.label }}
      </button>
    </div>
    <input
      v-if="modelValue.authentication === 'PASSWORD'"
      :id="id"
      :value="modelValue.user"
      type="text"
      autocomplete="off"
      spellcheck="false"
      aria-label="Grafana user"
      data-testid="grafana-user"
      class="field text-text h-(--spacing-control) w-full rounded-control px-3 text-body"
      placeholder="User"
      @input="setGrafanaUser(($event.target as HTMLInputElement).value)"
    />
    <input
      :id="modelValue.authentication === 'PASSWORD' ? undefined : id"
      :value="modelValue.token"
      type="password"
      autocomplete="off"
      spellcheck="false"
      :aria-label="modelValue.authentication === 'PASSWORD' ? 'Grafana password' : 'Grafana service account token'"
      data-testid="grafana-token"
      class="field text-text h-(--spacing-control) w-full rounded-control px-3 font-mono text-body"
      :placeholder="modelValue.authentication === 'PASSWORD' ? 'Password' : 'glsa_…'"
      @input="setToken(($event.target as HTMLInputElement).value)"
    />
  </div>
  <div v-else-if="modelValue.kind === 'JIRA'" class="flex flex-col gap-2">
    <div class="flex gap-0.5 self-start rounded-control bg-surface p-0.5" role="group" aria-label="Jira deployment">
      <button
        v-for="option in JIRA_DEPLOYMENTS"
        :key="option.value"
        type="button"
        class="focus-ring h-6 rounded-control px-2.5 text-caption transition-colors"
        :class="modelValue.deployment === option.value ? 'bg-surface-raised text-text' : 'text-text-subtle hover:text-text'"
        :aria-pressed="modelValue.deployment === option.value"
        :data-testid="`jira-deployment-${option.value}`"
        @click="setDeployment(option.value)"
      >
        {{ option.label }}
      </button>
    </div>
    <input
      v-if="modelValue.deployment === 'CLOUD'"
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
      :id="modelValue.deployment === 'CLOUD' ? undefined : id"
      :value="modelValue.token"
      type="password"
      autocomplete="off"
      spellcheck="false"
      :aria-label="modelValue.deployment === 'CLOUD' ? 'Atlassian API token' : 'Personal access token'"
      data-testid="jira-token"
      class="field text-text h-(--spacing-control) w-full rounded-control px-3 font-mono text-body"
      :placeholder="modelValue.deployment === 'CLOUD' ? 'API token' : 'Personal access token'"
      @input="setToken(($event.target as HTMLInputElement).value)"
    />
  </div>
  <KubeconfigImport
    v-else
    :model-value="modelValue"
    @update:model-value="emit('update:modelValue', $event)"
  />
</template>
