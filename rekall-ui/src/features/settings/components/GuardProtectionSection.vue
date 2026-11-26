<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import SessionProviderMark from '@/features/claude/components/SessionProviderMark.vue'
import { SESSION_PROVIDER_LABEL, SESSION_PROVIDERS } from '@/features/claude/model/session-provider'
import type { SessionProvider } from '@/features/claude/model/session-provider'
import type { GuardProtectionState } from '@/features/settings/model/guardProtection'
import { useGuardProtectionStore } from '@/features/settings/store/guardProtection.store'
import { useToastStore } from '@/stores/toast.store'
import { onMounted, reactive } from 'vue'

interface Presentation {
  readonly tone: 'neutral' | 'safe' | 'warn'
  readonly label: string
  readonly action?: 'Install' | 'Repair' | 'Refresh'
}

const PRESENTATION: Readonly<Record<GuardProtectionState, Presentation>> = {
  INSTALLED: { tone: 'safe', label: 'Installed' },
  NOT_INSTALLED: { tone: 'neutral', label: 'Not installed', action: 'Install' },
  NEEDS_REPAIR: { tone: 'warn', label: 'Needs repair', action: 'Repair' },
  INVALID_SETTINGS: { tone: 'warn', label: 'Invalid settings', action: 'Refresh' },
  UNSUPPORTED: { tone: 'warn', label: 'Unavailable', action: 'Refresh' },
  TRUST_REQUIRED: { tone: 'warn', label: 'Trust required', action: 'Refresh' },
  DISABLED: { tone: 'warn', label: 'Disabled in Codex', action: 'Refresh' }
}

const store = useGuardProtectionStore()
const toast = useToastStore()
const pending = reactive<Partial<Record<SessionProvider, 'refresh' | 'install' | 'remove'>>>({})

function presentation(provider: SessionProvider): Presentation {
  const state = store.byProvider[provider]?.state
  return state ? PRESENTATION[state] : { tone: 'neutral', label: 'Couldn’t read status', action: 'Refresh' }
}

async function run(provider: SessionProvider, action: 'refresh' | 'install' | 'remove'): Promise<void> {
  if (pending[provider]) return
  pending[provider] = action
  try {
    if (action === 'install') await store.install(provider)
    else if (action === 'remove') await store.remove(provider)
    else await store.load(provider)
  } catch (caught) {
    toast.notifyError(caught)
  } finally {
    delete pending[provider]
  }
}

onMounted(() => {
  for (const provider of SESSION_PROVIDERS) void run(provider, 'refresh')
})
</script>

<template>
  <section aria-labelledby="guard-heading" data-testid="guard-section">
    <h2 id="guard-heading" class="mb-3 eyebrow text-caption">Protections</h2>

    <ul class="divide-y divide-border rounded-control border border-border bg-canvas">
      <li
        v-for="provider in SESSION_PROVIDERS"
        :key="provider"
        class="flex flex-wrap items-start justify-between gap-x-4 gap-y-2 px-3.5 py-3"
        :data-testid="`guard-provider-${provider}`"
      >
        <div class="min-w-0 flex-1">
          <div class="flex flex-wrap items-center gap-x-2 gap-y-1">
            <SessionProviderMark :provider="provider" />
            <h3 class="text-small font-semibold text-text">{{ SESSION_PROVIDER_LABEL[provider] }}</h3>
            <span role="status">
              <span
                v-if="pending[provider] === 'refresh' && !store.byProvider[provider]"
                class="text-caption text-text-subtle"
              >
                Checking…
              </span>
              <AppBadge v-else :tone="presentation(provider).tone" dot data-testid="guard-status">
                {{ presentation(provider).label }}
              </AppBadge>
            </span>
          </div>
          <p
            v-if="store.byProvider[provider]?.problem"
            class="mt-2 text-small leading-relaxed text-warn"
            data-testid="guard-problem"
          >
            {{ store.byProvider[provider]?.problem }}
          </p>
          <p
            v-if="store.byProvider[provider]"
            class="mt-1 truncate font-mono text-caption text-text-subtle"
            :title="store.byProvider[provider]?.settingsFile"
          >
            {{ store.byProvider[provider]?.settingsFile }}
          </p>
        </div>
        <div class="flex flex-wrap gap-2">
          <AppButton
            v-if="store.byProvider[provider]?.configured"
            size="sm"
            variant="quiet"
            :loading="pending[provider] === 'remove'"
            :disabled="Boolean(pending[provider])"
            :aria-label="`Remove ${SESSION_PROVIDER_LABEL[provider]} protections`"
            data-testid="guard-remove"
            @click="run(provider, 'remove')"
          >
            Remove
          </AppButton>
          <AppButton
            v-if="presentation(provider).action && (store.byProvider[provider] || !pending[provider])"
            size="sm"
            :loading="Boolean(pending[provider] && pending[provider] !== 'remove')"
            :disabled="Boolean(pending[provider])"
            :aria-label="`${presentation(provider).action} ${SESSION_PROVIDER_LABEL[provider]} protections`"
            data-testid="guard-action"
            @click="run(provider, presentation(provider).action === 'Refresh' ? 'refresh' : 'install')"
          >
            {{ presentation(provider).action }}
          </AppButton>
        </div>
      </li>
    </ul>

    <ul class="mt-3 space-y-1 text-caption leading-relaxed text-text-subtle" data-testid="guard-rules">
      <li>Blocks manual Git commits, pushes and staging on projects where Rekall commits for you.</li>
      <li>Blocks supported tool calls that name protected credential locations.</li>
      <li>Asks the session once to claim its running step before stopping.</li>
    </ul>
  </section>
</template>
