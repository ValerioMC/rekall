import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { defineComponent, ref } from 'vue'
import { useFlowDrag } from '@/features/flow/composables/useFlowDrag'

function press(x: number, y: number): PointerEvent {
  return Object.assign(new MouseEvent('pointerdown', { clientX: x, clientY: y, button: 0 }), {}) as unknown as PointerEvent
}

function mountDrag(scale: number) {
  const place = vi.fn().mockResolvedValue(undefined)
  let drag!: ReturnType<typeof useFlowDrag>
  const wrapper = mount(
    defineComponent({
      setup() {
        drag = useFlowDrag({
          surface: ref(null),
          scale: () => scale,
          boxes: () => new Map([['a', { x: 100, y: 100, width: 200, height: 60 }]]),
          endBox: () => ({ x: 0, y: 0, width: 0, height: 0 }),
          place,
          connect: vi.fn(),
          select: vi.fn()
        })
        return () => null
      }
    })
  )
  return { drag, place, wrapper }
}

describe('dragging a step on a zoomed canvas', () => {
  it('moves it by the pointer travel divided by the zoom, so it follows the pointer', () => {
    const { drag, wrapper } = mountDrag(0.5)

    drag.startMove('a', press(0, 0))
    window.dispatchEvent(new MouseEvent('pointermove', { clientX: 40, clientY: 20 }))

    expect(drag.live.value.get('a')).toEqual({ x: 180, y: 140 })
    wrapper.unmount()
  })

  it('moves it by the plain travel at full zoom', () => {
    const { drag, wrapper } = mountDrag(1)

    drag.startMove('a', press(0, 0))
    window.dispatchEvent(new MouseEvent('pointermove', { clientX: 40, clientY: 20 }))

    expect(drag.live.value.get('a')).toEqual({ x: 140, y: 120 })
    wrapper.unmount()
  })
})
