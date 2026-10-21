import { defineStore } from 'pinia'
import { ref } from 'vue'
import * as api from '@/features/connection/api/connections.api'
import type {
  Connection,
  ConnectionAccess,
  ConnectionCheck,
  ConnectionDetail,
  ConnectionInput,
  ConnectionRemoved,
  CredentialDraft,
  KubeContextOption
} from '@/features/connection/model/connection'
import type { CompanyId, ConnectionId } from '@/model/branded'

const ACCESS_PAGE = 50

/**
 * The connections of the companies the console has opened, with each one's projects, last check
 * and access log. A credential passes through on its way to the server and is never kept here.
 */
export const useConnectionsStore = defineStore('connections', () => {
  const byCompany = ref<Readonly<Record<string, readonly Connection[]>>>({})
  const details = ref<Readonly<Record<string, ConnectionDetail>>>({})
  const checks = ref<Readonly<Record<string, ConnectionCheck>>>({})
  const accessLogs = ref<Readonly<Record<string, readonly ConnectionAccess[]>>>({})
  const accessEnded = ref<ReadonlySet<string>>(new Set())
  const loading = ref<ReadonlySet<string>>(new Set())

  function connectionsOf(companyId: CompanyId): readonly Connection[] {
    return byCompany.value[companyId] ?? []
  }

  function isLoaded(companyId: CompanyId): boolean {
    return companyId in byCompany.value
  }

  function isLoading(companyId: CompanyId): boolean {
    return loading.value.has(companyId)
  }

  function detailOf(id: ConnectionId): ConnectionDetail | null {
    return details.value[id] ?? null
  }

  function checkOf(id: ConnectionId): ConnectionCheck | null {
    return checks.value[id] ?? null
  }

  function accessOf(id: ConnectionId): readonly ConnectionAccess[] {
    return accessLogs.value[id] ?? []
  }

  /** Whether an older page of the log may still be there to read. */
  function hasMoreAccess(id: ConnectionId): boolean {
    return id in accessLogs.value && !accessEnded.value.has(id)
  }

  async function load(companyId: CompanyId): Promise<void> {
    loading.value = new Set(loading.value).add(companyId)
    try {
      byCompany.value = { ...byCompany.value, [companyId]: await api.fetchCompanyConnections(companyId) }
    } finally {
      const rest = new Set(loading.value)
      rest.delete(companyId)
      loading.value = rest
    }
  }

  async function ensureLoaded(companyId: CompanyId): Promise<void> {
    if (!isLoaded(companyId) && !isLoading(companyId)) await load(companyId)
  }

  async function loadDetail(id: ConnectionId): Promise<ConnectionDetail> {
    const detail = await api.fetchConnection(id)
    details.value = { ...details.value, [id]: detail }
    return detail
  }

  /** Puts a connection the server answered in its company's list and in its detail, if open. */
  function replace(saved: Connection): void {
    const list = connectionsOf(saved.companyId)
    const known = list.some((connection) => connection.id === saved.id)
    byCompany.value = {
      ...byCompany.value,
      [saved.companyId]: known
        ? list.map((connection) => (connection.id === saved.id ? saved : connection))
        : [...list, saved].sort(byLabel)
    }
    const detail = details.value[saved.id]
    if (detail) details.value = { ...details.value, [saved.id]: { ...detail, ...saved } }
  }

  /** Creates the connection, then stores its credential when one was given. */
  async function create(
    companyId: CompanyId,
    input: ConnectionInput,
    credential: CredentialDraft | null
  ): Promise<Connection> {
    const created = await api.createConnection(companyId, input)
    replace(created)
    if (!credential) return created
    return await setCredential(created.id, credential)
  }

  async function update(id: ConnectionId, input: ConnectionInput): Promise<Connection> {
    const saved = await api.updateConnection(id, input)
    replace(saved)
    return saved
  }

  async function setCredential(id: ConnectionId, draft: CredentialDraft): Promise<Connection> {
    const saved = await api.setCredential(id, draft)
    replace(saved)
    forgetCheck(id)
    return saved
  }

  async function clearCredential(id: ConnectionId): Promise<Connection> {
    const saved = await api.clearCredential(id)
    replace(saved)
    forgetCheck(id)
    return saved
  }

  async function remove(connection: Connection): Promise<ConnectionRemoved> {
    const removed = await api.deleteConnection(connection.id)
    byCompany.value = {
      ...byCompany.value,
      [connection.companyId]: connectionsOf(connection.companyId).filter(
        (known) => known.id !== connection.id
      )
    }
    const rest = { ...details.value }
    delete rest[connection.id]
    details.value = rest
    forgetCheck(connection.id)
    return removed
  }

  async function check(id: ConnectionId): Promise<ConnectionCheck> {
    const result = await api.checkConnection(id)
    checks.value = { ...checks.value, [id]: result }
    return result
  }

  function forgetCheck(id: ConnectionId): void {
    const rest = { ...checks.value }
    delete rest[id]
    checks.value = rest
  }

  async function loadAccess(id: ConnectionId): Promise<void> {
    const page = await api.fetchConnectionAccess(id, 0, ACCESS_PAGE)
    accessLogs.value = { ...accessLogs.value, [id]: page }
    markEnded(id, page.length < ACCESS_PAGE)
  }

  async function loadMoreAccess(id: ConnectionId): Promise<void> {
    const known = accessOf(id)
    const page = await api.fetchConnectionAccess(id, known.length, ACCESS_PAGE)
    accessLogs.value = { ...accessLogs.value, [id]: [...known, ...page] }
    markEnded(id, page.length < ACCESS_PAGE)
  }

  function markEnded(id: ConnectionId, ended: boolean): void {
    const next = new Set(accessEnded.value)
    if (ended) next.add(id)
    else next.delete(id)
    accessEnded.value = next
  }

  async function previewKubeconfig(kubeconfig: string): Promise<KubeContextOption[]> {
    return await api.previewKubeconfig(kubeconfig)
  }

  return {
    connectionsOf,
    isLoaded,
    isLoading,
    detailOf,
    checkOf,
    accessOf,
    hasMoreAccess,
    load,
    ensureLoaded,
    loadDetail,
    create,
    update,
    setCredential,
    clearCredential,
    remove,
    check,
    loadAccess,
    loadMoreAccess,
    previewKubeconfig
  }
})

function byLabel(a: Connection, b: Connection): number {
  return a.label.localeCompare(b.label)
}
