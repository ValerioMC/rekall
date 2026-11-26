<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import CopyGlyph from '@/components/ui/CopyGlyph.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import ProviderField from '@/features/settings/components/ProviderField.vue'
import ProviderPanel from '@/features/settings/components/ProviderPanel.vue'
import SessionPermissionSwitch from '@/features/settings/components/SessionPermissionSwitch.vue'
import { storeToRefs } from 'pinia'
import { useClaudeConnectionStore } from '@/features/claude/store/claudeConnection.store'
import { useClaudeCommandUpdateStore } from '@/features/claude/store/claudeCommandUpdate.store'
import CommandUpdateMark from '@/features/claude/components/CommandUpdateMark.vue'
import { useToastStore } from '@/stores/toast.store'
import {
  preferredEffort,
  preferredModel,
  setPreferredEffort,
  setPreferredModel,
  setSkipsPermissions,
  skipsPermissions
} from '@/common/config/claude-launch'
import {
  CLAUDE_EFFORT_CHOICES,
  CLAUDE_MODEL_CHOICES,
  claudeEffortChoiceLabel,
  claudeModelChoiceLabel
} from '@/features/claude/model/claude'
import type { ClaudeConnectionStatus, ClaudeEffortChoice, ClaudeModelChoice } from '@/features/claude/model/claude'

const toast = useToastStore()

const connection = useClaudeConnectionStore()
const { installation } = storeToRefs(connection)
const { updating: updatingCommands } = storeToRefs(useClaudeCommandUpdateStore())
const loading = ref(true)
const installing = ref(false)
const installed = ref(false)
const copied = ref(false)

interface Presentation {
  readonly tone: 'neutral' | 'safe' | 'warn'
  readonly label: string
  readonly action: string | null
  readonly variant: 'primary' | 'secondary'
}

const PRESENTATION: Readonly<Record<ClaudeConnectionStatus, Presentation>> = {
  CONNECTED: { tone: 'safe', label: 'Connected', action: 'Reinstall', variant: 'secondary' },
  OUTDATED: { tone: 'warn', label: 'Out of date', action: 'Repair', variant: 'primary' },
  NOT_CONNECTED: { tone: 'neutral', label: 'Not connected', action: 'Connect', variant: 'secondary' },
  CLI_MISSING: { tone: 'neutral', label: 'Claude Code not found', action: null, variant: 'secondary' }
}

const presentation = computed<Presentation | null>(() =>
  installation.value ? PRESENTATION[installation.value.status] : null
)

const explanation = computed<string>(() => {
  const current = installation.value
  if (!current) return ''
  switch (current.status) {
    case 'CONNECTED':
      return 'A new session finds Rekall here, in any folder, and starts with /rk.'
    case 'OUTDATED':
      if (current.registeredUrl && current.registeredUrl !== current.endpoint) {
        return `Claude Code is pointed at ${current.registeredUrl}, which is not where this instance is serving.`
      }
      if (current.folderScoped.length) {
        return current.folderScoped.length === 1
          ? 'One folder keeps a setup of its own, which wins inside it. Repair clears it.'
          : `${current.folderScoped.length} folders keep a setup of their own, which wins inside them. Repair clears them.`
      }
      return 'The registration is right, but the installed /rk command is an older copy.'
    case 'NOT_CONNECTED':
      return 'Registers the server for every folder and installs the /rk command.'
    default:
      return 'There is no claude binary here to write the registration with. Run this in a terminal instead:'
  }
})

async function load(): Promise<void> {
  loading.value = true
  try {
    await connection.load()
  } catch (caught) {
    toast.notifyError(caught)
  } finally {
    loading.value = false
  }
}

async function install(): Promise<void> {
  installing.value = true
  try {
    await connection.install()
    installed.value = true
    toast.notify('Registered. A session started from now on will find it.')
  } catch (caught) {
    toast.notifyError(caught)
  } finally {
    installing.value = false
  }
}

const skipPermissions = ref(skipsPermissions())

function chooseSkipPermissions(value: boolean): void {
  skipPermissions.value = value
  setSkipsPermissions(value)
}

const modelChoices = CLAUDE_MODEL_CHOICES.map((value) => ({ value, label: claudeModelChoiceLabel(value) }))
const model = ref<ClaudeModelChoice>(preferredModel())

function chooseModel(value: string): void {
  const choice = CLAUDE_MODEL_CHOICES.find((candidate) => candidate === value)
  if (!choice) return
  model.value = choice
  setPreferredModel(choice)
}

