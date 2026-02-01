import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { Store } from "@tauri-apps/plugin-store";
import { TOUR_STEPS } from "../config/tourSteps";

// Tauri store instance
let store: Store | null = null;

// Storage key
const TOUR_STORE_KEY = "tour_state";

/**
 * Initialize Tauri store
 */
const initStore = async () => {
  store ??= await Store.load("tour.json");
  return store;
};

/**
 * Tour Store
 * Manages the first user tour state and progression
 */
export const useTourStore = defineStore("tour", () => {
  // State
  const isActive = ref(false);
  const currentStepIndex = ref(0);
  const hasCompletedFirstTour = ref(false);
  const isTransitioning = ref(false);
  const isLoading = ref(false);

  // Computed
  const currentStep = computed(() => TOUR_STEPS[currentStepIndex.value]);
  const totalSteps = computed(() => TOUR_STEPS.length);
  const isFirstStep = computed(() => currentStepIndex.value === 0);
  const isLastStep = computed(
    () => currentStepIndex.value === TOUR_STEPS.length - 1,
  );
  const progress = computed(
    () => ((currentStepIndex.value + 1) / TOUR_STEPS.length) * 100,
  );

  /**
   * Load tour state from storage
   */
  const loadState = async () => {
    isLoading.value = true;
    try {
      const tauriStore = await initStore();
      const savedState = await tauriStore.get<{ hasCompletedFirstTour: boolean }>(
        TOUR_STORE_KEY,
      );

      if (savedState) {
        hasCompletedFirstTour.value = savedState.hasCompletedFirstTour;
      }
    } catch (error) {
      console.error("Failed to load tour state:", error);
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Save tour state to storage
   */
  const saveState = async () => {
    try {
      const tauriStore = await initStore();
      await tauriStore.set(TOUR_STORE_KEY, {
        hasCompletedFirstTour: hasCompletedFirstTour.value,
      });
      await tauriStore.save();
    } catch (error) {
      console.error("Failed to save tour state:", error);
    }
  };

  /**
   * Start the tour
   */
  const startTour = async () => {
    if (isActive.value) return;

    currentStepIndex.value = 0;
    isActive.value = true;

    // Execute beforeShow if defined
    const step = TOUR_STEPS[0];
    if (step.beforeShow) {
      await step.beforeShow();
    }
  };

  /**
   * Go to next step
   */
  const nextStep = async () => {
    if (isTransitioning.value || !isActive.value) return;

    const currentStep = TOUR_STEPS[currentStepIndex.value];

    // Execute afterComplete for current step
    if (currentStep.afterComplete) {
      await currentStep.afterComplete();
    }

    if (currentStepIndex.value < TOUR_STEPS.length - 1) {
      isTransitioning.value = true;
      currentStepIndex.value++;

      // Execute beforeShow for next step
      const nextStepData = TOUR_STEPS[currentStepIndex.value];
      if (nextStepData.beforeShow) {
        await nextStepData.beforeShow();
      }

      setTimeout(() => {
        isTransitioning.value = false;
      }, 300);
    } else {
      // Last step - complete the tour
      await completeTour();
    }
  };

  /**
   * Go to previous step
   */
  const prevStep = async () => {
    if (isTransitioning.value || !isActive.value || currentStepIndex.value === 0)
      return;

    isTransitioning.value = true;
    currentStepIndex.value--;

    // Execute beforeShow for previous step
    const step = TOUR_STEPS[currentStepIndex.value];
    if (step.beforeShow) {
      await step.beforeShow();
    }

    setTimeout(() => {
      isTransitioning.value = false;
    }, 300);
  };

  /**
   * Go to specific step
   */
  const goToStep = async (index: number) => {
    if (
      isTransitioning.value ||
      !isActive.value ||
      index < 0 ||
      index >= TOUR_STEPS.length
    )
      return;

    isTransitioning.value = true;
    currentStepIndex.value = index;

    // Execute beforeShow for target step
    const step = TOUR_STEPS[index];
    if (step.beforeShow) {
      await step.beforeShow();
    }

    setTimeout(() => {
      isTransitioning.value = false;
    }, 300);
  };

  /**
   * End tour (used by both skip and complete)
   */
  const endTour = async () => {
    isActive.value = false;
    currentStepIndex.value = 0;
    hasCompletedFirstTour.value = true;
    await saveState();
  };

  /**
   * Skip the tour
   */
  const skipTour = () => endTour();

  /**
   * Complete the tour
   */
  const completeTour = () => endTour();

  /**
   * Reset tour state (for testing or re-showing tour)
   */
  const resetTour = async () => {
    hasCompletedFirstTour.value = false;
    currentStepIndex.value = 0;
    isActive.value = false;
    await saveState();
  };

  return {
    // State
    isActive,
    currentStepIndex,
    hasCompletedFirstTour,
    isTransitioning,
    isLoading,

    // Computed
    currentStep,
    totalSteps,
    isFirstStep,
    isLastStep,
    progress,

    // Actions
    loadState,
    saveState,
    startTour,
    nextStep,
    prevStep,
    goToStep,
    skipTour,
    completeTour,
    resetTour,
  };
});
