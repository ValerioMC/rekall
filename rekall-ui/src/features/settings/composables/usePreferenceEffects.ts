import { watchEffect } from 'vue'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'

/** Puts the motion and text scale choices on the document, where the stylesheet reads them. */
export function usePreferenceEffects(): void {
  const preferences = usePreferencesStore()
  watchEffect(() => {
    const root = document.documentElement
    root.dataset.motion = preferences.motion
    root.style.fontSize = `${preferences.scale}%`
  })
}
