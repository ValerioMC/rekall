<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import { fetchClaudeInstallation, installClaudeIntegration } from '@/api/claude.api'
import { useToastStore } from '@/stores/toast.store'
import { canLaunchClaudeCode } from '@/common/native/desktop'
import { setSkipsPermissions, skipsPermissions } from '@/common/config/claude-launch'
import type { ClaudeConnectionStatus, ClaudeInstallation } from '@/model/claude'

/**
 * The Claude Code registration, as one line of state and one button.
 *
 * <p>Everything this section can do is the same action — write the registration from scratch —
 * so there is one button whatever state it finds, and the state decides what the button is
 * called and whether it is worth the accent. A machine without the `claude` binary is the only
 * case with nothing to press: there the command itself is the content.
 */
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

/** One sentence saying what the state means, never a restatement of the badge above it. */
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

/**
 * Whether a session opened from a button runs without asking.
 *
 * <p>Only shown where the button exists, which is the application window: offered in a browser
 * tab it would be a switch with nothing behind it. Off until it is turned on, and the sentence
 * under it says what it costs rather than what it saves.
 */
const canLaunch = canLaunchClaudeCode()
const skipPermissions = ref(skipsPermissions())

function toggleSkipPermissions(): void {
  skipPermissions.value = !skipPermissions.value
  setSkipsPermissions(skipPermissions.value)
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
    <p class="mb-3 text-[11px] font-semibold uppercase tracking-[0.09em] text-text-subtle">Claude Code</p>

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
