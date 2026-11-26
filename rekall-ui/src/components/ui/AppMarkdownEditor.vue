<script setup lang="ts">
import { computed, onBeforeUnmount, ref, useId } from 'vue'
import { MdEditor, MdPreview, NormalToolbar, type ExposeParam, type ToolbarNames } from 'md-editor-v3'
import '@/common/config/markdown-editor'
import PathPicker from '@/components/ui/PathPicker.vue'
import TaskWidthMenu from '@/features/task/components/TaskWidthMenu.vue'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'
import { columnMaxWidth } from '@/features/task/model/task-width'
import ReadSearchBar from '@/components/ui/ReadSearchBar.vue'
import TableContextMenu from '@/components/ui/TableContextMenu.vue'
import { componentCompletion } from '@/common/markdown/component-completion'
import { paintComponentReferences } from '@/common/markdown/component-reference'
import { connectionCompletion } from '@/common/markdown/connection-completion'
import { paintConnectionReferences } from '@/common/markdown/connection-reference'
import { taskCompletion } from '@/common/markdown/task-completion'
import { pageCompletion } from '@/common/markdown/page-completion'
import { PAGE_CHIP_CLASS, paintPageReferences, type ReferencePage } from '@/common/markdown/page-reference'
import { paintTaskReferences } from '@/common/markdown/task-reference'
import { useReferenceChips } from '@/composables/useReferenceChips'
import { useEditorPathPicker } from '@/composables/useEditorPathPicker'
import { usePictureInsert } from '@/features/picture/composables/usePictureInsert'
import { withFolderSlash } from '@/common/path/path-query'
import type { PictureOwner } from '@/features/picture/model/picture'
import { useFittedEditorHeight } from '@/composables/useFittedEditorHeight'
import { useMermaidFences } from '@/composables/useMermaidFences'
import { useReadModeSearch } from '@/composables/useReadModeSearch'
import { useReferenceWidgets } from '@/composables/useReferenceWidgets'
import { useTableContextMenu } from '@/composables/useTableContextMenu'
import type { ProjectComponent } from '@/features/project/model/project-component'
import type { ProjectConnection } from '@/features/connection/model/connection'
import type { Task } from '@/features/catalog/model/catalog'

const props = withDefaults(
  defineProps<{
    modelValue: string
    readonly?: boolean
    height?: string
    /** Size the writing surface to its text, with `height` as the least it takes, instead of holding `height` and scrolling inside. */
    grow?: boolean
    placeholder?: string
    /**
     * Read inside something else (a step's detail under its title) rather than as the pane: the
     * page sits flush left at the container's width with no reading-column controls, since the
     * width and alignment they set belong to a pane-sized page, not to a card.
     */
    compact?: boolean
    /** Puts the caret in the writing surface as it opens, for an editor the reader has just asked to write in. */
    autofocus?: boolean
    /** Read as a message in a log: `compact`, and also without the indent that the section ticks hang into. */
    bare?: boolean
    /** Offers the `Aa` key for the reading column's width; off where the pane sets the page width itself. */
    readSettings?: boolean
    /** Whether the toolbar offers the page full screen, for text whose pane already gives it the room it needs. */
    pageFullscreen?: boolean
    /**
     * The folder the path picker opens in: the project's, when the text belongs to one. Without
     * it the picker opens in the home folder, and it can reach anywhere on the machine either way.
     */
    pathRoot?: string | null
    /**
     * The record the text belongs to, for pictures: with one, the toolbar's image button picks a
     * file and copies it into that record's folder. Without one the button writes bare markdown.
     */
    pictureOwner?: PictureOwner | null
    /**
     * The project's components, when the text belongs to one: a `[[Name]]` in the read view is
     * drawn as a chip for the component, or as a broken reference. `null` (a note with no
     * project) leaves every reference as plain text. Write mode draws the same chip, with its glyph,
     * until the caret enters it, and offers these names after `[[`.
     */
    components?: readonly ProjectComponent[] | null
    /** The project's connection bindings, for `[[connection:label]]` the way `components` serves `[[Name]]`. */
    connections?: readonly ProjectConnection[] | null
    /** The project's other tasks, for `[[task:label]]`; offered only once `[[task:` is typed. */
    tasks?: readonly Task[] | null
    /** The project's pages, for `[[page:Title]]`; offered only once `[[page:` is typed. A click on one in the read view asks to open it. */
    pages?: readonly ReferencePage[] | null
  }>(),
  {
    readonly: false,
    height: '420px',
    grow: false,
    placeholder: '# Contesto',
    compact: false,
    bare: false,
    autofocus: false,
    readSettings: true,
    pageFullscreen: true,
    pathRoot: null,
    pictureOwner: null,
    components: null,
    connections: null,
    tasks: null,
    pages: null
  }
)

