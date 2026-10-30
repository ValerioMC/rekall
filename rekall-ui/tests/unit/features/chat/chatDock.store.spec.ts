import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useChatDockStore } from '@/features/chat/store/chatDock.store'
import { installStorage } from '../../support/memoryStorage'

describe('the chat dock', () => {
  beforeEach(() => {
    installStorage()
    setActivePinia(createPinia())
  })

  it('opens at 440 px, visible, when nothing was remembered', () => {
    const dock = useChatDockStore()

    expect(dock.shownWidth).toBe(440)
    expect(dock.hidden).toBe(false)
  })

  it('remembers its width and visibility for the next visit', () => {
    const dock = useChatDockStore()
    dock.setWidth(520)
    dock.hide()

    setActivePinia(createPinia())
    const next = useChatDockStore()

    expect(next.width).toBe(520)
    expect(next.hidden).toBe(true)
  })

  it('ignores a stored width outside its limits', () => {
    installStorage().set('rekall.chat.width', '5000')

    expect(useChatDockStore().width).toBe(440)
  })

  it('keeps the width between 360 and 720 px, and back to 440 on reset', () => {
    const dock = useChatDockStore()

    dock.setWidth(100)
    expect(dock.width).toBe(360)
    dock.setWidth(9000)
    expect(dock.width).toBe(720)
    dock.resetWidth()
    expect(dock.width).toBe(440)
  })

  it('docks inline while the note pane keeps 360 px, and overlays it when it would not', () => {
    const dock = useChatDockStore()

    dock.observe(1200)
    expect(dock.mode).toBe('inline')
    dock.observe(700)
    expect(dock.mode).toBe('overlay')
    expect(dock.shownWidth).toBe(440)
    dock.observe(420)
    expect(dock.shownWidth).toBe(372)
  })

  it('is hidden, full or docked, and restores from full', () => {
    const dock = useChatDockStore()
    dock.observe(1200)

    dock.toggleExpanded()
    expect(dock.mode).toBe('full')
    dock.restore()
    expect(dock.mode).toBe('inline')
    dock.toggle()
    expect(dock.mode).toBe('hidden')
    dock.show()
    expect(dock.mode).toBe('inline')
  })

  it('still works when the browser refuses storage', () => {
    installStorage({
      getItem: () => {
        throw new Error('blocked')
      },
      setItem: () => {
        throw new Error('blocked')
      }
    })
    const dock = useChatDockStore()

    expect(dock.width).toBe(440)
    expect(() => dock.setWidth(500)).not.toThrow()
    expect(dock.width).toBe(500)
  })
})
