import { Logger, LoggerAdapter } from '@squadsync/shared';
import { info, warn, error, debug } from '@tauri-apps/plugin-log';

const formatArgs = (args: unknown[]) => {
  if (args.length === 0) return '';
  return args
    .map((arg) => {
      if (typeof arg === 'object') {
        try {
          return JSON.stringify(arg);
        } catch {
          return String(arg);
        }
      }
      return String(arg);
    })
    .join(' ');
};

const tauriAdapter: LoggerAdapter = {
  debug: (msg, ...args) => debug(`${msg} ${formatArgs(args)}`),
  info: (msg, ...args) => info(`${msg} ${formatArgs(args)}`),
  warn: (msg, ...args) => warn(`${msg} ${formatArgs(args)}`),
  error: (msg, ...args) => error(`${msg} ${formatArgs(args)}`),
};

export const logger = new Logger({
  service: 'Desktop',
  adapter: tauriAdapter,
  format: false, // Tauri plugin handles timestamp/level
});
