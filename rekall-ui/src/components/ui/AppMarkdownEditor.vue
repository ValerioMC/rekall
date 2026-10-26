<script setup lang="ts">
import { computed, onBeforeUnmount, ref, useId, watch } from 'vue'
import { MdEditor, MdPreview, NormalToolbar, type ExposeParam, type ToolbarNames } from 'md-editor-v3'
import '@/common/config/markdown-editor'
import PathPicker from '@/components/ui/PathPicker.vue'
import ReadModeSettings from '@/components/ui/ReadModeSettings.vue'
import ReadSearchBar from '@/components/ui/ReadSearchBar.vue'
import TableContextMenu from '@/components/ui/TableContextMenu.vue'
import { componentCompletion } from '@/common/markdown/component-completion'
import { connectionCompletion } from '@/common/markdown/connection-completion'
import { useComponentReferenceChips } from '@/features/project/composables/useComponentReferenceChips'
import { useConnectionReferenceChips } from '@/features/connection/composables/useConnectionReferenceChips'
import { useEditorPathPicker } from '@/composables/useEditorPathPicker'
import { useFittedEditorHeight } from '@/composables/useFittedEditorHeight'
import { useReadModeSearch } from '@/composables/useReadModeSearch'
import { useTableContextMenu } from '@/composables/useTableContextMenu'
import type { ProjectComponent } from '@/features/project/model/project-component'
import type { ProjectConnection } from '@/features/connection/model/connection'
import { readModeAlign, readModeWidth, setReadModeAlign, setReadModeWidth } from '@/common/config/read-mode'

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
    /**
     * The folder the path picker opens in: the project's, when the text belongs to one. Without
     * it the picker opens in the home folder, and it can reach anywhere on the machine either way.
     */
    pathRoot?: string | null
    /**
     * The project's components, when the text belongs to one: a `[[Name]]` in the read view is
     * drawn as a chip for the component, or as a broken reference. `null` (a note with no
     * project) leaves every reference as plain text. Write mode shows the source as written and
     * offers these names after `[[`.
     */
    components?: readonly ProjectComponent[] | null
    /** The project's connection bindings, for `[[connection:label]]` the way `components` serves `[[Name]]`. */
    connections?: readonly ProjectConnection[] | null
  }>(),
  {
    readonly: false,
    height: '420px',
    grow: false,
    placeholder: '# Contesto',
    compact: false,
    pathRoot: null,
    components: null,
    connections: null
  }
)

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()

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

const tableMenu = useTableContextMenu(editor)

const componentChips = useComponentReferenceChips(root, () => props.components)
const connectionChips = useConnectionReferenceChips(root, () => props.connections)

function paintChips(): void {
  componentChips.paint()
  connectionChips.paint()
}

// One source each for the editor's life; they read the props when asked, so a new list needs no reconfigure.
const completions = [componentCompletion(() => props.components), connectionCompletion(() => props.connections)]

// A card-sized read view has no room for a bar; ⌘F there is left to the page.
const search = useReadModeSearch(root, { enabled: () => props.readonly && !props.compact })

// Unique per instance: the library keys its global event bus on this id, and two previews sharing one clear each other's entries on unmount.
const editorId = `rekall-md-${useId()}`

// The reading column's width and alignment, shared across every Read-mode pane via localStorage.
const readWidth = ref(readModeWidth())
const readAlign = ref(readModeAlign())
watch(readWidth, (value) => setReadModeWidth(value))
watch(readAlign, (value) => setReadModeAlign(value))
const readStyle = computed(() => ({ '--rk-read-width': `${readWidth.value}px` }))

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
</script>

<template>
  <div
    ref="root"
    class="rekall-md"
    :class="{ 'rekall-md--readonly': readonly, 'rekall-md--compact': readonly && compact }"
    :style="readonly && !compact ? readStyle : undefined"
    :data-align="readonly && !compact ? readAlign : undefined"
    @keydown.capture="onEditorKeydown"
    @rk-table-menu="tableMenu.open"
  >
    <div v-if="readonly && !compact" class="rekall-read-controls">
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
        <ReadModeSettings v-model:width="readWidth" v-model:align="readAlign" />
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
      :toolbars="TOOLBARS"
      :completions="completions"
      :preview="false"
      :footers="['markdownTotal']"
      :style="{ height: editorHeight }"
      :placeholder="placeholder"
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
          data-path-picker-trigger
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
      <TableContextMenu
        v-if="tableMenu.request.value"
        :request="tableMenu.request.value"
        @choose="tableMenu.choose"
        @close="tableMenu.close"
      />
    </Teleport>
  </div>
</template>
