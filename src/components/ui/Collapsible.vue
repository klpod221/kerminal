<template>
  <div class="w-full">
    <!-- Header -->
    <button
      type="button"
      class="w-full flex items-center justify-between p-2 bg-transparent border-none cursor-pointer transition-all duration-200 ease-in-out hover:bg-gray-700/30 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500/50"
      :class="{ expanded: isExpanded }"
      @click="toggle"
    >
      <div class="flex items-center space-x-3 min-w-0 flex-1">
        <div
          class="flex items-center justify-center text-gray-400 collapsible-icon flex-shrink-0"
          :class="{ rotated: isExpanded }"
        >
          <ChevronRight :size="16" />
        </div>
        <h3 class="text-sm font-medium text-gray-300 truncate">
          {{ title }}
        </h3>
        <span v-if="subtitle" class="text-xs text-gray-500 truncate">
          {{ subtitle }}
        </span>
      </div>
      <div
        v-if="badge"
        class="px-2 py-1 text-xs bg-gray-700 text-gray-300 rounded-full flex-shrink-0 ml-2"
      >
        {{ badge }}
      </div>
    </button>

    <!-- Border line -->
    <div
      class="w-full h-px opacity-60 collapsible-border"
      :class="{ expanded: isExpanded }"
    ></div>

    <!-- Content -->
    <Transition
      name="collapsible"
      @enter="onEnter"
      @after-enter="onAfterEnter"
      @leave="onLeave"
      @after-leave="onAfterLeave"
    >
      <div v-show="isExpanded" ref="contentRef" class="collapsible-content">
        <div class="pt-4 ml-4 pl-4 border-l-2 border-gray-600/50">
          <slot />
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from "vue";
import { ChevronRight } from "lucide-vue-next";

// Props
interface Props {
  title: string;
  subtitle?: string;
  badge?: string;
  defaultExpanded?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  defaultExpanded: false,
});

// State
const isExpanded = ref(props.defaultExpanded);
const contentRef = ref<HTMLElement>();

// Methods
const toggle = () => {
  isExpanded.value = !isExpanded.value;
};

// Animation handlers
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

// Expose methods
defineExpose({
  toggle,
  isExpanded,
});
</script>

<style scoped>
.collapsible-icon {
  transition: transform 0.2s ease-in-out;
}

.collapsible-icon.rotated {
  transform: rotate(90deg);
}

.collapsible-border {
  background: linear-gradient(
    to right,
    transparent,
    rgb(75, 85, 99),
    transparent
  );
  transition: all 0.3s ease-in-out;
}

.collapsible-border.expanded {
  background: linear-gradient(
    to right,
    transparent,
    rgba(59, 130, 246, 0.5),
    transparent
  );
}

.collapsible-content {
  overflow: hidden;
}

/* Transition classes */
.collapsible-enter-active,
.collapsible-leave-active {
  transition: height 0.3s ease-in-out;
}

.collapsible-enter-from,
.collapsible-leave-to {
  height: 0;
}
</style>
