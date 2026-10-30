import { defineStore } from 'pinia'
import { ref } from 'vue'
import { fetchTelemetrySettings, saveTelemetrySettings } from '@/features/settings/api/telemetry.api'
import type { TelemetryChange, TelemetrySettings } from '@/features/settings/model/telemetry'

/** Where Claude Code's traces are sent, as the server stores it. */
export const useTelemetrySettingsStore = defineStore('telemetrySettings', () => {
  const settings = ref<TelemetrySettings | null>(null)

  async function load(): Promise<void> {
    settings.value = await fetchTelemetrySettings()
  }

  async function save(change: TelemetryChange): Promise<void> {
    settings.value = await saveTelemetrySettings(change)
  }

  return { settings, load, save }
})
