import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/features/workingset/api/working-set.api'
import ProjectTaskOpenButton from '@/features/project/components/ProjectTaskOpenButton.vue'
import { useWorkingSetStore } from '@/features/workingset/store/workingSet.store'
import type { ProjectId, TaskId } from '@/model/branded'

vi.mock('@/features/workingset/api/working-set.api')
vi.mock('vue-router', () => ({ useRouter: () => ({ push: vi.fn() }) }))

const T1 = 't1' as TaskId

describe('the open marker on a ledger row', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('is shown only for a task that is open in the bar', async () => {
    vi.mocked(api.fetchWorkingSet).mockResolvedValue({ tasks: [{ taskId: T1, projectId: 'vega' as ProjectId }], places: [] })
    await useWorkingSetStore().load()

    const open = mount(ProjectTaskOpenButton, { props: { taskId: T1, title: 'Report' } })
    const other = mount(ProjectTaskOpenButton, { props: { taskId: 't2' as TaskId, title: 'Deploy' } })

    expect(open.find('[data-testid="task-row-working-set"]').exists()).toBe(true)
    expect(other.find('[data-testid="task-row-working-set"]').exists()).toBe(false)
  })
})
