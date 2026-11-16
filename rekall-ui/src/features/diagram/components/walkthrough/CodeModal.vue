<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, toRef, watch } from 'vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import BeatBadge from '@/features/diagram/components/walkthrough/BeatBadge.vue'
import CodeFileView from '@/features/diagram/components/walkthrough/CodeFileView.vue'
import WalkthroughEnds from '@/features/diagram/components/walkthrough/WalkthroughEnds.vue'
import { useSourceFile } from '@/features/diagram/composables/useSourceFile'
import { baseOf, folderOf, spanLabel } from '@/features/diagram/lib/source-label'
import { filesOfWalkthrough, firstBeatInFile, linkedMethods, sourcesOf } from '@/features/diagram/lib/walkthrough/code-links'
import type { WalkthroughBeat } from '@/features/diagram/lib/walkthrough/walkthrough-beats'
import type { CodeLinks } from '@/features/diagram/model/diagram'
import type { DiagramId } from '@/model/branded'

/**
 * The implementation behind the current step, in a sheet beside the diagram: every file the
 * walkthrough touches as a tab, the whole file open on the lines of the step. Clicking a method
 * that has a step of its own moves the walkthrough there.
 */
const props = defineProps<{
  diagramId: DiagramId
  beats: readonly WalkthroughBeat[]
  position: number
  links: CodeLinks
  labels: ReadonlyMap<string, string>
  external: ReadonlySet<string>
}>()

const emit = defineEmits<{ close: []; walk: [index: number] }>()

const closeKey = ref<HTMLButtonElement | null>(null)
const chosenFile = ref<string | null>(null)
let opener: Element | null = null

const beat = computed(() => props.beats[props.position]!)
const sources = computed(() => sourcesOf(props.links, beat.value))
const files = computed(() => filesOfWalkthrough(props.beats, props.links))
const file = computed(() => chosenFile.value ?? sources.value[0]?.file ?? null)
const span = computed(() => sources.value.find((source) => source.file === file.value) ?? null)
const methods = computed(() => linkedMethods(props.beats, props.links, props.position))
const stepCounts = computed(() => new Map(files.value.map((name) => [name, props.beats.filter((candidate) => sourcesOf(props.links, candidate).some((source) => source.file === name)).length])))

const { excerpt, failure } = useSourceFile(toRef(props, 'diagramId'), file)

watch(
  () => props.position,
  () => {
    chosenFile.value = null
  }
)

function openTab(name: string): void {
  if (sources.value.some((source) => source.file === name)) {
    chosenFile.value = name
    return
  }
  emit('walk', firstBeatInFile(props.beats, props.links, name))
}

onMounted(() => {
  opener = document.activeElement
  closeKey.value?.focus()
})
onBeforeUnmount(() => {
  if (opener instanceof HTMLElement) opener.focus({ preventScroll: true })
})
</script>