const emit = defineEmits<{ 'update:modelValue': [value: string]; openPage: [title: string] }>()

const body = computed({
  get: () => props.modelValue,
  set: (value: string) => emit('update:modelValue', value)
})

const editor = ref<ExposeParam | null>(null)
const root = ref<HTMLElement | null>(null)

// md-editor-v3 6.5.6 never destroys its CodeMirror view on unmount; without this each pane swap leaks the detached editor DOM.
onBeforeUnmount(() => editor.value?.getEditorView()?.destroy())

const { editorHeight, refit } = useFittedEditorHeight(editor, root, { grow: () => props.grow, height: () => props.height })
const {
  open: pickerOpen,
  start: pickerStart,
  toggle: togglePicker,
  close: closePicker,
  locateCaret,
  format: formatPick,
  insert: insertPath,
  onKeydown: onEditorKeydown
} = useEditorPathPicker(editor, { readonly: () => props.readonly, pathRoot: () => props.pathRoot })

const {
  open: picturePickerOpen,
  start: picturePickerStart,
  toggle: togglePicturePicker,
  close: closePicturePicker,
  locateCaret: locatePictureCaret,
  visit: visitPictureFolder,
  insert: insertPicture
} = usePictureInsert(editor, {
  readonly: () => props.readonly,
  pathRoot: () => props.pathRoot,
  owner: () => props.pictureOwner
})

const tableMenu = useTableContextMenu(editor)

const componentChips = useReferenceChips(root, () => props.components, paintComponentReferences)
const connectionChips = useReferenceChips(root, () => props.connections, paintConnectionReferences)
const taskChips = useReferenceChips(root, () => props.tasks, paintTaskReferences)
const pageChips = useReferenceChips(root, () => props.pages, paintPageReferences)
const mermaidFences = useMermaidFences(root)

useReferenceWidgets(editor, () => ({
  components: props.components,
  connections: props.connections,
  tasks: props.tasks,
  pages: props.pages
}))

function paintChips(): void {
  componentChips.paint()
  connectionChips.paint()
  taskChips.paint()
  pageChips.paint()
  mermaidFences.paint()
}

// One source each for the editor's life; they read the props when asked, so a new list needs no reconfigure.
const completions = [
  componentCompletion(() => props.components),
  connectionCompletion(() => props.connections),
  taskCompletion(() => props.tasks),
  pageCompletion(() => props.pages)
]

// A card-sized read view has no room for a bar; ⌘F there is left to the page.
const flush = computed(() => props.compact || props.bare)

function openPageClicked(event: MouseEvent): void {
  if (!props.readonly || !(event.target instanceof Element)) return
  const chip = event.target.closest<HTMLElement>(`.${PAGE_CHIP_CLASS}[data-state='found']`)
  if (chip?.dataset['page']) emit('openPage', chip.dataset['page'])
}

const search = useReadModeSearch(root, { enabled: () => props.readonly && !flush.value })

// Unique per instance: the library keys its global event bus on this id, and two previews sharing one clear each other's entries on unmount.
const editorId = `rekall-md-${useId()}`

// The reading column's width and alignment, shared across every Read-mode pane via localStorage.
const preferences = usePreferencesStore()
const readStyle = computed(() => ({ '--rk-read-width': columnMaxWidth(preferences.taskWidth) }))

const TOOLBARS: ToolbarNames[] = [
  'bold',
  'italic',
  'strikeThrough',
  '-',
  'title',
  'quote',
  'unorderedList',
  'orderedList',
  'task',
  '-',
  'codeRow',
  'code',
  'link',
  0,
  'image',
  'table',
  '-',
  'revoke',
  'next',
  '=',
  'pageFullscreen',
  'catalog'
]

const takesPictures = computed(() => props.pictureOwner !== null)

// A number stands for the button at that place in the `defToolbars` slot.
const toolbars = computed(() =>
  TOOLBARS.filter((name) => props.pageFullscreen || name !== 'pageFullscreen').map((name) =>
    name === 'image' && takesPictures.value ? 1 : name
  )
)
</script>

