<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition-opacity duration-300"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition-opacity duration-300"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="tourStore.isActive"
        class="fixed inset-0 z-9999 pointer-events-none"
      >
        <!-- Backdrop with spotlight cutout -->
        <div class="absolute inset-0 pointer-events-auto">
          <svg class="w-full h-full">
            <defs>
              <mask id="spotlight-mask">
                <rect x="0" y="0" width="100%" height="100%" fill="white" />
                <rect
                  v-if="spotlightRect && currentStep?.highlight"
                  :x="spotlightRect.x"
                  :y="spotlightRect.y"
                  :width="spotlightRect.width"
                  :height="spotlightRect.height"
                  :rx="8"
                  :ry="8"
                  fill="black"
                />
              </mask>
            </defs>
            <rect
              x="0"
              y="0"
              width="100%"
              height="100%"
              fill="rgba(0, 0, 0, 0.75)"
              mask="url(#spotlight-mask)"
            />
          </svg>
        </div>

        <!-- Spotlight border glow -->
        <div
          v-if="spotlightRect && currentStep?.highlight"
          class="absolute pointer-events-none transition-all duration-300 ease-out"
          :style="{
            left: `${spotlightRect.x - 2}px`,
            top: `${spotlightRect.y - 2}px`,
            width: `${spotlightRect.width + 4}px`,
            height: `${spotlightRect.height + 4}px`,
            borderRadius: '10px',
            boxShadow:
              '0 0 0 2px rgba(59, 130, 246, 0.5), 0 0 20px rgba(59, 130, 246, 0.3)',
          }"
        />

        <!-- Tooltip -->
        <TourTooltip
          v-if="currentStep"
          :step="currentStep"
          :step-index="tourStore.currentStepIndex"
          :total-steps="tourStore.totalSteps"
          :target-rect="targetRect"
          :is-first="tourStore.isFirstStep"
          :is-last="tourStore.isLastStep"
          :progress="tourStore.progress"
          @next="tourStore.nextStep"
          @prev="tourStore.prevStep"
          @skip="tourStore.skipTour"
          @complete="tourStore.completeTour"
        />
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { useTourStore } from "../../stores/tour";
import TourTooltip from "./TourTooltip.vue";

const tourStore = useTourStore();

const targetRect = ref<DOMRect | null>(null);
const spotlightRect = ref<{
  x: number;
  y: number;
  width: number;
  height: number;
} | null>(null);

const currentStep = computed(() => tourStore.currentStep);

/**
 * Find target element and calculate positions
 */
const updateTargetPosition = () => {
  if (!currentStep.value) {
    targetRect.value = null;
    spotlightRect.value = null;
    return;
  }

  const target = document.querySelector(currentStep.value.target);

  if (target) {
    const rect = target.getBoundingClientRect();
    targetRect.value = rect;

    const padding = currentStep.value.spotlightPadding ?? 8;
    spotlightRect.value = {
      x: rect.x - padding,
      y: rect.y - padding,
      width: rect.width + padding * 2,
      height: rect.height + padding * 2,
    };
  } else {
    // Center position for welcome/complete steps
    targetRect.value = null;
    spotlightRect.value = null;
  }
};

// Watch for step changes
watch(
  () => [tourStore.currentStepIndex, tourStore.isActive],
  () => {
    if (tourStore.isActive) {
      // Small delay to ensure DOM is ready
      setTimeout(updateTargetPosition, 50);
    }
  },
  { immediate: true },
);

// Handle window resize
const handleResize = () => {
  if (tourStore.isActive) {
    updateTargetPosition();
  }
};

onMounted(() => {
  window.addEventListener("resize", handleResize);
});

onUnmounted(() => {
  window.removeEventListener("resize", handleResize);
});
</script>
