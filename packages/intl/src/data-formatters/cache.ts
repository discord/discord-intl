import type { DurationFormat as FormatJsDurationFormat } from '@formatjs/intl-durationformat';

type Cache<Value> = Map<string, Value>;

type TemporaryIntlDurationFormat = typeof FormatJsDurationFormat;

class FormatterCache {
  dateTime: Cache<Intl.DateTimeFormat> = new Map();
  duration: Cache<TemporaryIntlDurationFormat> = new Map();
  list: Cache<Intl.ListFormat> = new Map();
  number: Cache<Intl.NumberFormat> = new Map();
  pluralRules: Cache<Intl.PluralRules> = new Map();
  relativeTime: Cache<Intl.RelativeTimeFormat> = new Map();

  getDateTimeFormatter(...args: ConstructorParameters<typeof Intl.DateTimeFormat>) {
    return this._getCached(this.dateTime, args, (args) => new Intl.DateTimeFormat(...args));
  }

  getDurationFormatter(...args: ConstructorParameters<TemporaryIntlDurationFormat>) {
    return this._getCached(
      this.duration,
      args,
      // @ts-expect-error DurationFormat is _not_ included in typescript
      // https://github.com/microsoft/TypeScript/issues/60608
      (args) => new Intl.DurationFormat(...args),
    );
  }

  getListFormatter(...args: ConstructorParameters<typeof Intl.ListFormat>) {
    return this._getCached(this.list, args, (args) => new Intl.ListFormat(...args));
  }

  getNumberFormatter(...args: ConstructorParameters<typeof Intl.NumberFormat>) {
    return this._getCached(this.number, args, (args) => new Intl.NumberFormat(...args));
  }

  getPluralRules(...args: ConstructorParameters<typeof Intl.PluralRules>) {
    return this._getCached(this.pluralRules, args, (args) => new Intl.PluralRules(...args));
  }

  getRelativeTimeFormatter(...args: ConstructorParameters<typeof Intl.RelativeTimeFormat>) {
    return this._getCached(this.relativeTime, args, (args) => new Intl.RelativeTimeFormat(...args));
  }

  _getCached<T, Args>(cache: Cache<T>, args: Args, constructor: (args: Args) => T): T {
    const key = this._getKey(args);
    const cached = cache.get(key);
    if (cached) return cached;

    const created = constructor(args);
    cache.set(key, created);
    return created;
  }

  _getKey(...args: any): string {
    return JSON.stringify(args);
  }
}

export const dataFormatterCache = new FormatterCache();
