const { IntlMessagesDatabase } = require('@discord/intl-message-database');

/**
 * A shared message database instance that's used and shared across all parts
 * of the loader and plugin together.
 */
const database = new IntlMessagesDatabase();

module.exports = { database };
