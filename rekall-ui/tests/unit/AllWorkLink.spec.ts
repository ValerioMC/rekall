import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createMemoryHistory, createRouter } from 'vue-router'
import { defineComponent } from 'vue'
import AllWorkLink from '@/components/shell/AllWorkLink.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { CompanyId, ProjectId } from '@/model/branded'

vi.mock('@/features/vulnerability/api/vulnerabilities.api')

const Blank = defineComponent({ template: '<div />' })

async function mountLinkOn(path: string) {
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', name: 'work', component: Blank },
      { path: '/projects', name: 'projects', component: Blank }
    ]
  })
  await router.push(path)
  await router.isReady()
  return { router, wrapper: mount(AllWorkLink, { global: { plugins: [router] } }) }
}

describe('the all work link', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('takes the reader to the Work screen from any other screen', async () => {
    const { router, wrapper } = await mountLinkOn('/projects')
    await wrapper.get('[data-testid="all-work-link"]').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.name).toBe('work')
  })

  it('drops the company and project the Work screen was narrowed to', async () => {
    const consoleStore = useConsoleStore()
    consoleStore.setScope('company-1' as CompanyId, 'project-1' as ProjectId)
    const { wrapper } = await mountLinkOn('/projects')
    await wrapper.get('[data-testid="all-work-link"]').trigger('click')
    expect(consoleStore.scopeCompany).toBeNull()
    expect(consoleStore.scopeProject).toBeNull()
  })
})
