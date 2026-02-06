import { ref, type Ref } from "vue";
import type { Terminal } from "@xterm/xterm";
import type { FitAddon } from "@xterm/addon-fit";

/**
 * Terminal instance managed by the registry
 */
import type { FlowController } from "./FlowController";

/**
 * Terminal instance managed by the registry
 */
export interface ManagedTerminal {
  id: string;
  container: HTMLDivElement;
  term: Terminal | null;
  fitAddon: FitAddon | null;
  flowController: FlowController | null;
  mountedTo: HTMLElement | null;
  onDataDisposable: { dispose: () => void } | null;
}

/**
 * Terminal Registry Singleton
 *
 * Manages terminal DOM nodes lifecycle to prevent re-rendering issues.
 * Terminals are created once in a hidden host and teleported to visible panels.
 */
class TerminalRegistryClass {
  private static instance: TerminalRegistryClass | null = null;

  private readonly terminals: Map<string, ManagedTerminal> = new Map();
  private hostElement: HTMLElement | null = null;

  /** Reactive list of registered terminal IDs */
  public readonly terminalIds: Ref<string[]> = ref([]);

  private constructor() {}

  /**
   * Get singleton instance
   */
  public static getInstance(): TerminalRegistryClass {
    TerminalRegistryClass.instance ??= new TerminalRegistryClass();
    return TerminalRegistryClass.instance;
  }

  /**
   * Set the hidden host element where terminals live when not mounted
   */
  public setHostElement(element: HTMLElement): void {
    this.hostElement = element;
  }

  /**
   * Get the hidden host element
   */
  public getHostElement(): HTMLElement | null {
    return this.hostElement;
  }

  /**
   * Register a terminal with its container element
   */
  public registerTerminal(
    id: string,
    container: HTMLDivElement,
    term: Terminal | null = null,
    fitAddon: FitAddon | null = null,
    flowController: FlowController | null = null
  ): void {
    if (this.terminals.has(id)) {
      // Update existing terminal
      const existing = this.terminals.get(id)!;
      existing.term = term;
      existing.fitAddon = fitAddon;
      existing.flowController = flowController;
      return;
    }

    const managed: ManagedTerminal = {
      id,
      container,
      term,
      fitAddon,
      flowController,
      mountedTo: null,
      onDataDisposable: null,
    };

    this.terminals.set(id, managed);
    this.terminalIds.value = Array.from(this.terminals.keys());

    // Initially place in hidden host
    if (this.hostElement && container.parentElement !== this.hostElement) {
      this.hostElement.appendChild(container);
    }
  }

  /**
   * Update terminal instance after xterm initialization
   */
  public updateTerminalInstance(
    id: string,
    term: Terminal,
    fitAddon: FitAddon,
    flowController: FlowController
  ): void {
    const managed = this.terminals.get(id);
    if (managed) {
      managed.term = term;
      managed.fitAddon = fitAddon;
      managed.flowController = flowController;
    }
  }

  /**
   * Mount terminal to a visible panel container
   */
  public mountToPanel(terminalId: string, targetElement: HTMLElement): boolean {
    const managed = this.terminals.get(terminalId);
    if (!managed) {
      console.warn(`[TerminalRegistry] Terminal ${terminalId} not found`);
      return false;
    }

    // Already mounted to this target
    if (managed.mountedTo === targetElement) {
      return true;
    }

    // Move container to target
    targetElement.appendChild(managed.container);
    managed.mountedTo = targetElement;

    // Trigger resize after DOM move
    requestAnimationFrame(() => {
      if (managed.fitAddon && managed.term) {
        managed.fitAddon.fit();
      }
    });

    return true;
  }

  /**
   * Unmount terminal from panel back to hidden host
   */
  public unmountFromPanel(terminalId: string): void {
    const managed = this.terminals.get(terminalId);
    if (!managed?.mountedTo) return;

    // Move back to hidden host
    if (this.hostElement) {
      this.hostElement.appendChild(managed.container);
    }
    managed.mountedTo = null;
  }

  /**
   * Get managed terminal by ID
   */
  public getTerminal(id: string): ManagedTerminal | undefined {
    return this.terminals.get(id);
  }

  /**
   * Check if terminal exists
   */
  public hasTerminal(id: string): boolean {
    return this.terminals.has(id);
  }

  /**
   * Completely destroy and remove a terminal
   */
  public destroyTerminal(id: string): void {
    const managed = this.terminals.get(id);
    if (!managed) return;

    // Dispose input handler first
    if (managed.onDataDisposable) {
      managed.onDataDisposable.dispose();
      managed.onDataDisposable = null;
    }

    // Detach flow controller
    if (managed.flowController) {
      managed.flowController.detach();
      managed.flowController = null;
    }

    // Dispose xterm instance
    if (managed.term) {
      managed.term.dispose();
    }

    // Remove container from DOM
    managed.container.remove();

    // Remove from registry
    this.terminals.delete(id);
    this.terminalIds.value = Array.from(this.terminals.keys());
  }

  /**
   * Set input handler for a terminal, safely replacing any existing handler
   */
  public setInputHandler(
    id: string,
    handler: (data: string) => void
  ): void {
    const managed = this.terminals.get(id);
    if (!managed?.term) return;

    // Dispose existing handler first to prevent duplicates
    if (managed.onDataDisposable) {
      managed.onDataDisposable.dispose();
      managed.onDataDisposable = null;
    }

    // Attach new handler and store disposable
    managed.onDataDisposable = managed.term.onData(handler);
  }

  /**
   * Get all registered terminal IDs
   */
  public getAllTerminalIds(): string[] {
    return Array.from(this.terminals.keys());
  }

  /**
   * Trigger fit for a terminal (useful after panel resize)
   */
  public fitTerminal(id: string): void {
    const managed = this.terminals.get(id);
    if (managed?.fitAddon && managed.term) {
      managed.fitAddon.fit();
    }
  }

  /**
   * Clear all terminals (for cleanup/reset)
   */
  public clear(): void {
    for (const [id] of this.terminals) {
      this.destroyTerminal(id);
    }
  }
}

export const TerminalRegistry = TerminalRegistryClass.getInstance();