<template>
  <div
    ref="root"
    class="rekall-md"
    :class="{
      'rekall-md--readonly': readonly,
      'rekall-md--compact': readonly && flush,
      'rekall-md--bare': readonly && bare
    }"
    :style="readonly && !flush ? readStyle : undefined"
    @keydown.capture="onEditorKeydown"
    @click="openPageClicked"
    @rk-table-menu="tableMenu.open"
  >
    <div v-if="readonly && !flush" class="rekall-read-controls">
      <div class="flex items-start gap-2">
        <ReadSearchBar
          v-if="search.open.value"
          v-model="search.query.value"
          :label="search.label.value"
          :count="search.count.value"
          :focus-requests="search.focusRequests.value"
          @next="search.next"
          @previous="search.previous"
          @close="search.close"
        />
        <TaskWidthMenu v-if="readSettings" v-model="preferences.taskWidth" />
      </div>
    </div>
    <MdPreview
      v-if="readonly"
      :model-value="modelValue"
      :editor-id="editorId"
      language="en-US"
      theme="dark"
      preview-theme="github"
      code-theme="atom"
      no-katex
      no-mermaid
      no-echarts
      no-img-zoom-in
      @on-html-changed="paintChips"
    />
    <MdEditor
      v-else
      ref="editor"
      v-model="body"
      :editor-id="editorId"
      language="en-US"
      theme="dark"
      preview-theme="github"
      code-theme="atom"
      :toolbars="toolbars"
      :completions="completions"
      :preview="false"
      :footers="['markdownTotal']"
      :style="{ height: editorHeight }"
      :placeholder="placeholder"
      :auto-focus="autofocus"
      no-katex
      no-mermaid
      no-echarts
      no-prettier
      no-upload-img
      @on-change="refit"
    >
      <template #defToolbars>
        <NormalToolbar
          title="Insert a path  ⌘⇧P"
          aria-label="Insert a path"
          :aria-expanded="pickerOpen"
          data-path-picker-trigger="path"
          data-testid="path-picker-trigger"
          :class="{ 'md-editor-toolbar-active': pickerOpen }"
          @on-click="togglePicker"
        >
          <!-- A folder with the slash of a path through it: this puts a path, not a file, in the text. -->
          <svg class="md-editor-icon" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path
              d="M3 6.5A1.5 1.5 0 0 1 4.5 5h4.4l2 2.2h8.6A1.5 1.5 0 0 1 21 8.7v9.8a1.5 1.5 0 0 1-1.5 1.5h-15A1.5 1.5 0 0 1 3 18.5v-12Z"
              stroke="currentColor"
              stroke-width="1.6"
              stroke-linejoin="round"
            />
            <path d="m13.6 10.4-3.2 6.2" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
          </svg>
        </NormalToolbar>
        <NormalToolbar
          title="Insert a picture"
          aria-label="Insert a picture"
          :aria-expanded="picturePickerOpen"
          data-path-picker-trigger="image"
          data-testid="picture-picker-trigger"
          :class="{ 'md-editor-toolbar-active': picturePickerOpen }"
          @on-click="togglePicturePicker"
        >
          <svg class="md-editor-icon" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <rect x="3" y="4.5" width="18" height="15" rx="2" stroke="currentColor" stroke-width="1.6" />
            <circle cx="8.6" cy="9.6" r="1.6" stroke="currentColor" stroke-width="1.6" />
            <path d="m4.5 18.3 5.2-4.7 3.5 3.1 3.6-4 3.8 4.2" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </NormalToolbar>
      </template>
    </MdEditor>

    <Teleport to="body">
      <Transition name="popover">
        <PathPicker
          v-if="pickerOpen"
          :root="pathRoot"
          :start="pickerStart"
          :locate="locateCaret"
          :format="formatPick"
          @pick="insertPath"
          @visit="pickerStart = $event"
          @close="closePicker"
        />
      </Transition>
      <Transition name="popover">
        <PathPicker
          v-if="picturePickerOpen"
          mode="image"
          :root="pathRoot"
          :start="picturePickerStart"
          :locate="locatePictureCaret"
          :format="withFolderSlash"
          @pick="insertPicture"
          @visit="visitPictureFolder"
          @close="closePicturePicker"
        />
      </Transition>
      <TableContextMenu
        v-if="tableMenu.request.value"
        :request="tableMenu.request.value"
        @choose="tableMenu.choose"
        @close="tableMenu.close"
      />
    </Teleport>
  </div>
</template>
