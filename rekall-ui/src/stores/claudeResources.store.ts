import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { ApiError } from '@/api/client'
import {
  deleteResourceFile,
  fetchMemoryAnalysis,
  fetchResourceCatalog,
  fetchResourceFile,
  fetchSkillPlacements,
  placeSkill,
  renameSkillGroup,
  saveResourceFile
} from '@/api/claudeResources.api'
import type { ResourceCatalog, ResourceFile, ResourceSection, SkillPlacement } from '@/model/claude-resources'
import type { MemoryAnalysis } from '@/model/memory-analysis'

/**
 * Claude Code's home folder as the console edits it: the catalog, and one open file. A running
 * session may rewrite that file, so a save names the version it was opened from and a refusal
 * becomes a conflict the person resolves. Skill groups live in Rekall only, and the memory review
 * is read from disk when it is opened.
 */
export const useClaudeResourcesStore = defineStore('claudeResources', () => {
  const catalog = ref<ResourceCatalog | null>(null)
  const loaded = ref(false)
  const section = ref<ResourceSection>('memories')
  const query = ref('')
  const file = ref<ResourceFile | null>(null)
  const draft = ref('')
  const opening = ref(false)
  const saving = ref(false)
  const conflict = ref(false)
  const placements = ref<readonly SkillPlacement[]>([])
  const analysis = ref<MemoryAnalysis | null>(null)
  const reviewing = ref(false)
  const analysing = ref(false)

  const selectedPath = computed(() => file.value?.summary.path ?? null)
  const dirty = computed(() => file.value !== null && draft.value !== file.value.body)

  async function load(): Promise<void> {
    const [nextCatalog, nextPlacements] = await Promise.all([fetchResourceCatalog(), fetchSkillPlacements()])
    catalog.value = nextCatalog
    placements.value = nextPlacements
    analysis.value = null
    loaded.value = true
  }

  function show(opened: ResourceFile): void {
    file.value = opened
    draft.value = opened.body
    conflict.value = false
  }

  async function open(path: string): Promise<void> {
    reviewing.value = false
    opening.value = true
    try {
      show(await fetchResourceFile(path))
    } finally {
      opening.value = false
    }
  }

  function close(): void {
    file.value = null
    draft.value = ''
    conflict.value = false
  }

  /** Shows the review in place of the open file, reading the memories again. */
  async function openReview(): Promise<void> {
    close()
    reviewing.value = true
    await analyse()
  }

  async function analyse(): Promise<void> {
    analysing.value = true
    try {
      analysis.value = await fetchMemoryAnalysis()
    } finally {
      analysing.value = false
    }
  }

  function closeReview(): void {
    reviewing.value = false
  }

  /** Moves a skill into a group, or out of its group with none; only Rekall's own record changes. */
  async function moveSkill(skillPath: string, group: string | null): Promise<void> {
    placements.value = await placeSkill(skillPath, group)
  }

  async function renameGroup(from: string, to: string): Promise<void> {
    placements.value = await renameSkillGroup(from, to)
  }

  function revert(): void {
    draft.value = file.value?.body ?? ''
  }

  /** A refusal for a stale version keeps the draft and raises the conflict; any other failure is thrown. */
  async function save(): Promise<boolean> {
    const current = file.value
    if (!current) return false
    saving.value = true
    try {
      show(await saveResourceFile(current.summary.path, draft.value, current.summary.modifiedAt))
      await load()
      return true
    } catch (failure) {
      if (failure instanceof ApiError && failure.status === 409) {
        conflict.value = true
        return false
      }
      throw failure
    } finally {
      saving.value = false
    }
  }

  /** Keeps the draft and writes it over whatever is on disk now. */
  async function overwrite(): Promise<void> {
    const current = file.value
    if (!current) return
    const onDisk = await fetchResourceFile(current.summary.path)
    file.value = { summary: onDisk.summary, body: onDisk.body }
    conflict.value = false
    await save()
  }

  /** Drops the draft for what is on disk now. */
  async function reloadFromDisk(): Promise<void> {
    const current = file.value
    if (!current) return
    show(await fetchResourceFile(current.summary.path))
  }

  async function create(path: string, body: string): Promise<void> {
    show(await saveResourceFile(path, body, null))
    await load()
  }

  async function remove(path: string): Promise<void> {
    await deleteResourceFile(path)
    if (file.value?.summary.path === path) close()
    await load()
  }

  return {
    catalog, loaded, section, query, file, draft, opening, saving, conflict, placements, analysis, reviewing, analysing,
    selectedPath, dirty, load, open, close, openReview, analyse, closeReview, moveSkill, renameGroup, revert, save, overwrite, reloadFromDisk, create, remove
  }
})
