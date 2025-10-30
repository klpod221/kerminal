declare module 'asciinema-player' {
  export interface PlayerOptions {
    cols?: number;
    rows?: number;
    autoPlay?: boolean;
    preload?: boolean;
    loop?: boolean | number;
    startAt?: number | string;
    speed?: number;
    idleTimeLimit?: number;
    theme?: string;
    poster?: string;
    fit?: string;
    fontSize?: string;
    terminalFontSize?: string;
    terminalFontFamily?: string;
    terminalLineHeight?: number;
  }

  export interface Player {
    dispose(): void;
    play(): void;
    pause(): void;
    seek(time: number): void;
    getCurrentTime(): number;
    getDuration(): number;
  }

  export function create(
    src: string | object,
    element: HTMLElement,
    options?: PlayerOptions
  ): Player;
}

