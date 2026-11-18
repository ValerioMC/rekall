import { onMounted, ref } from 'vue'
import { fetchEpicConversion } from '@/features/catalog/api/epic-conversion.api'
import { conversionLosses, type ConversionLoss, type EpicConversion } from '@/features/catalog/model/epic-conversion'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useConsoleStore } from '@/stores/console.store'
import type { TaskId } from '@/model/branded'

/** Reads what turning the task into an epic would delete when the dialog opens, and does the conversion. */
export function useEpicConversion(taskId: TaskId, onLoadFailed: () => void) {
  const store = useConsoleStore()
  const { run, isRunning: isConverting } = useAsyncAction()
  const conversion = ref<EpicConversion | null>(null)
  const losses = ref<readonly ConversionLoss[]>([])

  onMounted(async () => {
    const loaded = await run(() => fetchEpicConversion(taskId))
    if (!loaded) return onLoadFailed()
    conversion.value = loaded
    losses.value = conversionLosses(loaded)
  })

  async function convert(message: string): Promise<boolean> {
    const converted = await run(async () => {
      await store.convertTaskToEpic(taskId)
      return true
    }, message)
    return converted === true
  }

  return { conversion, losses, isConverting, convert }
}
