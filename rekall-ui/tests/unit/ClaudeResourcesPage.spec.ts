import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createMemoryHistory, createRouter } from 'vue-router'
import { defineComponent } from 'vue'
import ClaudeResourcesPage from '@/components/claude-resources/ClaudeResourcesPage.vue'
import { useConsoleStore } from '@/stores/console.store'
import { catalog, file, group, summary } from './claude-resources.fixtures'


const api = vi.hoisted(() => ({
  fetchResourceCatalog: vi.fn(),
  fetchResourceFile: vi.fn(),
  saveResourceFile: vi.fn(),
  deleteResourceFile: vi.fn(),
  fetchSkillPlacements: vi.fn(),
  placeSkill: vi.fn(),
  renameSkillGroup: vi.fn(),
  fetchMemoryAnalysis: vi.fn()
}))
vi.mock('@/api/claudeResources.api', () => api)

// The editor is a CodeMirror instance: the page is what is under test, not md-editor-v3.
const EditorStub = defineComponent({
  props: { modelValue: { type: String, default: '' }, readonly: Boolean },
  emits: ['update:modelValue'],
  template: '<textarea data-testid="editor" :readonly="readonly" :value="modelValue" @input="$emit(\'update:modelValue\', $event.target.value)" />'
})

const CATALOG = catalog({
  instructions: summary('CLAUDE.md', { kind: 'instructions', name: 'CLAUDE.md' }),
  skills: [summary('skills/docx/SKILL.md', { kind: 'skill', name: 'docx', description: 'word files' })],
  memoryGroups: [
    group('-work-vega', '/work/vega', [summary('projects/-work-vega/memory/lean.md', { name: 'Lean visual', memoryType: 'feedback', description: 'few rounds' })]),
    group('-gone', '/gone/away', [summary('projects/-gone/memory/old.md', { name: 'Old rule' })], { folderExists: false })
  ]
})

async function page() {
  const router = createRouter({ history: createMemoryHistory(), routes: [{ path: '/', component: { template: '<div />' } }] })
  await router.push('/')
  const wrapper = mount(ClaudeResourcesPage, {
    global: { plugins: [router], stubs: { AppMarkdownEditor: EditorStub, Teleport: true } },
    attachTo: document.body
  })
  await flushPromises()
  return wrapper
}

