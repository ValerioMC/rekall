import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import BackupsSection from '@/components/settings/BackupsSection.vue'
import type { BackupFile, BackupStatus } from '@/model/backup'

const fetchBackupStatus = vi.fn<() => Promise<BackupStatus>>()
const takeBackup = vi.fn<() => Promise<BackupFile>>()
const restoreBackup = vi.fn<(name: string) => Promise<BackupFile>>()
vi.mock('@/api/backups.api', () => ({
  fetchBackupStatus: () => fetchBackupStatus(),
  takeBackup: () => takeBackup(),
  restoreBackup: (name: string) => restoreBackup(name),
  restoreUploadedBackup: vi.fn(),
  backupDownloadUrl: (name: string) => `/api/backups/${name}`
}))

const reloadWhenBack = vi.fn(async () => true)
vi.mock('@/composables/useDatabaseSetup', () => ({ reloadWhenBack: () => reloadWhenBack() }))

const backup = (name: string, reason: BackupFile['reason']): BackupFile => ({
  name,
  sizeBytes: 2048,
  createdAt: '2026-09-22T08:00:00Z',
  reason
})

const status = (backups: BackupFile[]): BackupStatus => ({
  available: true,
  folder: '/data/rekall/backups',
  intervalHours: 24,
  keep: 10,
  backups,
  lastFailure: null
})

describe('the Backups section', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    fetchBackupStatus.mockReset()
    takeBackup.mockReset()
    restoreBackup.mockReset()
    reloadWhenBack.mockClear()
  })

  it('lists the backups with a download link each, and takes one on demand', async () => {
    fetchBackupStatus.mockResolvedValueOnce(status([backup('rekall-20260922-080000-auto.zip', 'AUTO')]))
    const wrapper = mount(BackupsSection)
    await flushPromises()

    const row = wrapper.find('[data-testid="backup-row"]')
    expect(row.text()).toContain('automatic')
    expect(row.find('a').attributes('href')).toBe('/api/backups/rekall-20260922-080000-auto.zip')

    takeBackup.mockResolvedValueOnce(backup('rekall-20260922-090000-manual.zip', 'MANUAL'))
    fetchBackupStatus.mockResolvedValueOnce(
      status([backup('rekall-20260922-090000-manual.zip', 'MANUAL'), backup('rekall-20260922-080000-auto.zip', 'AUTO')])
    )
    await wrapper.find('[data-testid="backup-now"]').trigger('click')
    await flushPromises()

    expect(wrapper.findAll('[data-testid="backup-row"]')).toHaveLength(2)
  })

  it('restores only once the confirmation is accepted, then waits for the restart and says it is busy meanwhile', async () => {
    fetchBackupStatus.mockResolvedValue(status([backup('rekall-20260922-080000-auto.zip', 'AUTO')]))
    restoreBackup.mockResolvedValue(backup('rekall-20260922-100000-before-restore.zip', 'BEFORE_RESTORE'))
    const wrapper = mount(BackupsSection, { attachTo: document.body })
    await flushPromises()

    await wrapper.find('[data-testid="backup-restore"]').trigger('click')
    expect(restoreBackup).not.toHaveBeenCalled()
    expect(wrapper.emitted('busy')?.at(-1)).toEqual([true])

    const confirm = [...document.body.querySelectorAll('button')].find((button) =>
      button.textContent?.includes('Restore and restart')
    )
    confirm!.click()
    await flushPromises()

    expect(restoreBackup).toHaveBeenCalledWith('rekall-20260922-080000-auto.zip')
    expect(reloadWhenBack).toHaveBeenCalledOnce()
    wrapper.unmount()
  })

  it('says so when the database is not a file and has nothing to back up', async () => {
    fetchBackupStatus.mockResolvedValue({ ...status([]), available: false, folder: null })
    const wrapper = mount(BackupsSection)
    await flushPromises()

    expect(wrapper.text()).toContain('not a file on disk')
    expect(wrapper.find('[data-testid="backup-now"]').exists()).toBe(false)
  })
})
