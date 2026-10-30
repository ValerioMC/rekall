import { apiClient, request } from '../../../api/client'
import { TelemetrySettingsSchema } from './schemas/telemetry.schema'
import type { TelemetryChange, TelemetrySettings } from '@/features/settings/model/telemetry'

export async function fetchTelemetrySettings(): Promise<TelemetrySettings> {
  return request(async () => TelemetrySettingsSchema.parse(await apiClient('/api/telemetry/settings')))
}

export async function saveTelemetrySettings(change: TelemetryChange): Promise<TelemetrySettings> {
  return request(async () =>
    TelemetrySettingsSchema.parse(await apiClient('/api/telemetry/settings', { method: 'PUT', body: change }))
  )
}
