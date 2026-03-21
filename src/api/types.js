/**
 * @typedef {Object} Deck
 * @property {number} id
 * @property {string} name
 * @property {Record<string, unknown>} commander
 * @property {unknown[]} cards
 * @property {number} mana_value
 * @property {number} mana_pips
 * @property {number} black_pips
 * @property {number} blue_pips
 * @property {number} white_pips
 * @property {number} green_pips
 * @property {number} red_pips
 */

/**
 * JS representation of Rust `Card` from `card_model.rs`.
 * Enum vectors are serialized as string arrays by serde.
 *
 * @typedef {Object} Card
 * @property {number} id
 * @property {string} image
 * @property {string} name
 * @property {string | null} mana_cost
 * @property {number} mana_value
 * @property {string[]} card_type
 * @property {string[]} super_type
 * @property {string[]} sub_type
 * @property {string | null} oracle_text
 */

export {};