<template>
  <AppDialog label="Implementation" placement="right" scrim="light" panel-class="sm:max-w-[940px]" panel-testid="code-modal" @dismiss="emit('close')">
    <header class="flex h-12 shrink-0 items-center gap-2.5 border-b border-border pl-4 pr-2">
      <h2 class="text-title font-semibold text-text">Implementation</h2>
      <span class="font-mono text-caption tabular-nums text-text-subtle">Step {{ position + 1 }} of {{ beats.length }}</span>
      <span class="flex-1" />
      <button class="focus-ring grid size-7 place-items-center rounded-control text-text-muted transition-colors hover:bg-surface-raised hover:text-text disabled:opacity-40" aria-label="Previous step" :disabled="position === 0" data-testid="code-previous" @click="emit('walk', position - 1)">
        <svg viewBox="0 0 16 16" class="size-4" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m10 3.5-4.5 4.5 4.5 4.5" /></svg>
      </button>
      <button class="focus-ring grid size-7 place-items-center rounded-control text-text-muted transition-colors hover:bg-surface-raised hover:text-text disabled:opacity-40" aria-label="Next step" :disabled="position === beats.length - 1" data-testid="code-next" @click="emit('walk', position + 1)">
        <svg viewBox="0 0 16 16" class="size-4" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m6 3.5 4.5 4.5L6 12.5" /></svg>
      </button>
      <button ref="closeKey" class="focus-ring inline-flex h-7 items-center gap-2 rounded-control pl-2.5 pr-2 text-small text-text-muted transition-colors hover:bg-surface-raised hover:text-text" aria-label="Close the code" data-testid="code-close" @click="emit('close')">
        Close <kbd>Esc</kbd>
      </button>
    </header>

    <div class="shrink-0 border-b border-border px-4 pb-3 pt-2.5">
      <div class="flex min-w-0 items-center gap-2.5">
        <BeatBadge :motion="beat.motion" />
        <h3 class="min-w-0 flex-1 truncate font-mono text-body font-semibold text-text">{{ beat.message.text || '(no text)' }}</h3>
        <WalkthroughEnds class="shrink-0" :message="beat.message" :labels="labels" :external="external" />
      </div>
      <p v-if="beat.explain" class="mt-1.5 line-clamp-2 text-small leading-normal text-text-muted">{{ beat.explain }}</p>
    </div>

    <div class="flex h-9 shrink-0 items-stretch overflow-hidden border-b border-border bg-surface" role="tablist" aria-label="Files of the walkthrough">
      <button
        v-for="name in files"
        :key="name"
        role="tab"
        class="focus-ring relative flex items-center gap-2 border-r border-border px-3.5 font-mono text-small whitespace-nowrap transition-colors"
        :class="name === file ? 'bg-canvas text-text tab-current' : 'text-text-subtle hover:text-text'"
        :aria-selected="name === file"
        :title="name"
        data-testid="code-tab"
        @click="openTab(name)"
      >
        {{ baseOf(name) }}
        <span class="min-w-[18px] rounded-full border border-border-strong px-1.5 text-center font-mono text-micro tabular-nums" :class="name === file && 'border-accent/35 text-accent-strong'">{{ stepCounts.get(name) }}</span>
      </button>
    </div>

    <div v-if="file !== null" class="flex h-8 shrink-0 items-center gap-1.5 border-b border-border bg-canvas px-3.5 font-mono text-caption text-text-subtle" data-testid="code-crumbs">
      <span class="min-w-0 truncate">{{ folderOf(file) }}</span>
      <span class="shrink-0 text-text-muted">{{ baseOf(file) }}</span>
      <template v-if="span?.symbol">
        <span class="shrink-0 text-border-strong" aria-hidden="true">›</span>
        <span class="shrink-0 font-medium text-text">{{ span.symbol }}</span>
      </template>
      <span v-if="span" class="ml-auto shrink-0 pl-3 tabular-nums text-accent-strong">{{ spanLabel(span) }}</span>
    </div>

    <p v-if="file === null" class="m-4 rounded-control border border-dashed border-border px-3 py-2.5 text-small leading-relaxed text-text-subtle" data-testid="code-unlinked">This step is not linked to a place in the code.</p>
    <p v-else-if="failure" class="m-4 text-small leading-relaxed text-text-muted" role="status" data-testid="code-failure">{{ failure }}</p>
    <div v-else-if="!excerpt" class="flex flex-col gap-2 bg-canvas px-4 py-4" role="status" aria-busy="true">
      <span class="sr-only">Loading the code</span>
      <span v-for="index in 8" :key="index" class="skeleton h-3 rounded-key" :style="{ width: `${30 + ((index * 23) % 55)}%` }" />
    </div>
    <CodeFileView v-else :excerpt="excerpt" :start="span?.startLine ?? null" :end="span?.endLine ?? null" :methods="methods" :position="position" @walk="emit('walk', $event)" />
  </AppDialog>
</template>

<style scoped>
.tab-current::after {
  content: '';
  position: absolute;
  inset-inline: 0;
  bottom: -1px;
  height: 2px;
  background: var(--color-accent);
}
</style>
