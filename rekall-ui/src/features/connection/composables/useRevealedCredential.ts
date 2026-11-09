import { onUnmounted, ref, watch, type Ref } from 'vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import type { Connection, RevealedCredential } from '@/features/connection/model/connection'

const COPIED_FLASH_MS = 1400

export type CopiedPart = 'secret' | 'email'

/**
 * Reads a connection's stored credential only when the person shows or copies it, and drops it
 * again when it is hidden, when the credential is replaced or removed, and when the view goes.
 */
export function useRevealedCredential(connection: Ref<Connection>) {
  const store = useConnectionsStore()
  const { run, isRunning: loading } = useAsyncAction()
  const revealed = ref<RevealedCredential | null>(null)
  const visible = ref(false)
  const copied = ref<CopiedPart | null>(null)
  let flashTimer: ReturnType<typeof setTimeout> | null = null

  function drop(): void {
    revealed.value = null
    visible.value = false
  }

  async function read(): Promise<RevealedCredential | null> {
    return revealed.value ?? (await run(() => store.revealCredential(connection.value.id)))
  }

  async function toggle(): Promise<void> {
    if (visible.value) return drop()
    const credential = await read()
    if (!credential) return
    revealed.value = credential
    visible.value = true
  }

  async function copy(part: CopiedPart): Promise<void> {
    const credential = await read()
    if (!credential) return
    const text = part === 'email' ? credential.email : (credential.token ?? credential.kubeconfig)
    if (text === null) return
    await navigator.clipboard?.writeText(text)
    flash(part)
  }

  function flash(part: CopiedPart): void {
    copied.value = part
    if (flashTimer) clearTimeout(flashTimer)
    flashTimer = setTimeout(() => (copied.value = null), COPIED_FLASH_MS)
  }

  watch(() => [connection.value.id, connection.value.credentialSetAt], drop)

  onUnmounted(() => {
    drop()
    if (flashTimer) clearTimeout(flashTimer)
  })

  return { revealed, visible, loading, copied, toggle, copy }
}