const effortChoices = CLAUDE_EFFORT_CHOICES.map((value) => ({ value, label: claudeEffortChoiceLabel(value) }))
const effort = ref<ClaudeEffortChoice>(preferredEffort())

function chooseEffort(value: string): void {
  const choice = CLAUDE_EFFORT_CHOICES.find((candidate) => candidate === value)
  if (!choice) return
  effort.value = choice
  setPreferredEffort(choice)
}

async function copyCommand(): Promise<void> {
  const command = installation.value?.manualCommand
  if (!command || !navigator.clipboard) return
  await navigator.clipboard.writeText(command)
  copied.value = true
  window.setTimeout(() => (copied.value = false), 1500)
}

onMounted(() => void load())
</script>

<template>
  <ProviderPanel provider="CLAUDE" data-testid="claude-section">
    <template v-if="!loading && installation" #status>
      <AppBadge v-if="updatingCommands" tone="accent" data-testid="claude-status">
        <CommandUpdateMark />
      </AppBadge>
      <AppBadge v-else :tone="presentation!.tone" dot data-testid="claude-status">
        {{ presentation!.label }}
      </AppBadge>
    </template>
    <template v-if="!loading && presentation?.action && !updatingCommands" #action>
      <AppButton
        size="sm"
        :variant="presentation.variant"
        :loading="installing"
        data-testid="claude-install"
        @click="install"
      >
        {{ presentation.action }}
      </AppButton>
    </template>

    <div v-if="loading" class="px-5 py-3.5">
      <div class="skeleton h-20 rounded-control" aria-hidden="true" />
    </div>

    <template v-else-if="installation">
      <div class="min-w-0 px-5 py-3.5">
        <p class="text-small leading-relaxed text-text-muted">{{ explanation }}</p>
        <div
          v-if="installation.status === 'CLI_MISSING'"
          class="mt-2.5 flex items-center gap-2 rounded-control border border-border bg-canvas px-2.5 py-2"
        >
          <code class="min-w-0 flex-1 break-words font-mono text-caption text-text">
            {{ installation.manualCommand }}
          </code>
          <AppButton
            size="sm"
            variant="ghost"
            data-testid="claude-copy"
            :title="copied ? 'Copied' : 'Copy this command'"
            :aria-label="copied ? 'Copied' : 'Copy this command'"
            @click="copyCommand"
          >
            <CopyGlyph :copied="copied" />
          </AppButton>
        </div>
        <p v-if="installed" class="mt-2 text-small text-text" data-testid="claude-restart-hint">
          Sessions already open keep the configuration they started with. This one needs restarting.
        </p>
      </div>

      <ProviderField label="Model" field-id="claude-model">
        <template #control="{ describedBy }">
          <AppSelect
            id="claude-model"
            :model-value="model"
            :options="modelChoices"
            :described-by="describedBy"
            data-testid="claude-model"
            @update:model-value="chooseModel"
          />
        </template>
        Adds <code class="font-mono text-text-muted">--model</code>. Each name is Claude Code’s latest model of that
        family; Account default leaves your own setting alone.
      </ProviderField>

      <ProviderField label="Reasoning effort" field-id="claude-effort">
        <template #control="{ describedBy }">
          <AppSelect
            id="claude-effort"
            :model-value="effort"
            :options="effortChoices"
            :described-by="describedBy"
            data-testid="claude-effort"
            @update:model-value="chooseEffort"
          />
        </template>
        Adds <code class="font-mono text-text-muted">--effort</code>. A higher level thinks longer and spends more, on
        models that support extended thinking.
      </ProviderField>

      <SessionPermissionSwitch
        label="Open sessions without permission prompts"
        :model-value="skipPermissions"
        data-testid="claude-skip-permissions"
        @update:model-value="chooseSkipPermissions"
      >
        Adds <code class="font-mono text-text-muted">--dangerously-skip-permissions</code>. The session edits, runs and
        deletes without asking first.
      </SessionPermissionSwitch>

      <p class="border-t border-border px-5 py-3 text-small leading-relaxed text-text-subtle">
        The run queue has its own Claude Code model, effort and permission setting, in its panel.
      </p>
    </template>

    <div v-else class="flex flex-col items-start gap-3 px-5 py-3.5 text-small text-text-subtle">
      <p>Couldn't read the Claude Code configuration.</p>
      <AppButton size="sm" variant="secondary" @click="load">Retry</AppButton>
    </div>
  </ProviderPanel>
</template>
