export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3,
}

export type LogFn = (message: string, ...args: unknown[]) => void;

export interface LoggerAdapter {
  debug: LogFn;
  info: LogFn;
  warn: LogFn;
  error: LogFn;
}

const consoleAdapter: LoggerAdapter = {
  // eslint-disable-next-line no-console
  debug: (msg, ...args) => console.debug(msg, ...args),
  info: (msg, ...args) => console.info(msg, ...args),
  warn: (msg, ...args) => console.warn(msg, ...args),
  error: (msg, ...args) => console.error(msg, ...args),
};

export interface LoggerOptions {
  service: string;
  level?: LogLevel;
  adapter?: LoggerAdapter;
  format?: boolean;
}

export class Logger {
  private service: string;
  private level: LogLevel;
  private adapter: LoggerAdapter;
  private shouldFormat: boolean;

  constructor(options: LoggerOptions) {
    this.service = options.service;
    this.level = options.level ?? LogLevel.INFO;
    this.adapter = options.adapter ?? consoleAdapter;
    this.shouldFormat = options.format ?? true;
  }

  private format(level: string, message: string): string {
    if (!this.shouldFormat) return message;
    const timestamp = new Date().toISOString();
    return `[${timestamp}] [${this.service}] [${level}] ${message}`;
  }

  debug(message: string, ...args: unknown[]) {
    if (this.level <= LogLevel.DEBUG) {
      this.adapter.debug(this.format('DEBUG', message), ...args);
    }
  }

  info(message: string, ...args: unknown[]) {
    if (this.level <= LogLevel.INFO) {
      this.adapter.info(this.format('INFO', message), ...args);
    }
  }

  warn(message: string, ...args: unknown[]) {
    if (this.level <= LogLevel.WARN) {
      this.adapter.warn(this.format('WARN', message), ...args);
    }
  }

  error(message: string, ...args: unknown[]) {
    if (this.level <= LogLevel.ERROR) {
      this.adapter.error(this.format('ERROR', message), ...args);
    }
  }
}
