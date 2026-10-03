globalThis.window = globalThis;
globalThis.navigator = { languages: ['en-US'] };

/**
 * @param {string} title
 * @param {() => unknown} callback
 * @param {boolean} log
 */
function bench(title, callback, log = true) {
  const start = performance.now();
  callback();
  const end = performance.now();
  if (log) {
    console.log(title + ': ', end - start);
  }
}

const locales = [
  'bg',
  'cs',
  'da',
  'de',
  'el',
  'en-GB',
  'es-419',
  'es-ES',
  'fi',
  'fr',
  'hi',
  'hr',
  'hu',
  'id',
  'it',
  'ja',
  'ko',
  'lt',
  'nl',
  'no',
  'pl',
  'pt-BR',
  'ro',
  'ru',
  'sv-SE',
  'th',
  'tr',
  'uk',
  'vi',
  'zh-CN',
  'zh-TW',
];

module.exports = {
  bench,
  locales,
};
