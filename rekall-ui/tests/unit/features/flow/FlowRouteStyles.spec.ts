import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import FlowCanvas from '@/features/flow/components/FlowCanvas.vue'
import { readFlow } from '@/features/flow/lib/step-flow'
import { releaseFlow } from './flow.fixtures'

function canvas() {
  return mount(FlowCanvas, { props: { flow: readFlow(releaseFlow()), selectedId: null } })
}

function routeGroup(wrapper: ReturnType<typeof canvas>, key: string) {
  return wrapper.get(`path[id="flow-route-${key}"]`).element.parentElement!
}

describe('the route states on the flow canvas', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('classes each route as live, walked or pending', () => {
    const wrapper = canvas()

    expect(routeGroup(wrapper, 'fix-fixed').getAttribute('class')).toContain('flow-route-live')
    for (const key of ['build-built', 'deploy-deployed', 'test-app-error']) {
      expect(routeGroup(wrapper, key).getAttribute('class')).toContain('flow-route-walked')
    }
    for (const key of ['test-passed', 'exit:ship']) {
      expect(routeGroup(wrapper, key).getAttribute('class')).toContain('flow-route-pending')
    }
  })

  it('gives a pending route a track and no other route', () => {
    const wrapper = canvas()
    const pending = routeGroup(wrapper, 'test-passed')

    expect(pending.querySelectorAll('.flow-route-track')).toHaveLength(1)
    for (const key of ['fix-fixed', 'build-built']) {
      expect(routeGroup(wrapper, key).querySelector('.flow-route-track')).toBeNull()
    }
  })

  it('marks no direction along any route but the arrowhead at its end', () => {
    const paths = [...canvas().element.querySelectorAll('path.flow-route')]

    expect(paths.length).toBeGreaterThan(0)
    expect(paths.every((path) => path.hasAttribute('marker-end') && !path.hasAttribute('marker-mid'))).toBe(true)
    expect(canvas().findAll('.flow-chevron')).toHaveLength(0)
  })

  it('runs a light only along the live route', () => {
    const wrapper = canvas()

    expect(wrapper.findAll('.flow-route-run')).toHaveLength(1)
    expect(routeGroup(wrapper, 'fix-fixed').querySelector('.flow-route-run')).not.toBeNull()
  })

  it('draws every arrowhead in user-space units so the line weight does not scale it', () => {
    const markers = [...canvas().element.querySelectorAll('marker')]

    expect(markers).toHaveLength(4)
    expect(markers.every((marker) => marker.getAttribute('markerUnits') === 'userSpaceOnUse')).toBe(true)
  })
})
