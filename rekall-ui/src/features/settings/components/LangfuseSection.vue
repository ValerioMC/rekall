<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppField from '@/components/ui/AppField.vue'
import AppInput from '@/components/ui/AppInput.vue'
import { storeToRefs } from 'pinia'
import { useTelemetrySettingsStore } from '@/features/settings/store/telemetrySettings.store'
import { useToastStore } from '@/stores/toast.store'
import type { TelemetryChange, TelemetrySettings } from '@/features/settings/model/telemetry'

const toast = useToastStore()
const store = useTelemetrySettingsStore()
const { settings } = storeToRefs(store)

const loading = ref(true)
const saving = ref(false)
const enabled = ref(false)
const host = ref('')
const publicKey = ref('')
const secretKey = ref('')
const replacingSecret = ref(false)

const showsSecretField = computed<boolean>(() => !settings.value?.secretSaved || replacingSecret.value)

function fillForm(current: TelemetrySettings): void {
  enabled.value = current.enabled
  host.value = current.host
  publicKey.value = current.publicKey
  secretKey.value = ''
  replacingSecret.value = false
}

function buildChange(current: TelemetrySettings): TelemetryChange {
  const typedSecret = secretKey.value.trim()
  return {
    enabled: enabled.value,
    host: host.value,
    publicKey: publicKey.value,
    logContent: current.logContent,
    ...(typedSecret ? { secretKey: typedSecret } : {})
  }
}

async function load(): Promise<void> {
  loading.value = true
  try {
    await store.load()
    if (settings.value) fillForm(settings.value)
  } catch (caught) {
    toast.notifyError(caught)
  } finally {
    loading.value = false
  }
}

async function save(): Promise<void> {
  if (!settings.value) return
  saving.value = true
  try {
    await store.save(buildChange(settings.value))
    if (settings.value) fillForm(settings.value)
    toast.notify('Saved. Terminals opened from now on use it.')
  } catch (caught) {
    toast.notifyError(caught)
  } finally {
    saving.value = false
  }
}

onMounted(() => void load())
</script>

<template>
  <section data-testid="langfuse-section">
    <p class="mb-3 eyebrow text-caption">Langfuse</p>

    <div v-if="loading" class="skeleton h-20 rounded-control" aria-hidden="true" />

    <form
      v-else-if="settings"
      class="rounded-control border border-border bg-canvas px-3.5 py-3"
      @submit.prevent="save"
    >
      <div class="flex items-start gap-3">
        <div class="min-w-0 flex-1">
          <p class="text-small text-text">Send Claude Code traces to Langfuse</p>
          <p class="mt-0.5 text-caption leading-relaxed text-text-subtle">
            Sessions Rekall opens report to your own Langfuse, one trace per prompt, tagged with the task. It applies
            to terminals opened after saving; a terminal already open keeps what it started with. Rekall never
            contacts Langfuse itself, so having it down changes nothing.
          </p>
        </div>
        <button
          type="button"
          role="switch"
          :aria-checked="enabled"
          aria-label="Send Claude Code traces to Langfuse"
          class="focus-ring mt-0.5 h-5 w-9 shrink-0 rounded-full border transition-colors"
          :class="enabled ? 'border-accent bg-accent-soft' : 'border-border bg-surface-raised'"
          data-testid="langfuse-enabled"
          @click="enabled = !enabled"
        >
          <span
            class="block size-3.5 rounded-full transition-transform"
            :class="enabled ? 'translate-x-[18px] bg-accent' : 'translate-x-[2px] bg-text-subtle'"
          />
        </button>
      </div>

      <div class="mt-3 border-t border-border pt-3">
        <AppField label="Host">
          <template #default="{ fieldId }">
            <AppInput
              :id="fieldId"
              v-model="host"
              mono
              placeholder="http://localhost:3000"
              data-testid="langfuse-host"
            />
          </template>
        </AppField>

        <AppField label="Public key">
          <template #default="{ fieldId }">
            <AppInput :id="fieldId" v-model="publicKey" mono placeholder="pk-lf-…" data-testid="langfuse-public-key" />
          </template>
        </AppField>

        <AppField label="Secret key">
          <template #default="{ fieldId }">
            <div v-if="!showsSecretField" class="flex items-center gap-2">
              <span class="text-small text-text-muted" data-testid="langfuse-secret-saved">set</span>
              <AppButton
                size="sm"
                variant="ghost"
                type="button"
                data-testid="langfuse-replace-secret"
                @click="replacingSecret = true"
              >
                Replace
              </AppButton>
            </div>
            <AppInput
              v-else
              :id="fieldId"
              v-model="secretKey"
              type="password"
              mono
              placeholder="sk-lf-…"
              data-testid="langfuse-secret-key"
            />
          </template>
        </AppField>

        <div class="flex justify-end">
          <AppButton size="sm" type="submit" :loading="saving" data-testid="langfuse-save">Save</AppButton>
        </div>
      </div>
    </form>

    <div v-else class="flex flex-col items-start gap-3 text-small text-text-subtle">
      <p>Couldn't read the Langfuse setting.</p>
      <AppButton size="sm" variant="secondary" @click="load">Retry</AppButton>
    </div>
  </section>
</template>
