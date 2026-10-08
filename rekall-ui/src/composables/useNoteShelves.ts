import { reactive, ref, watch, type Ref } from 'vue'
import { loadPersistedSet, savePersistedSet, toggledSet } from '@/common/config/persisted-set'
import { scopeKey } from '@/model/note-scope'
import type { RekallDocument } from '@/model/catalog'

const COLLAPSE_KEY = 'rekall.nav.collapsed-note-groups'

/**
 * What the notes navigator has folded away: whole scope cards, kept across restarts, and each
 * scope's loose shelf. A loose note picked from elsewhere (search, the walk) opens its shelf so
 * the selection is visible.
 */
export interface NoteShelves {
  readonly collapsedGroups: ReadonlySet<string>
  readonly openLoose: ReadonlySet<string>
  toggleCollapsed(groupKey: string): void
  toggleLoose(groupKey: string): void
}

export function useNoteShelves(selectedNote: Ref<RekallDocument | null>): NoteShelves {
  const collapsed = ref<ReadonlySet<string>>(loadPersistedSet<string>(COLLAPSE_KEY))
  const loose = ref<ReadonlySet<string>>(new Set())

  const shelves = reactive({
    collapsedGroups: collapsed,
    openLoose: loose,
    toggleCollapsed(groupKey: string): void {
      collapsed.value = toggledSet(collapsed.value, groupKey)
      savePersistedSet(COLLAPSE_KEY, collapsed.value)
    },
    toggleLoose(groupKey: string): void {
      loose.value = toggledSet(loose.value, groupKey)
    }
  })

  watch(
    () => selectedNote.value?.id,
    () => {
      const note = selectedNote.value
      if (!note || note.tasks.length) return
      const groupKey = scopeKey(note.scope)
      if (!loose.value.has(groupKey)) shelves.toggleLoose(groupKey)
    },
    { immediate: true }
  )

  return shelves
}
