import type { DurationFormatOptions, DurationInput } from '@formatjs/intl-durationformat/src/types';

export type TemporaryDurationInput = DurationInput;
export type TemporaryDurationFormatOptions = DurationFormatOptions;

export interface FormatConfigType {
  date: Record<string, Intl.DateTimeFormatOptions>;
  duration: Record<string, TemporaryDurationFormatOptions>;
  list: Record<string, Intl.ListFormatOptions>;
  number: Record<string, Intl.NumberFormatOptions>;
  relativeTime: Record<string, Intl.RelativeTimeFormatOptions>;
  time: Record<string, Intl.DateTimeFormatOptions>;
}

export function resolveFormatConfigOptions<T extends object, const K extends string>(
  config: Record<K, T>,
  style?: T & { format?: K },
): T {
  if (typeof style?.format === 'string') {
    return {
      ...config[style.format],
      ...style,
    };
  }
  return style;
}

export const DEFAULT_FORMAT_CONFIG = {
  // These have no known defaults, so they can't be applied easily.
  duration: {},
  list: {},
  relativeTime: {},

  /**
   * Default formatting configuration options for common date, time, and number formats. This is
   * taken almost directly from FormatJS's defaults here, for the sake of compatibility.
   * https://github.com/formatjs/formatjs/blob/c30975bfbe2db7eb62f4dbe6c8ad6ca5e786dcb3/packages/intl-messageformat/src/core.ts#L229-L296
   */
  number: {
    integer: { maximumFractionDigits: 0 },
    currency: { style: 'currency' },
    percent: { style: 'percent' },
  },

  date: {
    short: {
      month: 'numeric',
      day: 'numeric',
      year: '2-digit',
    },

    medium: {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    },

    long: {
      month: 'long',
      day: 'numeric',
      year: 'numeric',
    },

    full: {
      weekday: 'long',
      month: 'long',
      day: 'numeric',
      year: 'numeric',
    },
  },

  time: {
    short: {
      hour: 'numeric',
      minute: 'numeric',
    },

    medium: {
      hour: 'numeric',
      minute: 'numeric',
      second: 'numeric',
    },

    long: {
      hour: 'numeric',
      minute: 'numeric',
      second: 'numeric',
      timeZoneName: 'short',
    },

    full: {
      hour: 'numeric',
      minute: 'numeric',
      second: 'numeric',
      timeZoneName: 'short',
    },
  },
} satisfies FormatConfigType;
