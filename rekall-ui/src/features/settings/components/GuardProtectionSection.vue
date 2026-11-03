<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { storeToRefs } from 'pinia'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import { useGuardProtectionStore } from '@/features/settings/store/guardProtection.store'
import type { GuardProtectionState } from '@/features/settings/model/guardProtection'
import { useToastStore } from '@/stores/toast.store'

const toast = useToastStore()
const store = useGuardProtectionStore()
const { protection } = storeToRefs(store)
const loading = ref(true)
const working = ref(false)

interface Presentation {
  readonly tone: 'neutral' | 'safe' | 'warn'
  readonly label: string
}

const PRESENTATION: Readonly<Record<GuardProtectionState, Presentation>> = {
  INSTALLED: { tone: 'safe', label: 'Installed' },
  NOT_INSTALLED: { tone: 'neutral', label: 'Not installed' },
  INVALID_SETTINGS: { tone: 'warn', label: 'Settings file not valid' }
}

const RULES: readonly string[] = [
  'On a project where Rekall commits for you, git commit, push and add are refused.',
  "Reading, copying or searching the files where credentials are kept is refused.",
  'A session cannot end while a step is running and was never claimed.'
]

const presentation = computed<Presentation | null>(() =>
  protection.value ? PRESENTATION[protection.value.state] : null
)

async function load(): Promise<void> {
  loading.value = true
  try {
    await store.load()
  } catch (caught) {
    toast.notifyError(caught)
  } finally {
    loading.value = false
  }
}

async function change(action: () => Promise<void>, done: string): Promise<void> {
  working.value = true
  try {
    await action()
    toast.notify(done)
  } catch (caught) {
    toast.notifyError(caught)
  } finally {
    working.value = false
  }
}

const install = (): Promise<void> =>
  change(() => store.install(), 'Protections installed. A session started from now on is covered.')
const remove = (): Promise<void> => change(() => store.remove(), 'Protections removed.')

onMounted(() => void load())
</script>

<template>
  <section data-testid="guard-section">
    <p class="mb-3 eyebrow text-caption">Protections</p>

    <div v-if="loading" class="skeleton h-20 rounded-control" aria-hidden="true" />

    <div v-else-if="protection" class="rounded-control border border-border bg-canvas px-3.5 py-3">
      <div class="flex items-start justify-between gap-3">
        <div class="min-w-0 flex-1">
          <AppBadge :tone="presentation!.tone" dot data-testid="guard-status">
            {{ presentation!.label }}
          </AppBadge>
          <p class="mt-1.5 text-small leading-relaxed text-text-muted">
            A hook is a small program Claude Code runs by itself, outside the model, just before a tool runs
            and just before a session ends. If it says no, the action does not happen and the model is told
            why. A rule written in a prompt can be forgotten; a hook cannot be skipped.
          </p>
          <p
            v-if="protection.problem"
            class="mt-1.5 text-small leading-relaxed text-warn"
            data-testid="guard-problem"
          >
            {{ protection.problem }}
          </p>
          <p class="mt-1 truncate font-mono text-caption text-text-subtle" :title="protection.settingsFile">
            {{ protection.settingsFile }}
          </p>
        </div>
        <div class="flex shrink-0 gap-2">
          <AppButton
            v-if="protection.state === 'INSTALLED'"
            size="sm"
            variant="secondary"
            :loading="working"
            data-testid="guard-remove"
            @click="remove"
          >
            Remove protections
          </AppButton>
          <AppButton
            v-if="protection.state !== 'INVALID_SETTINGS'"
            size="sm"
            variant="primary"
            :loading="working"
            data-testid="guard-install"
            @click="install"
          >
            {{ protection.state === 'INSTALLED' ? 'Reinstall' : 'Install protections' }}
          </AppButton>
        </div>
      </div>

      <ul class="mt-3 space-y-1 border-t border-border pt-3" data-testid="guard-rules">
        <li v-for="rule in RULES" :key="rule" class="text-caption leading-relaxed text-text-subtle">
          {{ rule }}
        </li>
      </ul>
    </div>

    <div v-else class="flex flex-col items-start gap-3 text-small text-text-subtle">
      <p>Couldn't read the protections.</p>
      <AppButton size="sm" variant="secondary" @click="load">Retry</AppButton>
    </div>
  </section>
</template>
