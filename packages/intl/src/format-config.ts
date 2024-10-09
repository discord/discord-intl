export interface FormatConfig {
  number: { [name: string]: Intl.NumberFormatOptions };
  time: { [name: string]: Intl.DateTimeFormatOptions };
  date: { [name: string]: Intl.DateTimeFormatOptions };
}

/**
 * Default formatting configuration options for common date, time, and number formats. This is
 * taken almost directly from FormatJS's defaults here, for the sake of compatibility.
 * https://github.com/formatjs/formatjs/blob/c30975bfbe2db7eb62f4dbe6c8ad6ca5e786dcb3/packages/intl-messageformat/src/core.ts#L229-L296
 */
export const DEFAULT_FORMAT_CONFIG: FormatConfig = {
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
};
