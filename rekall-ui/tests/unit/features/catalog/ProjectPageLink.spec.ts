import { describe, expect, it } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import { defineComponent } from 'vue'
import ProjectPageLink from '@/features/catalog/components/ProjectPageLink.vue'
import type { ProjectId } from '@/model/branded'

const Blank = defineComponent({ template: '<div />' })

const vega = 'p-vega' as ProjectId

async function mountLink(): Promise<{ wrapper: ReturnType<typeof mount>; router: Router }> {
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', name: 'console', component: Blank },
      { path: '/projects/:id', name: 'project-detail', component: Blank }
    ]
  })
  await router.push('/')
  await router.isReady()
  const wrapper = mount(ProjectPageLink, {
    props: { projectId: vega, projectTitle: 'Vega Platform' },
    global: { plugins: [router] }
  })
  return { wrapper, router }
}

describe('the link from a navigator project header to its page', () => {
  it('points at that project page and names the project it opens', async () => {
    const { wrapper } = await mountLink()
    const link = wrapper.get('[data-testid="open-project-page"]')
    expect(link.attributes('href')).toBe('/projects/p-vega')
    expect(link.attributes('aria-label')).toBe('Open Vega Platform')
  })

  it('leaves the console for the project page in one click', async () => {
    const { wrapper, router } = await mountLink()
    await wrapper.get('[data-testid="open-project-page"]').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.name).toBe('project-detail')
    expect(router.currentRoute.value.params.id).toBe('p-vega')
  })
})
