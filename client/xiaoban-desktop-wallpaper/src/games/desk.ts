import { GameSession, type GameManifest } from './protocol.ts';

/** What the native host puts on the desktop-layer window before the page starts. */
export interface DeskInit {
  manifest: GameManifest;
  screen: { width: number; height: number };
  capabilities: string[];
}

export interface DeskIo {
  /** Calls a native command with this window's host token already attached. */
  invoke(command: string, args?: Record<string, unknown>): Promise<unknown>;
  /** Sends a message to the game frame. */
  post(message: Record<string, unknown>): void;
}

const HOST = { channel: 'xiaban.game-host', protocol: 1 } as const;

/**
 * The trusted side of one desktop-layer game. The game lives in a sandboxed frame; this class is the only thing
 * that talks to it, and it passes along only what the game's manifest asked for.
 */
export class DeskHost {
  readonly session: GameSession;
  private saving: Promise<unknown> = Promise.resolve();

  constructor(
    private readonly init: DeskInit,
    readonly nonce: string,
    source: Window,
    private readonly io: DeskIo,
  ) {
    this.session = new GameSession(init.manifest.id, nonce, source);
  }

  /** The handshake: the game replies with `game.ready` once it has read this. */
  start(save: Record<string, unknown> | null) {
    this.io.post({
      ...HOST, type: 'host.init', nonce: this.nonce, save, safeArea: { top: 0, bottom: 0, left: 0, right: 0 },
      layer: 'desktop', capabilities: this.init.capabilities, screen: this.init.screen,
    });
  }

  /** One reading of the screen from the native side. Fields the game did not ask for are never forwarded. */
  frame(detail: unknown) {
    if (!this.session.ready || !detail || typeof detail !== 'object') return;
    const reading = detail as Record<string, unknown>;
    const message: Record<string, unknown> = { ...HOST, type: 'host.frame', nonce: this.nonce };
    if (this.init.capabilities.includes('windows') && Array.isArray(reading.windows)) message.windows = reading.windows;
    if (this.init.capabilities.includes('pointer') && reading.pointer && typeof reading.pointer === 'object') message.pointer = reading.pointer;
    if (message.windows === undefined && message.pointer === undefined) return;
    this.io.post(message);
  }

  async handle(event: Pick<MessageEvent, 'source' | 'data'>) {
    const message = this.session.accept(event);
    if (!message) return;
    if (message.type === 'game.regions') {
      await this.io.invoke('desk_set_regions', { rects: message.rects });
    } else if (message.type === 'game.save' && this.init.manifest.save) {
      const data = message.data;
      this.saving = this.saving.catch(() => undefined).then(() => this.io.invoke('save_game_save', { gameId: this.init.manifest.id, data }));
      await this.saving.catch(() => undefined);
    } else if (message.type === 'game.exit') {
      await this.stop();
    }
  }

  /** Lets a save in flight finish, then ends the game. */
  async stop() {
    await this.saving.catch(() => undefined);
    await this.io.invoke('close_desktop_game');
  }
}
