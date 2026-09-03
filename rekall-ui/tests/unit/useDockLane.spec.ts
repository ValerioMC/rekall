import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, nextTick } from 'vue'
import { mount } from '@vue/test-utils'
import { useDockLane } from '@/composables/useDockLane'

/**
 * The measurement the running dock hands the rest of the interface so the bottom right corner
 * is shared rather than taken. The consequence of getting it wrong is a pill sitting on the
 * button that adds a step, which is how this came to exist.
 *
 * A ResizeObserver that reports once on `observe` and a fixed box, so the numbers under test are
 * the arithmetic of the lane and not the layout of a headless DOM.
 */
class ImmediateResizeObserver implements ResizeObserver {
  constructor(private readonly callback: ResizeObserverCallback) {}

  observe(): void {
    this.callback([], this)
  }

  unobserve(): void {}

  disconnect(): void {}
}

const PILL_WIDTH = 170
const PILL_HEIGHT = 40

const Harness = defineComponent({
  props: { running: { type: Boolean, required: true } },
  setup(props) {
    const pill = useDockLane()
    return () => (props.running ? h('button', { ref: pill }, 'running') : null)
  }
})

function lane(): { width: string; height: string } {
  const style = document.documentElement.style
  return {
    width: style.getPropertyValue('--dock-lane-width'),
    height: style.getPropertyValue('--dock-lane-height')
  }
}

describe('useDockLane', () => {
  beforeEach(() => {
    vi.stubGlobal('ResizeObserver', ImmediateResizeObserver)
    vi.spyOn(Element.prototype, 'getBoundingClientRect').mockReturnValue({
      width: PILL_WIDTH,
      height: PILL_HEIGHT
    } as DOMRect)
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    vi.restoreAllMocks()
    document.documentElement.removeAttribute('style')
  })

  it('publishes the pill plus its gutter and its clearance', async () => {
    const wrapper = mount(Harness, { props: { running: true } })
    await nextTick()

    expect(lane()).toEqual({ width: `${16 + PILL_WIDTH + 12}px`, height: `${10 + PILL_HEIGHT + 12}px` })
    wrapper.unmount()
  })

  it('takes the lane back when nothing is running, so nothing moves on a quiet day', async () => {
    const wrapper = mount(Harness, { props: { running: true } })
    await nextTick()
    expect(lane().width).not.toBe('')

    await wrapper.setProps({ running: false })
    await nextTick()

    expect(lane()).toEqual({ width: '', height: '' })
    wrapper.unmount()
  })

  it('leaves nothing behind when the dock itself goes', async () => {
    const wrapper = mount(Harness, { props: { running: true } })
    await nextTick()

    wrapper.unmount()

    expect(lane()).toEqual({ width: '', height: '' })
  })
})
