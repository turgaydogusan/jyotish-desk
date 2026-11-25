// Sıralı, debounce'lu otomatik kayıt (C5): uçuşta en çok bir yazım; bekleyen değer yalnızca
// sonuncusudur. `flush` bekleyeni hemen yazar (pencere kapanırken, sayfa gizlenirken; C17).
import type { CommandError } from "../bindings";

export interface Saver<T> {
  schedule(value: T): void;
  flush(): void;
}

export interface SaverOptions<T> {
  debounceMs: number;
  onError(error: CommandError): void;
  onSaved(value: T): void;
}

export function createSaver<T>(write: (value: T) => Promise<CommandError | null>, opts: SaverOptions<T>): Saver<T> {
  let pending: { value: T } | null = null;
  let timer: ReturnType<typeof setTimeout> | null = null;
  let flying = false;

  const pump = async (): Promise<void> => {
    if (flying || pending === null) return;
    const { value } = pending;
    pending = null;
    flying = true;
    try {
      const error = await write(value);
      if (error) opts.onError(error);
      else opts.onSaved(value);
    } finally {
      flying = false;
    }
    // Uçuştayken gelen değer: zamanlayıcı bitmişse hemen yazılır.
    if (pending !== null && timer === null) await pump();
  };

  const flush = () => {
    if (timer !== null) {
      clearTimeout(timer);
      timer = null;
    }
    void pump();
  };

  return {
    schedule(value) {
      pending = { value };
      if (timer !== null) clearTimeout(timer);
      timer = setTimeout(() => {
        timer = null;
        void pump();
      }, opts.debounceMs);
    },
    flush,
  };
}
