<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { useCatalogStore } from '@/stores/catalog.store'

/**
 * Keyboard entry point to everything, on the meta key and K.
 *
 * This is a tool for someone who lives in a terminal: reaching a task should not require three
 * clicks through a sidebar. The palette is the fastest path to any project, any screen, and it
 * costs nothing to ignore if you would rather use the mouse.
 */
type Command = Readonly<{ id: string; label: string; hint: string; to: string; group: string }>

const router = useRouter()
const catalog = useCatalogStore()
const { projects, environments } = storeToRefs(catalog)

const isOpen = ref(false)
const query = ref('')
const activeIndex = ref(0)
const input = ref<HTMLInputElement | null>(null)

const commands = computed<Command[]>(() => [
  { id: 'nav-projects', label: 'Projects', hint: 'All projects', to: '/projects', group: 'Go to' },
  { id: 'nav-environments', label: 'Environments', hint: 'Clusters and namespaces', to: '/environments', group: 'Go to' },
  { id: 'nav-search', label: 'Search', hint: 'Across every note', to: '/search', group: 'Go to' },
  // The hint is the anchor, so the palette doubles as a reminder of what to type after /rk.
  ...projects.value.map((project) => ({
    id: `project-${project.id}`,
    label: project.name,
    hint: `project:${project.name}`,
    to: `/projects/${project.id}`,
    group: 'Project'
  })),
  ...environments.value.map((environment) => ({
    id: `environment-${environment.id}`,
    label: environment.label,
    hint: `environment:${environment.label}`,
    to: '/environments',
    group: 'Environment'
  }))
])

const results = computed(() => {
  const needle = query.value.trim().toLowerCase()
  if (!needle) return commands.value
  return commands.value.filter(
    (command) =>
      command.label.toLowerCase().includes(needle) || command.hint.toLowerCase().includes(needle)
  )
})

const groups = computed(() => {
  const grouped = new Map<string, Command[]>()
  for (const command of results.value) {
    grouped.set(command.group, [...(grouped.get(command.group) ?? []), command])
  }
  return [...grouped.entries()]
})

watch(results, () => (activeIndex.value = 0))

function open(): void {
  isOpen.value = true
  query.value = ''
  activeIndex.value = 0
  nextTick(() => input.value?.focus())
}

function close(): void {
  isOpen.value = false
}

function run(command: Command | undefined): void {
  if (!command) return
  close()
  void router.push(command.to)
}

function onKeydown(event: KeyboardEvent): void {
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
    event.preventDefault()
    if (isOpen.value) {
      close()
    } else {
      open()
    }
    return
  }
  if (!isOpen.value) return

  if (event.key === 'Escape') {
    event.preventDefault()
    close()
  } else if (event.key === 'ArrowDown') {
    event.preventDefault()
    activeIndex.value = (activeIndex.value + 1) % Math.max(results.value.length, 1)
  } else if (event.key === 'ArrowUp') {
    event.preventDefault()
    activeIndex.value = (activeIndex.value - 1 + results.value.length) % Math.max(results.value.length, 1)
  } else if (event.key === 'Enter') {
    event.preventDefault()
    run(results.value[activeIndex.value])
  }
}

function indexOf(command: Command): number {
  return results.value.findIndex((candidate) => candidate.id === command.id)
}

onMounted(() => window.addEventListener('keydown', onKeydown))
onUnmounted(() => window.removeEventListener('keydown', onKeydown))

defineExpose({ open })
</script>

<template>
  <Transition
    enter-active-class="transition duration-150 ease-out"
    enter-from-class="opacity-0"
    leave-active-class="transition duration-100 ease-in"
    leave-to-class="opacity-0"
  >
    <div
      v-if="isOpen"
      class="fixed inset-0 z-(--z-modal) flex items-start justify-center bg-black/60 px-4 pt-[12vh] backdrop-blur-sm"
      role="dialog"
      aria-modal="true"
      aria-label="Command palette"
      @click.self="close"
    >
      <div
        class="w-full max-w-xl overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-lift"
      >
        <div class="flex items-center gap-3 border-b border-border px-4">
          <svg viewBox="0 0 24 24" fill="none" class="size-4 shrink-0 text-text-subtle" aria-hidden="true">
            <circle cx="11" cy="11" r="7" stroke="currentColor" stroke-width="2" />
            <path d="M16.5 16.5L21 21" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
          </svg>
          <input
            ref="input"
            v-model="query"
            class="h-12 flex-1 bg-transparent text-[14px] text-text outline-none placeholder:text-text-subtle"
            placeholder="Jump to a project, an environment, a screen"
            aria-label="Search commands"
          />
          <kbd class="rounded border border-border px-1.5 py-0.5 text-[10px] text-text-subtle">esc</kbd>
        </div>

        <div class="max-h-[52vh] overflow-y-auto p-2">
          <p v-if="!results.length" class="px-3 py-6 text-center text-[13px] text-text-subtle">
            Nothing matches "{{ query }}".
          </p>

          <div v-for="[group, items] in groups" :key="group" class="mb-1 last:mb-0">
            <p class="px-3 py-1.5 text-[10px] font-semibold uppercase tracking-[0.09em] text-text-subtle">
              {{ group }}
            </p>
            <button
              v-for="command in items"
              :key="command.id"
              class="flex w-full items-center gap-3 rounded-[var(--radius-control)] px-3 py-2 text-left transition-colors"
              :class="
                indexOf(command) === activeIndex
                  ? 'bg-accent-soft text-accent'
                  : 'text-text-muted hover:bg-surface-raised hover:text-text'
              "
              @click="run(command)"
              @mousemove="activeIndex = indexOf(command)"
            >
              <span class="text-[13px] font-medium">{{ command.label }}</span>
              <span class="truncate font-mono text-[11.5px] text-text-subtle">{{ command.hint }}</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>
