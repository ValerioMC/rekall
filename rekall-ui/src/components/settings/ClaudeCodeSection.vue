<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import { fetchClaudeInstallation, installClaudeIntegration } from '@/api/claude.api'
import { useToastStore } from '@/stores/toast.store'
import { canLaunchClaudeCode } from '@/common/native/desktop'
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
} from '@/model/claude'
import type {
  ClaudeConnectionStatus,
  ClaudeEffortChoice,
  ClaudeInstallation,
  ClaudeModelChoice
} from '@/model/claude'

const toast = useToastStore()

const installation = ref<ClaudeInstallation | null>(null)
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
    installation.value = await fetchClaudeInstallation()
  } catch (caught) {
    toast.notifyError(caught)
  } finally {
    loading.value = false
  }
}

async function install(): Promise<void> {
  installing.value = true
  try {
    installation.value = await installClaudeIntegration()
    installed.value = true
    toast.notify('Registered. A session started from now on will find it.')
  } catch (caught) {
    toast.notifyError(caught)
  } finally {
    installing.value = false
  }
}

const canLaunch = canLaunchClaudeCode()
const skipPermissions = ref(skipsPermissions())

function toggleSkipPermissions(): void {
  skipPermissions.value = !skipPermissions.value
  setSkipsPermissions(skipPermissions.value)
}

const modelChoices = CLAUDE_MODEL_CHOICES
const model = ref<ClaudeModelChoice>(preferredModel())

function chooseModel(choice: ClaudeModelChoice): void {
  model.value = choice
  setPreferredModel(choice)
}

const effortChoices = CLAUDE_EFFORT_CHOICES
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
    <p class="mb-3 eyebrow text-[11px]">Claude Code</p>

    <div v-if="loading" class="skeleton h-20 rounded-[var(--radius-control)]" aria-hidden="true" />

    <div
      v-else-if="installation"
      class="rounded-[var(--radius-control)] border border-border bg-canvas px-3.5 py-3"
    >
      <div class="flex items-start justify-between gap-3">
        <div class="min-w-0 flex-1">
          <AppBadge :tone="presentation!.tone" dot data-testid="claude-status">
            {{ presentation!.label }}
          </AppBadge>
          <p class="mt-1.5 text-[12.5px] leading-relaxed text-text-muted">{{ explanation }}</p>
          <p class="mt-1 truncate font-mono text-[11.5px] text-text-subtle" :title="installation.endpoint">
            {{ installation.endpoint }}
          </p>
        </div>
        <AppButton
          v-if="presentation!.action"
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
        class="mt-2.5 flex items-center gap-2 rounded-md border border-border bg-surface-raised px-2.5 py-2"
      >
        <code class="min-w-0 flex-1 truncate font-mono text-[11.5px] text-text" :title="installation.manualCommand">
          {{ installation.manualCommand }}
        </code>
        <AppButton size="sm" variant="ghost" data-testid="claude-copy" @click="copyCommand">
          {{ copied ? 'Copied' : 'Copy' }}
        </AppButton>
      </div>

      <div
        v-if="canLaunch"
        class="mt-3 flex items-start gap-3 border-t border-border pt-3"
        data-testid="claude-skip-permissions"
      >
        <div class="min-w-0 flex-1">
          <p class="text-[12.5px] text-text">Open sessions without permission prompts</p>
          <p class="mt-0.5 text-[11.5px] leading-relaxed text-text-subtle">
            Adds <code class="text-anchor/80">--dangerously-skip-permissions</code> to what
            <span class="text-text-muted">Open in Claude Code</span> launches. That session edits,
            runs and deletes without asking first.
          </p>
        </div>
        <button
          type="button"
          role="switch"
          :aria-checked="skipPermissions"
          aria-label="Open sessions without permission prompts"
          class="focus-ring mt-0.5 h-5 w-9 shrink-0 rounded-full border transition-colors"
          :class="skipPermissions ? 'border-danger bg-danger-soft' : 'border-border bg-surface-raised'"
          @click="toggleSkipPermissions"
        >
          <span
            class="block size-3.5 rounded-full transition-transform"
            :class="skipPermissions ? 'translate-x-[18px] bg-danger' : 'translate-x-[2px] bg-text-subtle'"
          />
        </button>
      </div>

      <div class="mt-3 border-t border-border pt-3" data-testid="claude-model">
        <p class="text-[12.5px] text-text">Model for a “Run here” session</p>
        <p class="mt-0.5 text-[11.5px] leading-relaxed text-text-subtle">
          Adds <code class="text-anchor/80">--model</code> to what an in-app session launches; each
          alias is Claude Code's name for the latest model of that family, so nothing is pinned to a
          version. <span class="text-text-muted">Account default</span> leaves your Claude Code
          setting alone. A session already running keeps the model it started with.
        </p>
        <div class="mt-2 flex flex-wrap gap-1.5" role="radiogroup" aria-label="Model for a Run here session">
          <button
            v-for="choice in modelChoices"
            :key="choice"
            type="button"
            role="radio"
            :aria-checked="model === choice"
            class="focus-ring inline-flex items-center rounded-full border px-2.5 py-1 text-[11.5px] transition-colors"
            :class="
              model === choice
                ? 'border-accent bg-accent-soft text-accent'
                : 'border-border text-text-subtle hover:border-border-strong hover:text-text-muted'
            "
            :data-testid="`claude-model-${choice}`"
            @click="chooseModel(choice)"
          >
            {{ claudeModelChoiceLabel(choice) }}
          </button>
        </div>
      </div>

      <div class="mt-3 border-t border-border pt-3" data-testid="claude-effort">
        <p class="text-[12.5px] text-text">Reasoning effort</p>
        <p class="mt-0.5 text-[11.5px] leading-relaxed text-text-subtle">
          Adds <code class="text-anchor/80">--effort</code> to a new session. A higher level lets the
          model think longer on hard problems and spends more; it applies to models that support
          extended thinking. <span class="text-text-muted">Account default</span> leaves it unset.
        </p>
        <div class="mt-2 flex flex-wrap gap-1.5" role="radiogroup" aria-label="Reasoning effort for a Run here session">
          <button
            v-for="choice in effortChoices"
            :key="choice"
            type="button"
            role="radio"
            :aria-checked="effort === choice"
            class="focus-ring inline-flex items-center rounded-full border px-2.5 py-1 text-[11.5px] transition-colors"
            :class="
              effort === choice
                ? 'border-accent bg-accent-soft text-accent'
                : 'border-border text-text-subtle hover:border-border-strong hover:text-text-muted'
            "
            :data-testid="`claude-effort-${choice}`"
            @click="chooseEffort(choice)"
          >
            {{ claudeEffortChoiceLabel(choice) }}
          </button>
        </div>
      </div>

      <p v-if="installed" class="mt-2 text-[12px] text-text-muted" data-testid="claude-restart-hint">
        Sessions already open keep the configuration they started with. This one needs restarting.
      </p>
    </div>

    <div v-else class="flex flex-col items-start gap-3 text-[12.5px] text-text-subtle">
      <p>Couldn't read the Claude Code configuration.</p>
      <AppButton size="sm" variant="secondary" @click="load">Retry</AppButton>
    </div>
  </section>
</template>
