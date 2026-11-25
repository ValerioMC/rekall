import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { defineComponent, nextTick, ref } from 'vue'
import { provideStepEditor, type StepEditor } from '@/features/steps/composables/useStepEditor'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskStepId } from '@/model/branded'

const STEP = { id: 's-1' as TaskStepId, title: 'Add the button', bodyMarkdown: 'Old detail' } as TaskStep

async function openEditor() {
  const steps = ref<TaskStep[]>([STEP])
  const save = vi.fn()
  let editor!: StepEditor
  mount(
    defineComponent({
      setup() {
        editor = provideStepEditor({ steps: () => steps.value, save, markUnsaved: vi.fn() })
        return () => null
      }
    })
  )
  editor.open(STEP)
  await nextTick()
  const change = async (changes: Partial<TaskStep>): Promise<void> => {
    steps.value = [{ ...steps.value[0]!, ...changes }]
    await nextTick()
  }
  return { editor, save, change }
}

describe('the step editor', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('shows a rewording that arrives while the open step is untouched', async () => {
    const { editor, change } = await openEditor()

    await change({ title: 'Add the download button', bodyMarkdown: 'New detail' })

    expect(editor.title.value).toBe('Add the download button')
    expect(editor.body.value).toBe('New detail')
  })

  it('keeps what was typed when the step changes under it', async () => {
    const { editor, change } = await openEditor()
    editor.title.value = 'My title'
    editor.body.value = 'My detail'

    await change({ title: 'Add the download button', bodyMarkdown: 'New detail' })

    expect(editor.title.value).toBe('My title')
    expect(editor.body.value).toBe('My detail')
  })

  it('keeps a new line just typed when its own save comes back stripped', async () => {
    const { editor, change } = await openEditor()
    editor.body.value = 'Old detail, extended\n'

    await change({ bodyMarkdown: 'Old detail, extended' })

    expect(editor.body.value).toBe('Old detail, extended\n')
  })

  it('leaves the text alone when only the state of the step changes', async () => {
    const { editor, change } = await openEditor()
    editor.body.value = 'Old detail\n'

    await change({ state: 'RUNNING' })

    expect(editor.body.value).toBe('Old detail\n')
  })
})
