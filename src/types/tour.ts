/**
 * Tour step position relative to target element
 */
export type TourStepPosition = "top" | "bottom" | "left" | "right" | "center";

/**
 * Individual tour step configuration
 */
export interface TourStep {
  /** Unique identifier for the step */
  id: string;

  /** CSS selector or element ID for the target element */
  target: string;

  /** Title of the step */
  title: string;

  /** Description/content of the step */
  description: string;

  /** Position of the tooltip relative to target */
  position: TourStepPosition;

  /** Optional: Action to perform before showing this step */
  beforeShow?: () => void | Promise<void>;

  /** Optional: Action to perform after this step is completed */
  afterComplete?: () => void | Promise<void>;

  /** Optional: Whether to highlight the target element */
  highlight?: boolean;

  /** Optional: Custom spotlight padding around target */
  spotlightPadding?: number;
}

/**
 * Tour state for the store
 */
export interface TourState {
  /** Whether the tour is currently active */
  isActive: boolean;

  /** Current step index (0-based) */
  currentStepIndex: number;

  /** Whether the user has completed the first tour */
  hasCompletedFirstTour: boolean;

  /** Whether the tour is currently transitioning between steps */
  isTransitioning: boolean;
}

/**
 * Tour store actions
 */
export interface TourActions {
  startTour: () => void;
  nextStep: () => void;
  prevStep: () => void;
  goToStep: (index: number) => void;
  skipTour: () => void;
  completeTour: () => void;
}
