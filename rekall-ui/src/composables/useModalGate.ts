import { ref } from 'vue'

/**
 * Whether a blocking, app-wide modal is currently open, shared across every consumer.
 *
 * `App.vue`'s single-key console shortcuts ('n', 'j', 'k', 't', 'e', digits) check this first
 * and bail out entirely while it is true, rather than relying on a modal calling
 * `stopPropagation()` on every key: that approach also stops the event from ever reaching the
 * modal's own descendants — an Enter-to-save in one of its own text fields, for instance — since
 * a capture-phase `stopPropagation()` on `window` prevents the event from descending any
 * further at all, not only from reaching other listeners on `window` itself.
 */
const openCount = ref(0)

export function useModalGate() {
  return {
    isModalOpen: openCount,
    open: (): void => {
      openCount.value += 1
    },
    close: (): void => {
      openCount.value = Math.max(0, openCount.value - 1)
    }
  }
}
