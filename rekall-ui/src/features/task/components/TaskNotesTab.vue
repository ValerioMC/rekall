<script setup lang="ts">
import { computed, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useRoute, useRouter } from 'vue-router'
import AppButton from '@/components/ui/AppButton.vue'
import NotePane from '@/features/notes/components/NotePane.vue'
import TaskNoteList from '@/features/task/components/TaskNoteList.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useConsoleStore } from '@/stores/console.store'
import type { Task } from '@/features/catalog/model/catalog'
import type { RekallDocument } from '@/features/notes/model/document'
import { asDocumentId } from '@/model/branded'

/**
 * The Notes tab: the task's notes as a list beside the note pane. The note in view is in the
 * URL (`/task/:id/notes/:noteId`), so a link or a search hit lands on it and the arrows walk them.
 */
const props = defineProps<{ task: Task }>()

const store = useConsoleStore()
const route = useRoute()
const router = useRouter()
const { run } = useAsyncAction()
const { taskDocuments, selectedDocId } = storeToRefs(store)

const current = computed<RekallDocument | null>(
  () => taskDocuments.value.find((note) => note.id === selectedDocId.value) ?? taskDocuments.value[0] ?? null
)
const routedNoteId = computed(() => (typeof route.params.noteId === 'string' ? asDocumentId(route.params.noteId) : null))

watch(
  [routedNoteId, taskDocuments],
  ([id, notes]) => {
    const wanted = notes.find((note) => note.id === id)
    if (wanted && wanted.id !== selectedDocId.value) store.selectDocument(wanted.id)
  },
  { immediate: true }
)

watch(
  () => current.value?.id ?? null,
  (id) => {
    if (id === null) return
    if (id !== selectedDocId.value) store.selectDocument(id)
    if (id === routedNoteId.value) return
    void router.replace({ name: 'task', params: { id: props.task.id, tab: 'notes', noteId: id } })
  },
  { immediate: true }
)

function select(note: RekallDocument): void {
  store.selectDocument(note.id)
}

async function addFirst(): Promise<void> {
  await run(() => store.createNote(props.task.id), 'Information added')
}
</script>

<template>
  <div v-if="current" class="flex min-h-0 flex-1" data-testid="notes-tab">
    <TaskNoteList :task="task" :current="current" @select="select" />
    <NotePane />
  </div>
  <div v-else class="grid flex-1 place-items-center px-6" data-testid="notes-empty">
    <div class="flex max-w-sm flex-col items-center gap-3 text-center">
      <p class="text-body font-medium text-text">Nothing here yet</p>
      <p class="text-small text-text-muted">Notes attach context to this task: a reference, a decision, a finding.</p>
      <AppButton size="sm" data-testid="information-add" @click="addFirst">+ Add information</AppButton>
    </div>
  </div>
</template>
