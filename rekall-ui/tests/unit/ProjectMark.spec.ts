import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ProjectMark from '@/components/ui/ProjectMark.vue'
import { projectIconImageSrcset, projectIconImageUrl } from '@/features/project/lib/icon-image-url'

const PROJECT = '2758126a-6182-4945-9e1d-8738907b3031'

describe('ProjectMark', () => {
  it('draws the icon key in a tinted tile when the project has no picture', () => {
    const wrapper = mount(ProjectMark, { props: { projectId: PROJECT, icon: 'rocket', size: 24 } })

    expect(wrapper.find('[data-testid="project-mark-glyph"]').exists()).toBe(true)
    expect(wrapper.find('img').exists()).toBe(false)
  })

  it('shows the picture at its own size with one candidate per density in device pixels', () => {
    const wrapper = mount(ProjectMark, {
      props: { projectId: PROJECT, icon: 'folder', imageVersion: '2026-10-05T10:00:00Z', size: 24 }
    })

    const image = wrapper.get('img')
    expect(image.attributes('width')).toBe('24')
    expect(image.attributes('src')).toContain('size=24')
    expect(image.attributes('srcset')).toMatch(/size=24.* 1x, .*size=48.* 2x, .*size=72.* 3x/)
  })

  it('rounds the picture to the macOS icon corner whatever its shape', () => {
    const wrapper = mount(ProjectMark, {
      props: { projectId: PROJECT, icon: 'folder', imageVersion: 'v1', size: 36 }
    })

    expect(wrapper.get('img').attributes('style')).toContain('border-radius: 22.37%')
  })

  it('shows a file just picked, unscaled by the server, in place of the stored picture', () => {
    const wrapper = mount(ProjectMark, {
      props: { projectId: PROJECT, icon: 'folder', imageVersion: 'v1', previewSrc: 'blob:preview', size: 48 }
    })

    expect(wrapper.get('img').attributes('src')).toBe('blob:preview')
    expect(wrapper.get('img').attributes('srcset')).toBeUndefined()
  })
})

describe('project icon image URLs', () => {
  it('names the project, the size in pixels and the version', () => {
    expect(projectIconImageUrl(PROJECT, 'a b', 28)).toBe(
      `/api/projects/${PROJECT}/icon-image?size=28&v=a%20b`
    )
  })

  it('rounds each density to a whole device pixel', () => {
    expect(projectIconImageSrcset(PROJECT, 'v', 17)).toContain('size=51')
  })
})
