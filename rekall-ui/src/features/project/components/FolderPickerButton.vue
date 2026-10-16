<script setup lang="ts">
import { ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import PathPicker from '@/components/ui/PathPicker.vue'
import type { CaretBox } from '@/composables/useCaretPlacement'

/**
 * The Browse button of a folder field. It opens the same path picker the markdown editor uses,
 * hung under the button and listing folders only, and hands back the folder that was chosen.
 */
defineOptions({ inheritAttrs: false })

const props = defineProps<{
  /** The project's folder: where the picker opens when the field holds nothing to start from. */
  root: string | null
  /** What the field holds now; the picker opens there. */
  current: string
}>()

const emit = defineEmits<{ pick: [folder: string] }>()

const open = ref(false)
const button = ref<InstanceType<typeof AppButton> | null>(null)
/** The folder the picker was last left in, so a second pick from the same place is one keystroke. */
const visited = ref<string | null>(null)

function locate(): CaretBox | null {
  const element = button.value?.$el as HTMLElement | undefined
  if (!element) return null
  const box = element.getBoundingClientRect()
  return { left: box.left, top: box.top, bottom: box.bottom }
}

function show(): void {
  visited.value = props.current.trim() || null
  open.value = true
}

function choose(folder: string): void {
  open.value = false
  emit('pick', folder)
}
</script>

<template>
  <AppButton ref="button" v-bind="$attrs" size="sm" variant="secondary" data-path-picker-trigger @click="open ? (open = false) : show()">
    Browse
  </AppButton>

  <Teleport to="body">
    <Transition name="popover">
      <PathPicker
        v-if="open"
        folders-only
        :root="root"
        :start="visited"
        :locate="locate"
        :format="(path: string) => path"
        @pick="choose"
        @visit="visited = $event"
        @close="open = false"
      />
    </Transition>
  </Teleport>
</template>
