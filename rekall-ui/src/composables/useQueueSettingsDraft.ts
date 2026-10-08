import { onUnmounted, ref, watch, type Ref } from 'vue'
import { DEFAULT_CEILING_PERCENT, type RunQueue, type RunQueueSettings } from '@/model/runQueue'

const SETTINGS_DEBOUNCE_MS = 350

export interface QueueSettingsDraft {
  readonly ceilingOn: Ref<boolean>
  readonly ceiling: Ref<number>
  readonly skipPermissions: Ref<boolean>
  readonly model: Ref<string>
  readonly effort: Ref<string>
  /** Waits for a save already on its way, or saves now: what a start must do before it starts. */
  readonly settle: () => Promise<void>
}

/**
 * The run queue's settings as the panel edits them. Each change saves on its own after a short
 * pause, so they apply to the next session even while a queue runs; closing the panel flushes one
 * still waiting. A failed save is reported, never thrown at the control that caused it.
 */
export function useQueueSettingsDraft(
  queue: RunQueue,
  save: (settings: RunQueueSettings) => Promise<void>,
  report: (error: unknown) => void
): QueueSettingsDraft {
  const ceilingOn = ref(queue.ceilingPercent !== null)
  const ceiling = ref(queue.ceilingPercent ?? DEFAULT_CEILING_PERCENT)
  const skipPermissions = ref(queue.skipPermissions)
  const model = ref<string>(queue.model ?? 'default')
  const effort = ref<string>(queue.effort ?? 'default')

  let timer: ReturnType<typeof setTimeout> | undefined
  let pending: Promise<void> | null = null

  function saveNow(): Promise<void> {
    clearTimeout(timer)
    timer = undefined
    pending = save({
      ceilingPercent: ceilingOn.value ? ceiling.value : null,
      skipPermissions: skipPermissions.value,
      model: model.value === 'default' ? null : model.value,
      effort: effort.value === 'default' ? null : effort.value
    })
      .catch(report)
      .finally(() => (pending = null))
    return pending
  }

  watch([ceilingOn, ceiling, skipPermissions, model, effort], () => {
    clearTimeout(timer)
    timer = setTimeout(() => void saveNow(), SETTINGS_DEBOUNCE_MS)
  })

  onUnmounted(() => {
    if (timer) void saveNow()
  })

  return {
    ceilingOn,
    ceiling,
    skipPermissions,
    model,
    effort,
    settle: () => pending ?? saveNow()
  }
}
