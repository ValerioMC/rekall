import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import ProjectBlueprintPane from '@/features/project/components/ProjectBlueprintPane.vue'
import ProjectComponentPane from '@/features/project/components/ProjectComponentPane.vue'
import ProjectSectionPane from '@/features/project/components/ProjectSectionPane.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useProjectSectionsStore } from '@/features/project/store/projectSections.store'
import type { Project } from '@/features/catalog/model/catalog'
import type { ProjectComponent } from '@/features/project/model/project-component'
import type { ProjectSection } from '@/features/project/model/project-section'
import { asProjectComponentId, asProjectId, asProjectSectionId, asSectionContextId } from '@/model/branded'

const PROJECT_ID = asProjectId('p-1')
const SECTION = { id: asProjectSectionId('s-1'), projectId: PROJECT_ID, contextId: asSectionContextId('c-1'), title: 'Layers', bodyMarkdown: 'Text', position: 0, updatedAt: '2026-10-08T10:00:00Z' } as ProjectSection
const COMPONENT = { id: asProjectComponentId('k-1'), projectId: PROJECT_ID, name: 'api', folder: '/work/api', summary: 'About', description: 'Text', position: 0, updatedAt: '2026-10-08T10:00:00Z' } as ProjectComponent

const stubs = {
  AppMarkdownEditor: { props: ['modelValue'], emits: ['update:modelValue'], template: '<div data-testid="markdown" />' },
  ExportDialog: { props: ['project', 'initial'], template: '<div data-testid="export-dialog" :data-initial="JSON.stringify(initial)" />' },
  ProjectSectionContextChip: true,
  ProjectSectionGauge: true,
  ProjectComponentOutline: true,
  FolderPickerButton: true,
  TaskWidthMenu: true
}

function projectWith(blueprintMarkdown: string | null): Project {
  return { id: PROJECT_ID, title: 'Vega', companyName: 'VFORGE', anchor: 'project:vega', repoFolder: null, blueprintMarkdown } as unknown as Project
}

let pinia: Pinia
let wrapper: VueWrapper | null = null

function mountPane(component: unknown, props: Record<string, unknown>, project: Project = projectWith('The blueprint.')): VueWrapper {
  useConsoleStore().projects = [project]
  wrapper = mount(component as never, { props, global: { plugins: [pinia], stubs } } as never)
  return wrapper
}

const SECTION_PROPS = { section: SECTION, contexts: [], sections: [SECTION], pages: [], repoFolder: null, canMoveUp: false, canMoveDown: false }
const COMPONENT_PROPS = { component: COMPONENT, repoFolder: null, canMoveUp: false, canMoveDown: false }

beforeEach(() => {
  pinia = createPinia()
  setActivePinia(pinia)
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
})

describe('the blueprint pane', () => {
  it('opens the export dialog with the blueprint ticked', async () => {
    const pane = mountPane(ProjectBlueprintPane, { project: projectWith('The blueprint.') })

    await pane.get('[data-testid="export-action"]').trigger('click')
    await flushPromises()

    expect(JSON.parse(pane.get('[data-testid="export-dialog"]').attributes('data-initial') ?? '')).toEqual({ blueprint: true, sectionIds: [], componentIds: [] })
  })

  it('disables Export and says why when the blueprint is blank', () => {
    const pane = mountPane(ProjectBlueprintPane, { project: projectWith(null) }, projectWith(null))

    expect(pane.get('[data-testid="export-action"]').attributes('disabled')).toBeDefined()
    expect(pane.get('[data-testid="export-action-frame"]').attributes('title')).toBe('The blueprint is empty.')
  })
})

describe('the page pane', () => {
  it('opens the export dialog with this page ticked', async () => {
    const pane = mountPane(ProjectSectionPane, SECTION_PROPS)

    await pane.get('[data-testid="export-action"]').trigger('click')
    await flushPromises()

    expect(JSON.parse(pane.get('[data-testid="export-dialog"]').attributes('data-initial') ?? '')).toEqual({ blueprint: false, sectionIds: ['s-1'], componentIds: [] })
  })

  it('saves a draft that is still waiting before it opens the dialog', async () => {
    const store = useProjectSectionsStore()
    const update = vi.fn(async () => SECTION)
    store.update = update as never
    const pane = mountPane(ProjectSectionPane, SECTION_PROPS)

    await pane.get('[data-testid="section-title"]').setValue('Layers, revised')
    await pane.get('[data-testid="export-action"]').trigger('click')
    await flushPromises()

    expect(update).toHaveBeenCalledWith(SECTION.id, { title: 'Layers, revised', bodyMarkdown: 'Text' })
    expect(pane.find('[data-testid="export-dialog"]').exists()).toBe(true)
  })

  it('keeps the export key beside the width menu and mode toggle, and the page’s own keys apart', () => {
    const pane = mountPane(ProjectSectionPane, SECTION_PROPS)

    expect(pane.get('[data-testid="export-action"]').element.parentElement?.parentElement?.querySelector('[data-testid="section-mode-read"]')).not.toBeNull()
    expect(pane.get('[data-testid="section-delete"]').element.parentElement?.contains(pane.get('[data-testid="export-action"]').element)).toBe(false)
  })
})

describe('the component pane', () => {
  it('opens the export dialog with this component ticked', async () => {
    const pane = mountPane(ProjectComponentPane, COMPONENT_PROPS)

    await pane.get('[data-testid="export-action"]').trigger('click')
    await flushPromises()

    expect(JSON.parse(pane.get('[data-testid="export-dialog"]').attributes('data-initial') ?? '')).toEqual({ blueprint: false, sectionIds: [], componentIds: ['k-1'] })
  })

  it('stays disabled until the project is known to the console', () => {
    wrapper = mount(ProjectComponentPane, { props: COMPONENT_PROPS, global: { plugins: [pinia], stubs } })

    expect(wrapper.get('[data-testid="export-action"]').attributes('disabled')).toBeDefined()
  })
})
