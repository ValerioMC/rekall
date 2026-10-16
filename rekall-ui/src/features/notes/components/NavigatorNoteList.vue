<script setup lang="ts">
/** The navigator in notes mode: every note in scope, on a card for the project, company or workspace it lives in. */
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import NavigatorNoteGroup from '@/features/notes/components/NavigatorNoteGroup.vue'
import { groupNotesByScope } from '@/features/notes/lib/note-groups'
import { useNoteShelves } from '@/features/notes/composables/useNoteShelves'
import { useConsoleStore } from '@/stores/console.store'

const store = useConsoleStore()
const { visibleDocuments, projects, companies, selectedDocId, selectedDocument, noteComposerOpen } =
  storeToRefs(store)

const noteGroups = computed(() => groupNotesByScope(visibleDocuments.value, projects.value, companies.value))
const shelves = useNoteShelves(selectedDocument)

/** While a new note is being written, no listed note is the one in view. */
const highlightedId = computed(() => (noteComposerOpen.value ? null : selectedDocId.value))
</script>

<template>
  <NavigatorNoteGroup
    v-for="group in noteGroups"
    :key="group.key"
    :group="group"
    :shelves="shelves"
    :selected-note-id="highlightedId"
    @select="store.selectDocument"
  />

  <p v-if="!visibleDocuments.length" class="px-4 py-3 text-small leading-relaxed text-text-subtle">No note matches.</p>
</template>
