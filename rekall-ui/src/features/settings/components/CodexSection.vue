<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { storeToRefs } from 'pinia'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import ProviderField from '@/features/settings/components/ProviderField.vue'
import ProviderPanel from '@/features/settings/components/ProviderPanel.vue'
import SessionPermissionSwitch from '@/features/settings/components/SessionPermissionSwitch.vue'
import { codexPreferences, setCodexBypass, setCodexEffort, setCodexModel } from '@/common/config/session-launch'
import { useCodexStore } from '@/features/codex/store/codex.store'
import { useToastStore } from '@/stores/toast.store'

const codex = useCodexStore()
const toast = useToastStore()
const { setup, models } = storeToRefs(codex)
const loading = ref(true)
const installing = ref(false)
const setupFailed = ref(false)
const modelsFailed = ref(false)
const saved = codexPreferences()
const model = ref(saved.model)
const effort = ref(saved.effort)
const bypass = ref(saved.skipPermissions)

const modelOptions = computed(() => {
  const offered = models.value.map((item) => ({ value: item.model, label: item.displayName }))
  if (model.value !== 'default' && !offered.some((item) => item.value === model.value)) {
    offered.push({ value: model.value, label: model.value })
  }
  return [{ value: 'default', label: 'Account default' }, ...offered]
})

const effortOptions = computed(() => {
  const selected = models.value.find((item) => item.model === model.value)
  const offered = selected?.supportedReasoningEfforts.map(({ reasoningEffort }) => ({
    value: reasoningEffort,
    label: effortLabel(reasoningEffort)
  })) ?? []
  if (effort.value !== 'default' && !offered.some((item) => item.value === effort.value) && !selected) {
    offered.push({ value: effort.value, label: effort.value })
  }
  return [{ value: 'default', label: 'Account default' }, ...offered]
})

function effortLabel(value: string): string {
  return value === 'xhigh' ? 'Extra high' : value.charAt(0).toUpperCase() + value.slice(1)
}

async function load(): Promise<void> {
  loading.value = true
  setupFailed.value = false
  modelsFailed.value = false
  try {
    await codex.loadSetup()
    if (setup.value?.status !== 'CLI_MISSING') {
      await codex.loadModels().catch(() => { modelsFailed.value = true })
    }
  } catch {
    setupFailed.value = true
  } finally {
    loading.value = false
  }
}

async function connect(): Promise<void> {
  installing.value = true
  try {
    await codex.install()
  } catch (failure: unknown) {
    toast.notifyError(failure)
  } finally {
    installing.value = false
  }
}

function chooseModel(value: string): void {
  model.value = value
  setCodexModel(value)
  const selected = models.value.find((item) => item.model === value)
  if (!selected?.supportedReasoningEfforts.some((item) => item.reasoningEffort === effort.value)) {
    chooseEffort('default')
  }
}

function chooseEffort(value: string): void {
  effort.value = value
  setCodexEffort(value)
}

function chooseBypass(value: boolean): void {
  bypass.value = value
  setCodexBypass(value)
}

onMounted(() => void load())
</script>

<template>
  <ProviderPanel provider="CODEX" data-testid="codex-settings">
    <template v-if="!loading && setup && !setupFailed" #status>
      <AppBadge :tone="setup.status === 'CONNECTED' ? 'safe' : 'neutral'" dot data-testid="codex-status">
        <template v-if="setup.status === 'CONNECTED'">Connected</template>
        <template v-else-if="setup.status === 'CLI_MISSING'">CLI missing</template>
        <template v-else>Not connected</template>
      </AppBadge>
    </template>
    <template v-if="!loading && !setupFailed && setup?.status === 'NOT_CONNECTED'" #action>
      <AppButton size="sm" variant="secondary" :loading="installing" data-testid="codex-install" @click="connect">
        Connect Codex
      </AppButton>
    </template>

    <div v-if="loading" class="px-5 py-3.5">
      <div class="skeleton h-20 rounded-control" aria-hidden="true" />
    </div>

    <template v-else-if="setup && !setupFailed">
      <p class="px-5 py-3.5 text-small leading-relaxed text-text-muted">
        <template v-if="setup.status === 'CONNECTED'">New Codex sessions can use Rekall’s MCP tools.</template>
        <template v-else-if="setup.status === 'CLI_MISSING'">Install the Codex CLI and sign in to start Codex sessions.</template>
        <template v-else>Registers the Rekall MCP server and installs the Rekall skill.</template>
      </p>

      <ProviderField label="Model" field-id="codex-model">
        <template #control="{ describedBy }">
          <AppSelect
            id="codex-model"
            :model-value="model"
            :options="modelOptions"
            :described-by="describedBy"
            data-testid="codex-model"
            @update:model-value="chooseModel"
          />
        </template>
        <div v-if="modelsFailed" class="flex flex-wrap items-center gap-2">
          <p role="status">Couldn’t read the Codex model catalog.</p>
          <AppButton size="sm" variant="secondary" @click="load">Retry</AppButton>
        </div>
        <p v-else>The models your Codex CLI offers; Account default leaves your own setting alone.</p>
      </ProviderField>

      <ProviderField label="Reasoning effort" field-id="codex-effort">
        <template #control="{ describedBy }">
          <AppSelect
            id="codex-effort"
            :model-value="effort"
            :options="effortOptions"
            :described-by="describedBy"
            :disabled="model === 'default' || effortOptions.length < 2"
            data-testid="codex-effort"
            @update:model-value="chooseEffort"
          />
        </template>
        <template v-if="model === 'default'">Choose a model to override its reasoning effort.</template>
        <template v-else>The levels the chosen model supports.</template>
      </ProviderField>

      <SessionPermissionSwitch
        label="Bypass approvals and sandbox"
        :model-value="bypass"
        data-testid="codex-bypass"
        @update:model-value="chooseBypass"
      >
        Adds <code class="font-mono text-text-muted">--dangerously-bypass-approvals-and-sandbox</code>. Codex can run
        commands without confirmation or filesystem restrictions.
      </SessionPermissionSwitch>

      <p class="border-t border-border px-5 py-3 text-small leading-relaxed text-text-subtle">
        A queued Codex task keeps the settings it had here when it was added to the run queue.
      </p>
    </template>

    <div v-else class="flex flex-col items-start gap-3 px-5 py-3.5 text-small text-text-subtle">
      <p>Couldn’t read the Codex configuration.</p>
      <AppButton size="sm" variant="secondary" @click="load">Retry</AppButton>
    </div>
  </ProviderPanel>
</template>
