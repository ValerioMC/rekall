<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useContextMenuDismissal } from '@/composables/useContextMenuDismissal'
import PictureAlignmentGlyph from '@/features/picture/components/PictureAlignmentGlyph.vue'
import { PICTURE_ALIGNMENTS, PICTURE_ALIGNMENT_LABEL, alignmentOf } from '@/features/picture/lib/picture-alignment'
import type { PictureAlignment } from '@/features/picture/model/picture'
import type { PictureMenuRequest } from '@/common/live-preview/image-widget'

/** The right-click menu over a drawn picture: how it is aligned, with the current one marked. */
const props = defineProps<{ request: PictureMenuRequest }>()

const emit = defineEmits<{ choose: [alignment: PictureAlignment]; close: [] }>()

const menu = ref<HTMLElement | null>(null)
const current = computed(() => alignmentOf(props.request.address))

// Kept inside the window: a menu opened near the right or bottom edge flips back over the click.
const MENU_WIDTH = 200
const MENU_HEIGHT = 96
const left = Math.min(props.request.x, window.innerWidth - MENU_WIDTH - 8)
const top = Math.min(props.request.y, window.innerHeight - MENU_HEIGHT - 8)

useContextMenuDismissal(menu, () => emit('close'))

onMounted(() => menu.value?.querySelector<HTMLElement>('[aria-checked="true"]')?.focus())
</script>

<template>
  <div
    ref="menu"
    role="menu"
    aria-label="Picture"
    class="fixed z-(--z-overlay) w-[200px] rounded-card border border-border-strong bg-surface-raised p-1 shadow-lift"
    :style="{ left: `${left}px`, top: `${top}px` }"
    data-testid="picture-context-menu"
  >
    <button
      v-for="alignment in PICTURE_ALIGNMENTS"
      :key="alignment"
      type="button"
      role="menuitemradio"
      :aria-checked="alignment === current"
      class="focus-ring flex h-7 w-full cursor-pointer items-center gap-2 rounded-control px-2 text-left text-small transition-colors duration-(--motion-quick) hover:bg-surface-hover hover:text-text"
      :class="alignment === current ? 'text-text' : 'text-text-muted'"
      :data-testid="`picture-menu-${alignment}`"
      @click="emit('choose', alignment)"
    >
      <PictureAlignmentGlyph :alignment="alignment" />
      Align {{ PICTURE_ALIGNMENT_LABEL[alignment].toLowerCase() }}
      <svg v-if="alignment === current" class="ml-auto size-3.5 text-accent" viewBox="0 0 14 14" fill="none" aria-hidden="true">
        <path d="m3 7.5 2.7 2.7L11 4.2" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </button>
  </div>
</template>
