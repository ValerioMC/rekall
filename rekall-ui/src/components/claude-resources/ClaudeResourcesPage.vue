<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'
import { storeToRefs } from 'pinia'
import AppCatalogHeader from '@/components/catalog/AppCatalogHeader.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import NewResourceDialog from './NewResourceDialog.vue'
import ResourceEditor from './ResourceEditor.vue'
import ResourceRail from './ResourceRail.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useDiscardGuard } from '@/composables/useDiscardGuard'
import { useClaudeResourcesStore } from '@/stores/claudeResources.store'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import type { ResourceSection } from '@/model/claude-resources'

/**
 * The Claude screen: the library of `~/.claude` on the left, the open file on the right. Saving is
 * explicit and names the version opened, because a running session may write the same file.
 */
const store = useClaudeResourcesStore()
const { catalog, section, query, file, draft, dirty, saving, conflict, selectedPath, loaded } = storeToRefs(store)
const console_ = useConsoleStore()
const toast = useToastStore()
const { run, isRunning } = useAsyncAction()
const { asking, guard, askToLeave, confirm, cancel } = useDiscardGuard(dirty)

const creating = ref(false)
const confirmingDelete = ref(false)

const groupOfOpenFile = computed(
  () => catalog.value?.memoryGroups.find((group) => group.memories.some((memory) => memory.path === selectedPath.value) || group.index?.path === selectedPath.value)?.id ?? 'global'
)
const deleteBlast = computed(() => {
  const open = file.value?.summary
  if (!open) return ''
  const index = open.kind === 'memory' ? ' · its line leaves MEMORY.md' : ''
  return `deletes ~/.claude/${open.path} · not recoverable${index}`
})

function select(path: string): void {
  if (path === selectedPath.value) return
  guard(() => void run(() => store.open(path)))
}

function chooseSection(next: ResourceSection): void {
  section.value = next
}

async function save(): Promise<void> {
  if (!dirty.value || !file.value?.summary.editable) return
  const saved = await run(() => store.save())
  if (saved) toast.notify('Saved.')
}

async function create(path: string, body: string): Promise<void> {
  const made = await run(() => store.create(path, body))
  if (made !== null) creating.value = false
}

async function remove(): Promise<void> {
  const open = file.value
  confirmingDelete.value = false
  if (!open) return
  await run(() => store.remove(open.summary.path), 'Deleted.')
}

function onKeydown(event: KeyboardEvent): void {
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 's') {
    event.preventDefault()
    void save()
  }
}

onBeforeRouteLeave(() => askToLeave())

onMounted(() => {
  window.addEventListener('keydown', onKeydown)
  void run(() => store.load())
})
onUnmounted(() => window.removeEventListener('keydown', onKeydown))
</script>

<template>
  <div class="relative flex min-h-0 flex-1 flex-col bg-canvas">
    <AppCatalogHeader title="Claude">
      <template #title-suffix>
        <span v-if="catalog" class="hidden truncate font-mono text-caption text-text-subtle lg:block" :title="catalog.root">{{ catalog.root }}</span>
      </template>
    </AppCatalogHeader>

    <main class="flex min-h-0 flex-1" aria-label="Claude resources">
      <ResourceRail
        :catalog="catalog"
        :section="section"
        :query="query"
        :selected-path="selectedPath"
        :projects="console_.projects"
        @update:section="chooseSection"
        @update:query="query = $event"
        @select="select"
        @create="creating = true"
      />

      <Transition name="pane" mode="out-in">
        <ResourceEditor
          v-if="file"
          :key="file.summary.path"
          :file="file"
          :draft="draft"
          :dirty="dirty"
          :saving="saving"
          :conflict="conflict"
          @update:draft="draft = $event"
          @save="save"
          @revert="store.revert()"
          @remove="confirmingDelete = true"
          @reload="run(() => store.reloadFromDisk())"
          @overwrite="run(() => store.overwrite(), 'Overwritten.')"
        />
        <div v-else class="grid min-w-0 flex-1 place-items-center p-8" data-testid="resource-empty">
          <AppEmptyState
            :title="loaded ? 'Pick a file to read or edit' : 'Reading the Claude folder'"
            description="Instructions, memories, skills, commands and agents are plain markdown files. Changes are written straight to disk, where your next session reads them."
          />
        </div>
      </Transition>
    </main>

    <Transition name="dialog">
      <NewResourceDialog
        v-if="creating && catalog"
        :section="section"
        :groups="catalog.memoryGroups"
        :initial-group-id="groupOfOpenFile"
        :sending="isRunning"
        @cancel="creating = false"
        @create="create"
      />
    </Transition>
    <Transition name="dialog">
      <AppConfirm
        v-if="confirmingDelete && file"
        :title="`Delete ${file.summary.name}?`"
        body="Claude will no longer find it in new sessions. There is no copy to restore."
        :blast="deleteBlast"
        confirm-label="Delete file"
        @cancel="confirmingDelete = false"
        @confirm="remove"
      />
    </Transition>
    <Transition name="dialog">
      <AppConfirm
        v-if="asking && file"
        title="Discard your edits?"
        :body="`${file.summary.name} has changes that are not saved.`"
        blast="the unsaved text is lost"
        confirm-label="Discard"
        @cancel="cancel"
        @confirm="confirm"
      />
    </Transition>
  </div>
</template>
