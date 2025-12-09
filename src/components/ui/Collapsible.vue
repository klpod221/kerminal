<template>
  <div class="w-full">
    <!-- Header -->
    <button
      type="button"
      class="group w-full flex items-center justify-between px-2 py-1.5 bg-transparent border-none cursor-pointer transition-all duration-200 ease-in-out hover:bg-gray-800/40 rounded-md focus:outline-none"
      :class="{ 'bg-gray-800/20': isExpanded }"
      @click="toggle"
    >
      <div class="flex items-center space-x-2 min-w-0 flex-1">
        <div
          class="flex items-center justify-center text-gray-500 collapsible-icon shrink-0 transition-transform duration-200"
          :class="{ 'rotate-90 text-gray-400': isExpanded }"
        >
          <ChevronRight :size="14" />
        </div>
        <h3 class="text-sm font-medium text-gray-300 truncate">
          {{ title }}
        </h3>
        <span v-if="subtitle" class="text-xs text-gray-500 truncate">
          {{ subtitle }}
        </span>
      </div>
      <div class="flex items-center gap-2 shrink-0">
        <div @click.stop>
          <slot name="headerActions" />
        </div>
        <div
          v-if="badge"
          class="px-1.5 py-0.5 text-[11px] bg-gray-700/50 text-gray-400 rounded-md font-medium"
        >
          {{ badge }}
        </div>
      </div>
    </button>

    <!-- Content -->
    <Transition
      name="collapsible"
      @enter="onEnter"
      @after-enter="onAfterEnter"
      @leave="onLeave"
      @after-leave="onAfterLeave"
    >
      <div v-show="isExpanded" ref="contentRef" class="collapsible-content">
        <div class="pt-2 ml-2 pl-3 border-l border-gray-700/50">
          <slot />
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from "vue";
import { ChevronRight } from "lucide-vue-next";

interface Props {
  title: string;
  subtitle?: string;
  badge?: string;
  defaultExpanded?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  defaultExpanded: false,
});

const isExpanded = ref(props.defaultExpanded);
// const contentRef = ref<HTMLElement>();

const toggle = () => {
  isExpanded.value = !isExpanded.value;
};

const onEnter = (el: Element) => {
  const element = el as HTMLElement;
  element.style.height = "0";
  element.style.overflow = "hidden";
  nextTick(() => {
    element.style.height = element.scrollHeight + "px";
  });
};

const onAfterEnter = (el: Element) => {
  const element = el as HTMLElement;
  element.style.height = "";
  element.style.overflow = "";
};

const onLeave = (el: Element) => {
  const element = el as HTMLElement;
  element.style.height = element.scrollHeight + "px";
  element.style.overflow = "hidden";
  nextTick(() => {
    element.style.height = "0";
  });
};

const onAfterLeave = (el: Element) => {
  const element = el as HTMLElement;
  element.style.height = "";
};

defineExpose({
  toggle,
  isExpanded,
});
</script>

<style scoped>
.collapsible-content {
  overflow-x: visible;
}

/* Transition classes */
.collapsible-enter-active,
.collapsible-leave-active {
  transition: height 0.25s ease-in-out;
}

.collapsible-enter-from,
.collapsible-leave-to {
  height: 0;
}
</style>
