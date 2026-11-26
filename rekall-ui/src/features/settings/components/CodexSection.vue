<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { storeToRefs } from 'pinia'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import SessionChoiceGroup from '@/features/settings/components/SessionChoiceGroup.vue'
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
  <section data-testid="codex-settings" aria-labelledby="codex-settings-title">
    <h2 id="codex-settings-title" class="mb-3 eyebrow text-caption">Codex</h2>

    <div v-if="loading" class="skeleton h-20 rounded-control" aria-hidden="true" />

    <div v-else-if="setup && !setupFailed" class="rounded-control border border-border bg-canvas px-3.5 py-3">
      <div class="flex items-start justify-between gap-3">
        <div class="min-w-0 flex-1">
          <AppBadge :tone="setup.status === 'CONNECTED' ? 'safe' : 'neutral'" dot>
            <template v-if="setup.status === 'CONNECTED'">Connected</template>
            <template v-else-if="setup.status === 'CLI_MISSING'">CLI missing</template>
            <template v-else>Not connected</template>
          </AppBadge>
          <p class="mt-1.5 text-small leading-relaxed text-text-muted">
            <template v-if="setup.status === 'CONNECTED'">New Codex sessions can use Rekall’s MCP tools.</template>
            <template v-else-if="setup.status === 'CLI_MISSING'">Install the Codex CLI and sign in to start Codex sessions.</template>
            <template v-else>Registers the Rekall MCP server and installs the Rekall skill.</template>
          </p>
          <p class="mt-1 text-caption leading-relaxed text-text-subtle">
            Codex runs in Chat or Terminal. Automatic launches from the run queue use Claude Code.
          </p>
        </div>
        <AppButton v-if="setup.status === 'NOT_CONNECTED'" size="sm" variant="primary" :loading="installing" @click="connect">
          Connect Codex
        </AppButton>
      </div>

      <SessionPermissionSwitch
        label="Bypass approvals and sandbox"
        :model-value="bypass"
        data-testid="codex-bypass"
        @update:model-value="chooseBypass"
      >
        Adds <code class="text-anchor/80">--dangerously-bypass-approvals-and-sandbox</code>. Codex can run
        commands without confirmation or filesystem restrictions.
      </SessionPermissionSwitch>

      <div class="mt-3 border-t border-border pt-3" data-testid="codex-model">
        <p class="text-small text-text">Model for a “Run here” session</p>
        <SessionChoiceGroup
          label="Model for a Run here session"
          :options="modelOptions"
          :model-value="model"
          test-id-prefix="codex-model"
          @update:model-value="chooseModel"
        />
        <div v-if="modelsFailed" class="mt-2 flex flex-wrap items-center gap-2">
          <p class="text-small text-text-subtle" role="status">Couldn’t read the Codex model catalog.</p>
          <AppButton size="sm" variant="secondary" @click="load">Retry</AppButton>
        </div>
      </div>

      <div class="mt-3 border-t border-border pt-3" data-testid="codex-effort">
        <p class="text-small text-text">Reasoning effort</p>
        <p v-if="model === 'default'" class="mt-0.5 text-caption leading-relaxed text-text-subtle">
          Choose a model to override its reasoning effort.
        </p>
        <SessionChoiceGroup
          label="Reasoning effort for a Run here session"
          :options="effortOptions"
          :model-value="effort"
          :disabled="model === 'default' || effortOptions.length < 2"
          test-id-prefix="codex-effort"
          @update:model-value="chooseEffort"
        />
      </div>
    </div>

    <div v-else class="flex flex-col items-start gap-3 text-small text-text-subtle">
      <p>Couldn’t read the Codex configuration.</p>
      <AppButton size="sm" variant="secondary" @click="load">Retry</AppButton>
    </div>
  </section>
</template>
