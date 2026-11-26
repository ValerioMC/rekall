import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { guardStrayFileDrops } from '@/common/shell/stray-file-drop'

let lift: () => void

function dragged(
  type: 'dragover' | 'drop',
  over: Element | Node,
  types: string[] = ['Files'],
  alreadyTaken = false
): { refused: boolean; effect: string } {
  const dataTransfer = { types, dropEffect: 'copy' }
  const event = new Event(type, { bubbles: true, cancelable: true })
  Object.defineProperty(event, 'dataTransfer', { value: dataTransfer })
  if (alreadyTaken) event.preventDefault()
  over.dispatchEvent(event)
  return { refused: event.defaultPrevented, effect: dataTransfer.dropEffect }
}

beforeEach(() => {
  document.body.innerHTML = `
    <p>page</p>
    <div class="cm-content" contenteditable="true"><div>line</div></div>
    <div class="cm-content" contenteditable="false">read only</div>
    <input type="file" />`
  lift = guardStrayFileDrops(window)
})

afterEach(() => lift())

describe('guardStrayFileDrops', () => {
  it('refuses a file let go where nothing takes it', () => {
    const page = document.querySelector('p')!

    expect(dragged('dragover', page)).toEqual({ refused: true, effect: 'none' })
    expect(dragged('drop', page).refused).toBe(true)
    expect(dragged('drop', document.querySelector('[contenteditable="false"]')!).refused).toBe(true)
  })

  it('leaves files to a writing surface, a file field and a drop zone that took them', () => {
    const line = document.querySelector('.cm-content div')!

    expect(dragged('dragover', line)).toEqual({ refused: false, effect: 'copy' })
    expect(dragged('dragover', line.firstChild!).refused).toBe(false)
    expect(dragged('drop', document.querySelector('input')!).refused).toBe(false)
    expect(dragged('dragover', document.querySelector('p')!, ['Files'], true).effect).toBe('copy')
  })

  it('ignores a drag that carries no file, as the reordering of rows does', () => {
    expect(dragged('dragover', document.querySelector('p')!, ['text/plain']).refused).toBe(false)
  })
})
