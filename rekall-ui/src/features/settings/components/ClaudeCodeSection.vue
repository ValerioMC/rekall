<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import CopyGlyph from '@/components/ui/CopyGlyph.vue'
import AppButton from '@/components/ui/AppButton.vue'
import SessionChoiceGroup from '@/features/settings/components/SessionChoiceGroup.vue'
import SessionPermissionSwitch from '@/features/settings/components/SessionPermissionSwitch.vue'
import { storeToRefs } from 'pinia'
import { useClaudeConnectionStore } from '@/features/claude/store/claudeConnection.store'
import { useClaudeCommandUpdateStore } from '@/features/claude/store/claudeCommandUpdate.store'
import CommandUpdateMark from '@/features/claude/components/CommandUpdateMark.vue'
import { useToastStore } from '@/stores/toast.store'
import {
  preferredEffort,
  preferredModel,
  preferredSessionKind,
  setPreferredEffort,
  setPreferredModel,
  setPreferredSessionKind,
  setSkipsPermissions,
  skipsPermissions
} from '@/common/config/claude-launch'
import {
  CLAUDE_EFFORT_CHOICES,
  CLAUDE_MODEL_CHOICES,
  CLAUDE_SESSION_VIEWS,
  claudeEffortChoiceLabel,
  claudeModelChoiceLabel,
  claudeSessionViewLabel
} from '@/features/claude/model/claude'
import type {
  ClaudeConnectionStatus,
  ClaudeEffortChoice,
  ClaudeModelChoice,
  ClaudeSessionView
} from '@/features/claude/model/claude'

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
  NOT_CONNECTED: { tone: 'neutral', label: 'Not connected', action: 'Connect', variant: 'primary' },
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

const sessionViews = CLAUDE_SESSION_VIEWS.map((value) => ({ value, label: claudeSessionViewLabel(value) }))
const sessionView = ref<ClaudeSessionView>(preferredSessionKind())

function chooseSessionView(choice: ClaudeSessionView): void {
  sessionView.value = choice
  setPreferredSessionKind(choice)
}

const modelChoices = CLAUDE_MODEL_CHOICES.map((value) => ({ value, label: claudeModelChoiceLabel(value) }))
const model = ref<ClaudeModelChoice>(preferredModel())

function chooseModel(choice: ClaudeModelChoice): void {
  model.value = choice
  setPreferredModel(choice)
}

const effortChoices = CLAUDE_EFFORT_CHOICES.map((value) => ({ value, label: claudeEffortChoiceLabel(value) }))
const effort = ref<ClaudeEffortChoice>(preferredEffort())

function chooseEffort(choice: ClaudeEffortChoice): void {
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
  <section data-testid="claude-section">
    <p class="mb-3 eyebrow text-caption">Claude Code</p>

    <div v-if="loading" class="skeleton h-20 rounded-control" aria-hidden="true" />

    <div
      v-else-if="installation"
      class="rounded-control border border-border bg-canvas px-3.5 py-3"
    >
      <div class="flex items-start justify-between gap-3">
        <div class="min-w-0 flex-1">
          <AppBadge v-if="updatingCommands" tone="accent" data-testid="claude-status">
            <CommandUpdateMark />
          </AppBadge>
          <AppBadge v-else :tone="presentation!.tone" dot data-testid="claude-status">
            {{ presentation!.label }}
          </AppBadge>
          <p class="mt-1.5 text-small leading-relaxed text-text-muted">{{ explanation }}</p>
          <p class="mt-1 truncate font-mono text-caption text-text-subtle" :title="installation.endpoint">
            {{ installation.endpoint }}
          </p>
        </div>
        <AppButton
          v-if="presentation!.action && !updatingCommands"
          size="sm"
          :variant="presentation!.variant"
          :loading="installing"
          data-testid="claude-install"
          @click="install"
        >
          {{ presentation!.action }}
        </AppButton>
      </div>

      <div
        v-if="installation.status === 'CLI_MISSING'"
        class="mt-2.5 flex items-center gap-2 rounded-control border border-border bg-surface-raised px-2.5 py-2"
      >
        <code
          class="min-w-0 flex-1 truncate font-mono text-caption text-text"
          :title="installation.manualCommand"
        >
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

      <SessionPermissionSwitch
        label="Open sessions without permission prompts"
        :model-value="skipPermissions"
        data-testid="claude-skip-permissions"
        @update:model-value="chooseSkipPermissions"
      >
        Adds <code class="text-anchor/80">--dangerously-skip-permissions</code> to every session Rekall
        starts, <span class="text-text-muted">Run here</span> or
        <span class="text-text-muted">Open in terminal</span>. That session edits, runs and deletes
        without asking first.
      </SessionPermissionSwitch>

      <div class="mt-3 border-t border-border pt-3" data-testid="claude-session-view">
        <p class="text-small text-text">Session view</p>
        <p class="mt-0.5 text-caption leading-relaxed text-text-subtle">
          <span class="text-text-muted">Chat</span> shows the selected provider’s conversation and approval requests
          in a window beside the task.
          <span class="text-text-muted">Terminal</span> stays the default.
        </p>
        <SessionChoiceGroup
          label="Session view for a Run here session"
          :options="sessionViews"
          :model-value="sessionView"
          test-id-prefix="claude-session-view"
          @update:model-value="chooseSessionView"
        />
      </div>

      <div class="mt-3 border-t border-border pt-3" data-testid="claude-model">
        <p class="text-small text-text">Model for a “Run here” session</p>
        <p class="mt-0.5 text-caption leading-relaxed text-text-subtle">
          Adds <code class="text-anchor/80">--model</code> to what an in-app session launches; each alias is
          Claude Code's name for the latest model of that family, so nothing is pinned to a version.
          <span class="text-text-muted">Account default</span> leaves your Claude Code setting alone. A
          session already running keeps the model it started with.
        </p>
        <SessionChoiceGroup
          label="Model for a Run here session"
          :options="modelChoices"
          :model-value="model"
          test-id-prefix="claude-model"
          @update:model-value="chooseModel"
        />
      </div>

      <div class="mt-3 border-t border-border pt-3" data-testid="claude-effort">
        <p class="text-small text-text">Reasoning effort</p>
        <p class="mt-0.5 text-caption leading-relaxed text-text-subtle">
          Adds <code class="text-anchor/80">--effort</code> to a new session. A higher level lets the model
          think longer on hard problems and spends more; it applies to models that support extended thinking.
          <span class="text-text-muted">Account default</span> leaves it unset.
        </p>
        <SessionChoiceGroup
          label="Reasoning effort for a Run here session"
          :options="effortChoices"
          :model-value="effort"
          test-id-prefix="claude-effort"
          @update:model-value="chooseEffort"
        />
      </div>

      <p v-if="installed" class="mt-2 text-small text-text-muted" data-testid="claude-restart-hint">
        Sessions already open keep the configuration they started with. This one needs restarting.
      </p>
    </div>

    <div v-else class="flex flex-col items-start gap-3 text-small text-text-subtle">
      <p>Couldn't read the Claude Code configuration.</p>
      <AppButton size="sm" variant="secondary" @click="load">Retry</AppButton>
    </div>
  </section>
</template>