describe('the Claude resources page', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    document.body.innerHTML = ''
    api.fetchResourceCatalog.mockResolvedValue(CATALOG)
    api.fetchSkillPlacements.mockResolvedValue([])
    useConsoleStore().projects = []
  })

  it('lists memories per project, folded, with a gone folder flagged', async () => {
    const wrapper = await page()

    const groups = wrapper.findAll('[data-testid="memory-group"]')
    expect(groups).toHaveLength(2)
    expect(groups[0]?.text()).toContain('vega')
    expect(groups[1]?.text()).toContain('gone')
    expect(groups[0]?.text()).not.toContain('gone')
    expect(groups[1]?.find('button').text()).toContain('gone')
    expect(wrapper.find('[data-testid="resource-tab-memories"]').text()).toContain('2')
  })

  it('opens a memory, marks it dirty when edited, and saves it from the keyboard', async () => {
    api.fetchResourceFile.mockResolvedValue(file('projects/-work-vega/memory/lean.md', 'old', { name: 'Lean visual' }))
    api.saveResourceFile.mockResolvedValue(file('projects/-work-vega/memory/lean.md', 'new', { name: 'Lean visual' }))
    const wrapper = await page()

    await wrapper.findAll('[data-testid="memory-group"] > button')[0]!.trigger('click')
    await wrapper.find('[data-testid="resource-row"]').trigger('click')
    await flushPromises()
    expect(wrapper.find('[data-testid="resource-title"]').text()).toBe('Lean visual')
    expect(wrapper.find('[data-testid="resource-dirty"]').exists()).toBe(false)

    await wrapper.find('[data-testid="editor"]').setValue('new')
    expect(wrapper.find('[data-testid="resource-dirty"]').exists()).toBe(true)

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 's', metaKey: true }))
    await flushPromises()
    expect(api.saveResourceFile).toHaveBeenCalledWith('projects/-work-vega/memory/lean.md', 'new', '2026-10-12T09:00:00.000Z')
    expect(wrapper.find('[data-testid="resource-dirty"]').exists()).toBe(false)
  })

  it('shows the conflict bar when the file changed on disk, with its two ways out', async () => {
    const { ApiError } = await import('@/api/client')
    api.fetchResourceFile.mockResolvedValue(file('CLAUDE.md', 'old', { kind: 'instructions', name: 'CLAUDE.md' }))
    api.saveResourceFile.mockRejectedValue(new ApiError(409, 'changed', ''))
    const wrapper = await page()
    await wrapper.find('[data-testid="resource-row"]').trigger('click')
    await flushPromises()
    await wrapper.find('[data-testid="editor"]').setValue('mine')

    await wrapper.find('[data-testid="resource-save"]').trigger('click')
    await flushPromises()

    expect(wrapper.find('[data-testid="resource-conflict"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="resource-conflict-reload"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="resource-conflict-overwrite"]').exists()).toBe(true)
    expect((wrapper.find('[data-testid="editor"]').element as HTMLTextAreaElement).value).toBe('mine')
  })

  it('asks before leaving unsaved edits for another file', async () => {
    api.fetchResourceFile.mockImplementation(async (path: string) => file(path, 'old', { kind: path === 'CLAUDE.md' ? 'instructions' : 'skill', name: path }))
    const wrapper = await page()
    await wrapper.find('[data-testid="resource-row"]').trigger('click')
    await flushPromises()
    await wrapper.find('[data-testid="editor"]').setValue('mine')

    await wrapper.find('[data-testid="resource-tab-skills"]').trigger('click')
    // Row 0 is the pinned instructions, already open; row 1 is the first skill.
    await wrapper.findAll('[data-testid="resource-row"]')[1]!.trigger('click')
    await flushPromises()

    expect(document.body.textContent).toContain('Discard your edits?')
    expect(api.fetchResourceFile).toHaveBeenCalledTimes(1)
  })

  it('deletes a memory only after the confirmation', async () => {
    api.fetchResourceFile.mockResolvedValue(file('projects/-work-vega/memory/lean.md', 'x', { name: 'Lean visual' }))
    api.deleteResourceFile.mockResolvedValue(undefined)
    const wrapper = await page()
    await wrapper.findAll('[data-testid="memory-group"] > button')[0]!.trigger('click')
    await wrapper.find('[data-testid="resource-row"]').trigger('click')
    await flushPromises()

    await wrapper.find('[data-testid="resource-delete"]').trigger('click')
    expect(api.deleteResourceFile).not.toHaveBeenCalled()
    expect(document.body.textContent).toContain('its line leaves MEMORY.md')

    const confirm = [...document.querySelectorAll('button')].find((button) => button.textContent?.includes('Delete file'))!
    confirm.click()
    await flushPromises()
    expect(api.deleteResourceFile).toHaveBeenCalledWith('projects/-work-vega/memory/lean.md')
  })

  it('filters the list and opens every matching group', async () => {
    const wrapper = await page()

    await wrapper.find('[data-testid="resource-search"]').setValue('old rule')

    const groups = wrapper.findAll('[data-testid="memory-group"]')
    expect(groups).toHaveLength(1)
    expect(groups[0]?.text()).toContain('Old rule')
  })

  it('creates a skill named by the person in the place Claude Code looks', async () => {
    api.saveResourceFile.mockResolvedValue(file('skills/my-skill/SKILL.md', '---\n', { kind: 'skill' }))
    const wrapper = await page()
    await wrapper.find('[data-testid="resource-tab-skills"]').trigger('click')

    await wrapper.find('[data-testid="resource-new"]').trigger('click')
    const name = document.querySelector<HTMLInputElement>('[data-testid="new-resource-name"]')!
    name.value = 'My Skill'
    name.dispatchEvent(new Event('input'))
    await flushPromises()
    expect(document.querySelector('[data-testid="new-resource-path"]')?.textContent).toContain('~/.claude/skills/my-skill/SKILL.md')
    document.querySelector<HTMLButtonElement>('[data-testid="new-resource-submit"]')!.click()
    await flushPromises()

    expect(api.saveResourceFile).toHaveBeenCalledWith('skills/my-skill/SKILL.md', expect.stringContaining('name: my-skill'), null)
  })

  it('nests a project inside the one whose folder it sits in', async () => {
    api.fetchResourceCatalog.mockResolvedValue(
      catalog({
        memoryGroups: [
          group('-work-esa', '/work/esa', [summary('projects/-work-esa/memory/a.md', { name: 'Parent rule' })]),
          group('-work-esa-core', '/work/esa/core', [summary('projects/-work-esa-core/memory/b.md', { name: 'Child rule' })])
        ]
      })
    )
    const wrapper = await page()

    const top = wrapper.findAll('[data-testid="memory-group"]')[0]!
    expect(wrapper.findAll('[data-testid="memory-group"]')).toHaveLength(2)
    expect(top.find('[data-testid="memory-group"]').exists()).toBe(true)
    expect(top.find('[data-testid="memory-group"]').text()).toContain('core')
    expect(top.find('button').text()).toContain('+1')
  })

  it('shelves read-only skills by who provides them, folded, and keeps the persons own open', async () => {
    api.fetchResourceCatalog.mockResolvedValue(
      catalog({
        skills: [
          summary('skills/mine/SKILL.md', { kind: 'skill', name: 'mine', source: { kind: 'own', name: 'You', detail: null } }),
          summary('skills/synced/b/pdf/SKILL.md', { kind: 'skill', name: 'pdf', editable: false, source: { kind: 'synced', name: 'claude.ai', detail: 'anthropic' } })
        ]
      })
    )
    const wrapper = await page()
    await wrapper.find('[data-testid="resource-tab-skills"]').trigger('click')

    const own = wrapper.find('[data-testid="skill-shelf-own"]')
    const locked = wrapper.find('[data-testid="skill-shelf-locked"]')
    expect(own.text()).toContain('mine')
    expect(locked.text()).toContain('claude.ai')
    expect(locked.text()).toContain('Made by Anthropic')
    expect(locked.find('[aria-expanded]').attributes('aria-expanded')).toBe('false')
    expect(own.find('[data-testid="skill-group-trigger"]').exists()).toBe(true)
    expect(locked.find('[data-testid="skill-group-trigger"]').exists()).toBe(false)
  })

  it('files a skill into a new group without writing to the disk', async () => {
    api.fetchResourceCatalog.mockResolvedValue(
      catalog({ skills: [summary('skills/mine/SKILL.md', { kind: 'skill', name: 'mine', source: { kind: 'own', name: 'You', detail: null } })] })
    )
    api.placeSkill.mockResolvedValue([{ skillPath: 'skills/mine/SKILL.md', group: 'Languages' }])
    const wrapper = await page()
    await wrapper.find('[data-testid="resource-tab-skills"]').trigger('click')

    await wrapper.find('[data-testid="skill-group-trigger"]').trigger('click')
    document.querySelector<HTMLButtonElement>('[data-testid="skill-group-new"]')!.click()
    await flushPromises()
    const name = document.querySelector<HTMLInputElement>('[data-testid="skill-group-name"]')!
    name.value = 'Languages'
    name.dispatchEvent(new Event('input'))
    name.form!.dispatchEvent(new Event('submit', { cancelable: true }))
    await flushPromises()

    expect(api.placeSkill).toHaveBeenCalledWith('skills/mine/SKILL.md', 'Languages')
    expect(api.saveResourceFile).not.toHaveBeenCalled()
    expect(wrapper.find('[data-testid="skill-shelf-own"]').text()).toContain('Languages')
  })

  it('opens the review, lists what it found and opens a memory from it', async () => {
    api.fetchMemoryAnalysis.mockResolvedValue({
      memoriesRead: 4,
      documentsRead: 2,
      promotions: [
        {
          topic: ['dash', 'comment'],
          memories: [
            { path: 'projects/-work-vega/memory/lean.md', name: 'No em dashes', groupId: '-work-vega', memoryType: 'feedback' },
            { path: 'projects/-gone/memory/old.md', name: 'No dashes', groupId: '-gone', memoryType: 'feedback' }
          ],
          groupCount: 2,
          target: 'instructions',
          reason: 'Short and the same everywhere.'
        }
      ],
      obsolete: [
        {
          memory: { path: 'projects/-work-vega/memory/lean.md', name: 'Makefile style', groupId: '-work-vega', memoryType: null },
          coveredBy: { kind: 'globalInstructions', label: 'CLAUDE.md', location: '/home/me/.claude/CLAUDE.md', resourcePath: 'CLAUDE.md' },
          overlapPercent: 82,
          sharedPhrases: ['phony target']
        }
      ]
    })
    api.fetchResourceFile.mockResolvedValue(file('projects/-work-vega/memory/lean.md', 'x', { name: 'Lean visual' }))
    const wrapper = await page()

    await wrapper.find('[data-testid="review-entry"]').trigger('click')
    await flushPromises()

    expect(wrapper.find('[data-testid="review-count"]').text()).toBe('2')
    expect(wrapper.find('[data-testid="promotion-card"]').text()).toContain('Add to CLAUDE.md')
    expect(wrapper.find('[data-testid="promotion-card"]').text()).toContain('in 2 projects')
    expect(wrapper.find('[data-testid="obsolete-card"]').text()).toContain('82%')
    expect(wrapper.find('[data-testid="obsolete-card"]').text()).toContain('phony target')

    await wrapper.find('[data-testid="review-open-memory"]').trigger('click')
    await flushPromises()
    expect(api.fetchResourceFile).toHaveBeenCalledWith('projects/-work-vega/memory/lean.md')
    expect(wrapper.find('[data-testid="resource-title"]').text()).toBe('Lean visual')
    expect(wrapper.find('[data-testid="resource-tab-memories"]').attributes('aria-selected')).toBe('true')
  })

  it('says so when the review finds nothing', async () => {
    api.fetchMemoryAnalysis.mockResolvedValue({ memoriesRead: 4, documentsRead: 2, promotions: [], obsolete: [] })
    const wrapper = await page()

    await wrapper.find('[data-testid="review-entry"]').trigger('click')
    await flushPromises()

    expect(wrapper.find('[data-testid="review-empty"]').exists()).toBe(true)
  })
})
