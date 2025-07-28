<template>
  <div :class="['rounded-box border border-base-300 bg-base-100', 'transition-all', { 'shadow': open }]">
    <div class="flex items-center px-4 py-2 gap-2" tabindex="0">
      <!-- select-none -->
      <span class="font-mono text-sm flex-1">{{ path }}</span>
      <button class="btn btn-ghost btn-sm" :disabled="isOpening" @click.stop="onOpenFolder">
        <FolderOpen class="w-5 h-5" />
      </button>
      <button class="btn btn-ghost btn-sm p-0 min-w-0" @click="toggle" :aria-expanded="open" :aria-label="open ? 'Collapse' : 'Expand'">
        <ChevronRight :class="['w-5 h-5 transition-transform duration-200', open ? 'rotate-90' : '']" />
      </button>
    </div>
    <div v-show="open" class="px-4 pb-4">
      <slot></slot>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ChevronRight, FolderOpen } from 'lucide-vue-next'

const props = defineProps({
  path: { type: String, required: true },
  isOpening: { type: Boolean, default: false },
})
const emit = defineEmits(['open-folder'])

const open = ref(true)

function toggle() {
  open.value = !open.value
}
function onOpenFolder() {
  emit('open-folder', props.path)
}
</script>

<style scoped></style>
