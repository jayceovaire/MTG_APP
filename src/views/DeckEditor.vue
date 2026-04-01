<script setup>
import { computed, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { convertFileSrc } from "@tauri-apps/api/core";
import {
  mdiAlertCircleOutline,
  mdiCardsOutline,
  mdiCrownOutline,
  mdiPackageVariantClosedPlus,
  mdiPlus,
  mdiDownload,
  mdiExport,
  mdiGaugeFull,
  mdiCancel,
  mdiChevronLeft,
} from "@mdi/js";
import {
  addCardToDeckCommand,
  bulkAddCardsToDeckCommand,
  addCardToPackageCommand,
  addPackageToDeckCommand,
  createPackageCommand,
  deleteDeckCommanderCommand,
  getDeckCommand,
  getPackagesCommand,
  removeDeckCommanderCommand,
  removeCardFromDeckCommand,
  searchCardSuggestionsCommand,
  setDeckCommanderCommand,
  setDeckPartnerCommand,
  removeDeckPartnerCommand,
} from "../api/deckCommands.js";
import { getBase64ImagesCommand } from "../api/imageCommands.js";
import DeckCardRow from "../components/DeckCardRow.vue";
import ManaText from "../components/ManaText.vue";

const props = defineProps({
  deckId: {
    type: [String, Number],
    required: true,
  },
});

const router = useRouter();

const deck = ref(null);
const isLoading = ref(false);
const isUpdatingDeck = ref(false);
const loadError = ref("");
const searchName = ref("");
const cardSuggestions = ref([]);
const isSearchingCards = ref(false);
const isSearchFocused = ref(false);
const activeSuggestionIndex = ref(-1);
const isAddingCard = ref(false);
const snackbarVisible = ref(false);
const snackbarMessage = ref("");
const snackbarColor = ref("success");
const packages = ref([]);
const isLoadingPackages = ref(false);
const packageDialogVisible = ref(false);
const packageDialogMode = ref("card");
const selectedPackageId = ref(null);
const newPackageName = ref("");
const pendingPackageCardName = ref("");
const isSubmittingPackageAction = ref(false);
const isCreatingPackage = ref(false);
const testHand = ref([]);
// Import deck dialog state
const importDialogVisible = ref(false);
const importText = ref("");
const importErrors = ref([]);
const isImporting = ref(false);
const isPasting = ref(false);
let suggestionSearchTimeout = null;
let searchBlurTimeout = null;

const typeDisplayOrder = [
  "Creature",
  "Artifact",
  "Enchantment",
  "Instant",
  "Sorcery",
  "Planeswalker",
  "Battle",
  "Land",
  "Other",
];

function normalizeDeckId(value) {
  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed > 0 ? parsed : null;
}

function goBack() {
  router.back();
}

async function loadDeck() {
  const normalizedDeckId = normalizeDeckId(props.deckId);

  if (normalizedDeckId === null) {
    deck.value = null;
    loadError.value = "Invalid deck id.";
    return;
  }

  isLoading.value = true;
  loadError.value = "";

  try {
    deck.value = await getDeckCommand(normalizedDeckId);
    refreshTestHand();
  } catch (e) {
    deck.value = null;
    loadError.value = `Failed to load deck: ${String(e)}`;
    console.error(e);
  } finally {
    isLoading.value = false;
  }
}

async function loadPackages() {
  isLoadingPackages.value = true;

  try {
    packages.value = await getPackagesCommand();
  } catch (e) {
    showError(`Failed to load packages: ${String(e)}`);
    console.error(e);
  } finally {
    isLoadingPackages.value = false;
  }
}

function showSuccess(message) {
  snackbarMessage.value = message;
  snackbarColor.value = "success";
  snackbarVisible.value = true;
}

function showError(message) {
  snackbarMessage.value = message;
  snackbarColor.value = "error";
  snackbarVisible.value = true;
}

function closePackageDialog() {
  packageDialogVisible.value = false;
  selectedPackageId.value = null;
  newPackageName.value = "";
  pendingPackageCardName.value = "";
  isSubmittingPackageAction.value = false;
  isCreatingPackage.value = false;
}

function openImportDialog() {
  importDialogVisible.value = true;
  importText.value = "";
  importErrors.value = [];
}

function closeImportDialog() {
  importDialogVisible.value = false;
  importText.value = "";
  importErrors.value = [];
  isImporting.value = false;
  isPasting.value = false;
}

async function pasteFromClipboard() {
  try {
    isPasting.value = true;
    const text = await navigator.clipboard.readText();
    importText.value = text || "";
    importErrors.value = validateImportLines(importText.value);
  } catch (e) {
    showError("Failed to read clipboard.");
    console.error(e);
  } finally {
    isPasting.value = false;
  }
}

function validateImportLines(text) {
  const lines = text.split(/\r?\n/);
  const errors = [];
  for (let i = 0; i < lines.length; i++) {
    const raw = lines[i];
    const line = raw.trim();
    if (!line) continue;
    const m = line.match(/^(\d+)\s+(.+)$/);
    if (!m) {
      errors.push({ line: i + 1, text: raw });
    }
  }
  return errors;
}

async function handleImport() {
  importErrors.value = validateImportLines(importText.value);
  if (importErrors.value.length > 0) {
    showError("Fix import format errors before importing.");
    return;
  }

  const lines = importText.value.split(/\r?\n/).map((l) => l.trim()).filter((l) => l.length > 0);
  if (lines.length === 0) {
    showError("No cards to import.");
    return;
  }

  const normalizedDeckId = normalizeDeckId(props.deckId);
  if (normalizedDeckId === null) {
    showError("Invalid deck id.");
    return;
  }

  try {
    isImporting.value = true;
    
    const cardData = [];
    for (const raw of lines) {
      const m = raw.match(/^(\d+)\s+(.+)$/);
      if (!m) continue;
      const qty = Number(m[1]);
      const name = m[2].trim();
      cardData.push([qty, name]);
    }

    if (cardData.length > 0) {
      deck.value = await bulkAddCardsToDeckCommand(normalizedDeckId, cardData);
    }

    showSuccess(`Imported ${lines.length} lines to ${deck.value?.name || 'deck'}`);
    closeImportDialog();
  } catch (e) {
    console.error(e);
    showError(`Failed to import deck: ${String(e)}`);
  } finally {
    isImporting.value = false;
  }
}

async function exportDeckAsText() {
  if (!deck.value) return;

  try {
    const lines = [];

    // Commander section
    const commanders = commanderSection.value;
    if (commanders && commanders.length > 0) {
      for (const entry of commanders) {
        lines.push(`${entry.quantity} ${entry.card.name}`);
      }
    }

    // Mainboard section
    const sections = mainDeckSections.value;
    if (sections && sections.length > 0) {
      for (const section of sections) {
        for (const entry of section.entries) {
          lines.push(`${entry.quantity} ${entry.card.name}`);
        }
      }
    }

    if (lines.length === 0) {
      showError("Deck is empty. Nothing to export.");
      return;
    }

    const text = lines.join("\n");
    await navigator.clipboard.writeText(text);
    showSuccess("Deck list copied to clipboard as plain text.");
  } catch (e) {
    console.error(e);
    showError("Failed to export deck.");
  }
}

function openAddCardToPackageDialog(cardName) {
  packageDialogMode.value = "card";
  pendingPackageCardName.value = cardName;
  selectedPackageId.value = packages.value[0]?.id ?? null;
  newPackageName.value = "";
  packageDialogVisible.value = true;
}

function openAddPackageToDeckDialog() {
  packageDialogMode.value = "deck";
  pendingPackageCardName.value = "";
  selectedPackageId.value = packages.value[0]?.id ?? null;
  newPackageName.value = "";
  packageDialogVisible.value = true;
}

function clearSuggestions() {
  cardSuggestions.value = [];
  activeSuggestionIndex.value = -1;
}

function hideSuggestions() {
  isSearchFocused.value = false;
  clearSuggestions();
}

function handleSearchFocus() {
  if (searchBlurTimeout) {
    clearTimeout(searchBlurTimeout);
    searchBlurTimeout = null;
  }
  isSearchFocused.value = true;
}

function handleSearchBlur() {
  searchBlurTimeout = setTimeout(() => {
    hideSuggestions();
  }, 120);
}

function selectSuggestion(suggestion) {
  searchName.value = suggestion.name;
  isSearchFocused.value = false;
  clearSuggestions();
}

function moveSuggestion(direction) {
  if (cardSuggestions.value.length === 0) {
    return;
  }

  const nextIndex = activeSuggestionIndex.value + direction;
  if (nextIndex < 0) {
    activeSuggestionIndex.value = cardSuggestions.value.length - 1;
    return;
  }
  if (nextIndex >= cardSuggestions.value.length) {
    activeSuggestionIndex.value = 0;
    return;
  }
  activeSuggestionIndex.value = nextIndex;
}

async function handleSearchEnter() {
  if (cardSuggestions.value.length > 0 && activeSuggestionIndex.value >= 0) {
    selectSuggestion(cardSuggestions.value[activeSuggestionIndex.value]);
  }

  await handleAddCard();
}

/** Stable id match (avoids undefined===undefined treating every card as the commander). */
function sameCardId(a, b) {
  if (a == null || b == null) {
    return false;
  }
  const na = Number(a);
  const nb = Number(b);
  if (!Number.isFinite(na) || !Number.isFinite(nb)) {
    return false;
  }
  return na === nb;
}

function commanderCards(commander) {
  if (!commander) {
    return [];
  }

  if (typeof commander === "string") {
    return commander === "None" ? [] : [];
  }

  // Serde externally-tagged None: { "None": null }
  if (typeof commander === "object" && commander !== null && Object.prototype.hasOwnProperty.call(commander, "None")) {
    return [];
  }

  const single = commander.Single ?? commander.single;
  if (single) {
    return [single];
  }

  const partner = commander.Partner ?? commander.partner;
  if (Array.isArray(partner)) {
    return partner;
  }

  return [];
}

function primaryType(card) {
  const types = Array.isArray(card.card_type) ? card.card_type : [];
  return typeDisplayOrder.find((type) => types.includes(type)) || "Other";
}

function isCommanderCard(card) {
  if (card == null || card.id == null) {
    return false;
  }
  return commanderCards(deck.value?.commander).some((c) => c != null && sameCardId(c.id, card.id));
}

/** Rules only (used for current commander in canSetPartner, etc.). */
function canBeCommander(card) {
  const types = Array.isArray(card?.card_type) ? card.card_type : [];
  const superTypes = Array.isArray(card?.super_type) ? card.super_type : [];
  const subTypes = Array.isArray(card?.sub_type) ? card.sub_type : [];
  const oracleText = typeof card?.oracle_text === "string" ? card.oracle_text.toLowerCase() : "";

  const isLegendary = superTypes.includes("Legendary");
  const isCreature = types.includes("Creature");
  const isVehicle = subTypes.some((subtype) => subtype.toLowerCase() === "vehicle");
  const isBackground = subTypes.some((subtype) => subtype.toLowerCase() === "background");

  // Standard case: Legendary Creature or Legendary Vehicle
  if (isLegendary && (isCreature || isVehicle)) {
    return true;
  }

  // Special case: Backgrounds
  if (isLegendary && isBackground) {
    return true;
  }

  // Cards with "can be your commander" text (includes some planeswalkers)
  if (oracleText.includes("can be your commander")) {
    return true;
  }

  return false;
}

/** Whether the row may show "Set as Commander" (mainboard / not already in command zone). */
function canSetCommander(card) {
  if (isCommanderCard(card)) {
    return false;
  }
  return canBeCommander(card);
}

function hasPartnerMechanic(card) {
  const oracleText = typeof card?.oracle_text === "string" ? card.oracle_text : "";
  const subTypes = Array.isArray(card?.sub_type) ? card.sub_type : [];
  const lower = oracleText.toLowerCase();

  // Special case: Backgrounds
  if (subTypes.some((s) => s.toLowerCase() === "background")) {
    return true;
  }

  // Check each line for partner-like mechanics
  const hasPartnerLine = oracleText
    .split("\n")
    .map((line) => line.trim().toLowerCase())
    .some((line) =>
      line === "partner" ||
      line.startsWith("partner with ") ||
      line === "friends forever" ||
      line === "choose a background" ||
      line === "doctor's companion"
    );

  if (hasPartnerLine) {
    return true;
  }

  // Also check for "partner" as a standalone word anywhere in case it's formatted differently
  return lower.includes("partner") ||
         lower.includes("friends forever") ||
         lower.includes("choose a background") ||
         lower.includes("doctor's companion");
}

function canSetPartner(card) {
  if (isCommanderCard(card)) {
    return false;
  }
  if (!canBeCommander(card) || !hasPartnerMechanic(card)) {
    return false;
  }

  const commander = deck.value?.commander;
  if (!commander || commander === "None") {
    return false;
  }

  const single = commander.Single ?? commander.single;
  if (!single) {
    return false;
  }

  return hasPartnerMechanic(single) && canBeCommander(single);
}

/** Only for a card currently shown as partner commander in the command zone (not mainboard). */
function canRemovePartner(card) {
  const commander = deck.value?.commander;
  if (!commander || commander === "None") {
    return false;
  }
  const partner = commander.Partner ?? commander.partner;
  if (!Array.isArray(partner)) {
    return false;
  }
  if (card == null || card.id == null) {
    return false;
  }
  const isThisPartner = partner.some((partnerCard) => sameCardId(partnerCard?.id, card.id));
  if (!isThisPartner) {
    return false;
  }
  return canBeCommander(card) && hasPartnerMechanic(card);
}


function collapseCardCopies(cards) {
  const grouped = new Map();

  for (const card of cards) {
    const key = JSON.stringify([
      card.name,
      card.mana_cost || "",
      Array.isArray(card.card_type) ? card.card_type : [],
      Array.isArray(card.super_type) ? card.super_type : [],
      Array.isArray(card.sub_type) ? card.sub_type : [],
      card.oracle_text || "",
    ]);
    const entry = grouped.get(key);
    if (entry) {
      entry.quantity += 1;
      continue;
    }
    grouped.set(key, { card, quantity: 1 });
  }

  return [...grouped.values()].sort((a, b) => a.card.name.localeCompare(b.card.name));
}

const commanderSection = computed(() => collapseCardCopies(commanderCards(deck.value?.commander)));
const showCardSuggestions = computed(() => (
  isSearchFocused.value &&
  cardSuggestions.value.length > 0 &&
  searchName.value.trim().length >= 2
));
const allDeckCards = computed(() => {
  const mainboardCards = Array.isArray(deck.value?.cards) ? deck.value.cards : [];
  return [...mainboardCards, ...commanderCards(deck.value?.commander)];
});

const mainDeckSections = computed(() => {
  const cards = Array.isArray(deck.value?.cards) ? deck.value.cards : [];
  const groupedByType = new Map();

  for (const card of cards) {
    const type = primaryType(card);
    const bucket = groupedByType.get(type) || [];
    bucket.push(card);
    groupedByType.set(type, bucket);
  }

  return typeDisplayOrder
    .map((type) => ({
      title: type,
      entries: collapseCardCopies(groupedByType.get(type) || []),
    }))
    .filter((section) => section.entries.length > 0);
});

const deckCardTotal = computed(() => allDeckCards.value.length);
const avgManaValue = computed(() => {
  const cards = allDeckCards.value;
  if (cards.length === 0) {
    return 0;
  }

  const total = cards.reduce((sum, card) => sum + (Number(card.mana_value) || 0), 0);
  return (total / cards.length).toFixed(2);
});

const MANA_COLORS = {
  W: '#f8f6d8',
  U: '#c1d7e9',
  B: '#bab1ab',
  R: '#e49977',
  G: '#a3c095',
  grey: '#cac5c0'
};

const manaCurveDistribution = computed(() => {
  const cards = allDeckCards.value.filter(c => primaryType(c) !== 'Land');
  const dist = new Array(8).fill(0); // 0, 1, 2, 3, 4, 5, 6, 7+
  for (const card of cards) {
    const mv = Math.floor(Number(card.mana_value) || 0);
    if (mv >= 7) dist[7]++;
    else if (mv >= 0) dist[mv]++;
  }
  return dist;
});

const manaCurveColoredDistribution = computed(() => {
  const cards = allDeckCards.value.filter(c => primaryType(c) !== 'Land');
  const dist = Array.from({ length: 8 }, () => ({
    W: 0, U: 0, B: 0, R: 0, G: 0, grey: 0, total: 0
  }));

  for (const card of cards) {
    const mv = Math.floor(Number(card.mana_value) || 0);
    const index = mv >= 7 ? 7 : (mv >= 0 ? mv : 0);
    const manaCost = typeof card.mana_cost === "string" ? card.mana_cost : "";
    const symbols = manaCost.match(/\{[^}]+\}/g) || [];
    
    let w = 0, u = 0, b = 0, r = 0, g = 0;
    let coloredSymbolsCount = 0;
    
    for (const symbol of symbols) {
      let hasColor = false;
      if (symbol.includes('W')) { w++; hasColor = true; }
      if (symbol.includes('U')) { u++; hasColor = true; }
      if (symbol.includes('B')) { b++; hasColor = true; }
      if (symbol.includes('R')) { r++; hasColor = true; }
      if (symbol.includes('G')) { g++; hasColor = true; }
      if (hasColor) coloredSymbolsCount++;
    }
    
    const cardMv = Number(card.mana_value) || 0;
    const grey = Math.max(0, cardMv - coloredSymbolsCount);
    
    const totalUnits = w + u + b + r + g + grey;
    let cardW = 0, cardU = 0, cardB = 0, cardR = 0, cardG = 0, cardGrey = 0;
    
    if (totalUnits > 0) {
      cardW = w / totalUnits;
      cardU = u / totalUnits;
      cardB = b / totalUnits;
      cardR = r / totalUnits;
      cardG = g / totalUnits;
      cardGrey = grey / totalUnits;
    } else {
      cardGrey = 1;
    }
    
    dist[index].W += cardW;
    dist[index].U += cardU;
    dist[index].B += cardB;
    dist[index].R += cardR;
    dist[index].G += cardG;
    dist[index].grey += cardGrey;
    dist[index].total += 1;
  }
  
  return dist.map(d => {
    const segments = [
      { color: MANA_COLORS.W, value: d.W },
      { color: MANA_COLORS.U, value: d.U },
      { color: MANA_COLORS.B, value: d.B },
      { color: MANA_COLORS.R, value: d.R },
      { color: MANA_COLORS.G, value: d.G },
      { color: MANA_COLORS.grey, value: d.grey }
    ].filter(s => s.value > 0);
    
    return {
      total: d.total,
      segments
    };
  });
});

const manaCurveMode = ref('bar'); // 'bar' or 'line'
const pieChartMode = ref('pips-grey'); // 'pips-grey', 'combined'

const manaStats = computed(() => {
  const cards = allDeckCards.value;
  const totals = { W: 0, U: 0, B: 0, R: 0, G: 0, grey: 0, totalColoredSymbols: 0 };
  
  for (const card of cards) {
    const mv = Number(card.mana_value) || 0;
    const manaCost = typeof card.mana_cost === "string" ? card.mana_cost : "";
    
    // Improved pip counting to include hybrids/phyrexian
    // We count each symbol that contains a color letter as 1 colored symbol
    const symbols = manaCost.match(/\{[^}]+\}/g) || [];
    let cardColoredSymbols = 0;
    for (const symbol of symbols) {
      let isColored = false;
      if (symbol.includes('W')) { totals.W += 1; isColored = true; }
      if (symbol.includes('U')) { totals.U += 1; isColored = true; }
      if (symbol.includes('B')) { totals.B += 1; isColored = true; }
      if (symbol.includes('R')) { totals.R += 1; isColored = true; }
      if (symbol.includes('G')) { totals.G += 1; isColored = true; }
      
      if (isColored) {
          cardColoredSymbols++;
          totals.totalColoredSymbols++;
      }
    }
    
    // Grey mana for this card is MV - unique colored symbols
    totals.grey += Math.max(0, mv - cardColoredSymbols);
  }
  
  return totals;
});

const pieChartData = computed(() => {
  if (pieChartMode.value === 'pips-grey') {
    const stats = manaStats.value;
    return [
      { label: 'White', pip: '{W}', value: stats.W, color: MANA_COLORS.W },
      { label: 'Blue', pip: '{U}', value: stats.U, color: MANA_COLORS.U },
      { label: 'Black', pip: '{B}', value: stats.B, color: MANA_COLORS.B },
      { label: 'Red', pip: '{R}', value: stats.R, color: MANA_COLORS.R },
      { label: 'Green', pip: '{G}', value: stats.G, color: MANA_COLORS.G }
    ].filter(d => d.value > 0);
  } else {
    const stats = manaStats.value;
    return [
      { label: 'White', pip: '{W}', value: stats.W, color: MANA_COLORS.W },
      { label: 'Blue', pip: '{U}', value: stats.U, color: MANA_COLORS.U },
      { label: 'Black', pip: '{B}', value: stats.B, color: MANA_COLORS.B },
      { label: 'Red', pip: '{R}', value: stats.R, color: MANA_COLORS.R },
      { label: 'Green', pip: '{G}', value: stats.G, color: MANA_COLORS.G },
      { label: 'Grey', pip: '{C}', value: stats.grey, color: MANA_COLORS.grey }
    ].filter(d => d.value > 0);
  }
});

const manaCurveLegendItems = computed(() => [
  { label: 'White', pip: '{W}', color: MANA_COLORS.W },
  { label: 'Blue', pip: '{U}', color: MANA_COLORS.U },
  { label: 'Black', pip: '{B}', color: MANA_COLORS.B },
  { label: 'Red', pip: '{R}', color: MANA_COLORS.R },
  { label: 'Green', pip: '{G}', color: MANA_COLORS.G },
  { label: 'Grey', pip: '{C}', color: MANA_COLORS.grey }
]);

const pieSlices = computed(() => {
  const data = pieChartData.value;
  const total = data.reduce((sum, d) => sum + d.value, 0);
  if (total === 0) return [];
  
  if (data.length === 1) {
    const d = data[0];
    const radius = 100;
    // Full circle path using two 180-degree arcs to avoid zero-length arc issues
    const path = `M ${radius} 0 A ${radius} ${radius} 0 1 1 ${radius} ${2 * radius} A ${radius} ${radius} 0 1 1 ${radius} 0 Z`;
    return [{ ...d, path }];
  }

  let currentAngle = -Math.PI / 2; // Start from top
  return data.map(d => {
    const angle = (d.value / total) * 2 * Math.PI;
    const startAngle = currentAngle;
    const endAngle = currentAngle + angle;
    currentAngle = endAngle;
    
    // Path calculation
    const radius = 100;
    const x1 = radius + radius * Math.cos(startAngle);
    const y1 = radius + radius * Math.sin(startAngle);
    const x2 = radius + radius * Math.cos(endAngle);
    const y2 = radius + radius * Math.sin(endAngle);
    const largeArcFlag = angle <= Math.PI ? 0 : 1;
    const path = `M ${radius} ${radius} L ${x1} ${y1} A ${radius} ${radius} 0 ${largeArcFlag} 1 ${x2} ${y2} Z`;
    
    return { ...d, path };
  });
});

const maxCurveCount = computed(() => Math.max(...manaCurveDistribution.value, 1));

const manaCurvePoints = computed(() => {
  const dist = manaCurveDistribution.value;
  return dist.map((count, i) => ({
    x: i * 100 + 50,
    y: 200 - (count / maxCurveCount.value * 160) - 25
  }));
});

const manaCurvePath = computed(() => {
  const points = manaCurvePoints.value;
  return points.reduce((p, c, i) => p + (i === 0 ? `M ${c.x} ${c.y}` : ` L ${c.x} ${c.y}`), "");
});

const pipCounts = computed(() => {
  const cards = allDeckCards.value;
  const totals = { W: 0, U: 0, B: 0, R: 0, G: 0 };

  for (const card of cards) {
    const manaCost = typeof card.mana_cost === "string" ? card.mana_cost : "";
    for (const pip of ["W", "U", "B", "R", "G"]) {
      const matches = manaCost.match(new RegExp(`\\{${pip}\\}`, "g"));
      totals[pip] += matches ? matches.length : 0;
    }
  }

  return totals;
});

async function handleAddCard() {
  if (isAddingCard.value || isUpdatingDeck.value) {
    return;
  }

  const normalizedDeckId = normalizeDeckId(props.deckId);
  if (normalizedDeckId === null) {
    showError("Invalid deck id.");
    return;
  }

  const trimmed = searchName.value.trim();
  if (!trimmed) {
    showError("Enter a card name first.");
    return;
  }

  try {
    isAddingCard.value = true;
    deck.value = await addCardToDeckCommand(normalizedDeckId, trimmed);
    searchName.value = "";
    clearSuggestions();
    showSuccess(`Added "${trimmed}" to ${deck.value.name}`);
  } catch (e) {
    showError(`Failed to add card: ${String(e)}`);
    console.error(e);
  } finally {
    isAddingCard.value = false;
  }
}

async function handleAddCopy(cardName) {
  const normalizedDeckId = normalizeDeckId(props.deckId);
  if (normalizedDeckId === null) {
    showError("Invalid deck id.");
    return;
  }

  try {
    isUpdatingDeck.value = true;
    deck.value = await addCardToDeckCommand(normalizedDeckId, cardName);
  } catch (e) {
    showError(`Failed to add card: ${String(e)}`);
    console.error(e);
  } finally {
    isUpdatingDeck.value = false;
  }
}

async function handleRemoveCopy(cardId, cardName) {
  const normalizedDeckId = normalizeDeckId(props.deckId);
  if (normalizedDeckId === null) {
    showError("Invalid deck id.");
    return;
  }

  try {
    isUpdatingDeck.value = true;
    deck.value = await removeCardFromDeckCommand(normalizedDeckId, cardId);
    showSuccess(`Removed one "${cardName}"`);
  } catch (e) {
    showError(`Failed to remove card: ${String(e)}`);
    console.error(e);
  } finally {
    isUpdatingDeck.value = false;
  }
}

async function handleSetCommander(cardId, cardName) {
  const normalizedDeckId = normalizeDeckId(props.deckId);
  if (normalizedDeckId === null) {
    showError("Invalid deck id.");
    return;
  }

  try {
    isUpdatingDeck.value = true;
    deck.value = await setDeckCommanderCommand(normalizedDeckId, cardId);
    showSuccess(`Set "${cardName}" as commander`);
  } catch (e) {
    showError(`Failed to set commander: ${String(e)}`);
    console.error(e);
  } finally {
    isUpdatingDeck.value = false;
  }
}

async function handleSetPartner(cardId, cardName) {
  const normalizedDeckId = normalizeDeckId(props.deckId);
  if (normalizedDeckId === null) {
    showError("Invalid deck id.");
    return;
  }

  try {
    isUpdatingDeck.value = true;
    deck.value = await setDeckPartnerCommand(normalizedDeckId, cardId);
    showSuccess(`Set "${cardName}" as partner commander`);
  } catch (e) {
    showError(`Failed to set partner: ${String(e)}`);
    console.error(e);
  } finally {
    isUpdatingDeck.value = false;
  }
}

async function handleRemovePartner(cardId, cardName){
  const normalizedDeckId = normalizeDeckId(props.deckId);
  if (normalizedDeckId === null){
    showError("Invalid deck id.");
    return;
  }
  try {
    isUpdatingDeck.value = true;
    deck.value = await removeDeckPartnerCommand(normalizedDeckId,cardId);
    showSuccess(`Removed "${cardName}" as partner commander`);
  } catch (e){
    showError(`Failed to remove partner: ${String(e)}`);
    console.error(e)
  } finally {
    isUpdatingDeck.value = false;
  }

}


async function handleRemoveCommander(cardName) {
  const normalizedDeckId = normalizeDeckId(props.deckId);
  if (normalizedDeckId === null) {
    showError("Invalid deck id.");
    return;
  }

  try {
    isUpdatingDeck.value = true;
    deck.value = await removeDeckCommanderCommand(normalizedDeckId);
    showSuccess(`Removed "${cardName}" as commander`);
  } catch (e) {
    showError(`Failed to remove commander: ${String(e)}`);
    console.error(e);
  } finally {
    isUpdatingDeck.value = false;
  }
}

async function handleDeleteCommander(cardName) {
  const normalizedDeckId = normalizeDeckId(props.deckId);
  if (normalizedDeckId === null) {
    showError("Invalid deck id.");
    return;
  }

  try {
    isUpdatingDeck.value = true;
    deck.value = await deleteDeckCommanderCommand(normalizedDeckId);
    showSuccess(`Removed "${cardName}" from the deck`);
  } catch (e) {
    showError(`Failed to remove commander card: ${String(e)}`);
    console.error(e);
  } finally {
    isUpdatingDeck.value = false;
  }
}

async function handleCreatePackage() {
  const trimmed = newPackageName.value.trim();
  if (!trimmed) {
    showError("Enter a package name first.");
    return;
  }

  try {
    isCreatingPackage.value = true;
    const newPackage = await createPackageCommand(trimmed);
    packages.value.push(newPackage);
    selectedPackageId.value = newPackage.id;
    newPackageName.value = "";
    showSuccess(`Created package "${newPackage.name}"`);
  } catch (e) {
    showError(`Failed to create package: ${String(e)}`);
    console.error(e);
  } finally {
    isCreatingPackage.value = false;
  }
}

async function handleSubmitPackageDialog() {
  if (isSubmittingPackageAction.value) {
    return;
  }

  const packageId = Number(selectedPackageId.value);
  if (!Number.isInteger(packageId) || packageId <= 0) {
    showError("Select a package first.");
    return;
  }

  try {
    isSubmittingPackageAction.value = true;

    if (packageDialogMode.value === "card") {
      const updatedPackage = await addCardToPackageCommand(packageId, pendingPackageCardName.value);
      const index = packages.value.findIndex((entry) => entry.id === updatedPackage.id);
      if (index >= 0) {
        packages.value[index] = updatedPackage;
      } else {
        packages.value.push(updatedPackage);
      }
      showSuccess(`Added "${pendingPackageCardName.value}" to ${updatedPackage.name}`);
    } else {
      const normalizedDeckId = normalizeDeckId(props.deckId);
      if (normalizedDeckId === null) {
        showError("Invalid deck id.");
        return;
      }
      const selectedPackage = packages.value.find((entry) => entry.id === packageId);
      deck.value = await addPackageToDeckCommand(normalizedDeckId, packageId);
      showSuccess(`Added package "${selectedPackage?.name || "Package"}" to ${deck.value.name}`);
    }

    closePackageDialog();
  } catch (e) {
    showError(`Failed to apply package action: ${String(e)}`);
    console.error(e);
  } finally {
    isSubmittingPackageAction.value = false;
  }
}

function handleAddToPackage(cardName) {
  openAddCardToPackageDialog(cardName);
}

watch(() => props.deckId, loadDeck);
watch(searchName, (value) => {
  if (suggestionSearchTimeout) {
    clearTimeout(suggestionSearchTimeout);
  }

  const trimmed = value.trim();
  if (trimmed.length < 2) {
    isSearchingCards.value = false;
    clearSuggestions();
    return;
  }

  suggestionSearchTimeout = setTimeout(async () => {
    try {
      isSearchingCards.value = true;
      const suggestions = await searchCardSuggestionsCommand(trimmed);
      cardSuggestions.value = suggestions;
      activeSuggestionIndex.value = suggestions.length > 0 ? 0 : -1;
    } catch (e) {
      console.error(e);
      clearSuggestions();
    } finally {
      isSearchingCards.value = false;
    }
  }, 180);
});
async function refreshTestHand() {
  const cards = Array.isArray(deck.value?.cards) ? deck.value.cards : [];
  if (cards.length === 0) {
    testHand.value = [];
    return;
  }
  
  // Shuffle all copies of all cards
  const pool = [...cards];
  for (let i = pool.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [pool[i], pool[j]] = [pool[j], pool[i]];
  }
  
  // Take first 7
  const selectedCards = pool.slice(0, 7);
  const paths = selectedCards.map(c => c.image || c.image_uri || "");
  
  try {
    const b64s = await getBase64ImagesCommand(paths);
    testHand.value = selectedCards.map((card, idx) => ({
      ...card,
      displayImage: b64s[idx] || card.image_uri || ""
    }));
  } catch (e) {
    console.error("Failed to fetch test hand images", e);
    testHand.value = selectedCards;
  }
}

function getCardImage(card) {
  const raw = card?.image || card?.image_uri;
  if (!raw) return "";
  if (raw.startsWith("http") || raw.startsWith("data:")) {
    return raw;
  }
  return convertFileSrc(raw);
}

watch(() => deck.value?.cards?.length, (newLength) => {
  if (newLength > 0 && testHand.value.length === 0) {
    refreshTestHand();
  }
});

onMounted(async () => {
  await Promise.all([loadDeck(), loadPackages()]);
});
</script>

<template>
  <v-container class="deck-editor-page">
    <section class="deck-editor-hero">
      <div class="hero-content">
        <div class="d-flex align-center gap-4">
          <v-btn icon variant="text" @click="goBack" color="primary">
            <v-icon :icon="mdiChevronLeft" size="32"></v-icon>
          </v-btn>
          <h1 class="text-primary">{{ deck?.name || "Deck" }}</h1>
        </div>
        <div class="hero-actions">
          <div class="deck-search-wrap">
            <v-text-field
              v-model="searchName"
              class="deck-search"
              label="Search and add a card"
              density="comfortable"
              hide-details
              :loading="isSearchingCards"
              @focus="handleSearchFocus"
              @blur="handleSearchBlur"
              @keydown.enter.prevent="handleSearchEnter"
              @keydown.down.prevent="moveSuggestion(1)"
              @keydown.up.prevent="moveSuggestion(-1)"
              @keydown.esc="hideSuggestions"
            />

            <div v-if="showCardSuggestions" class="deck-search-suggestions">
              <button
                v-for="(suggestion, index) in cardSuggestions"
                :key="`${suggestion.name}-${index}`"
                type="button"
                class="deck-search-suggestion"
                :class="{
                  'deck-search-suggestion--active': index === activeSuggestionIndex,
                  'deck-search-suggestion--illegal':
                    suggestion.commander_legality && suggestion.commander_legality !== 'legal',
                  'deck-search-suggestion--game-changer':
                    suggestion.game_changer &&
                    !(suggestion.commander_legality && suggestion.commander_legality !== 'legal'),
                }"
                @mousedown.prevent="selectSuggestion(suggestion)"
              >
                <div class="deck-search-suggestion__top">
                  <div class="d-flex align-center flex-grow-1">
                    <span>{{ suggestion.name }}</span>
                    <span
                      v-if="suggestion.game_changer"
                      class="suggestion-pill suggestion-pill--game-changer ml-1"
                    >
                      GAME CHANGER
                    </span>
                    <span
                      v-if="suggestion.commander_legality && suggestion.commander_legality !== 'legal'"
                      class="suggestion-pill suggestion-pill--illegal ml-1"
                    >
                      {{ suggestion.commander_legality.toUpperCase().replace("_", " ") }}
                    </span>
                    <v-icon
                      v-if="suggestion.game_changer"
                      :icon="mdiGaugeFull"
                      size="14"
                      color="amber-darken-2"
                      title="Game Changer"
                      class="ml-1"
                    ></v-icon>
                    <v-icon
                      v-if="suggestion.commander_legality && suggestion.commander_legality !== 'legal'"
                      :icon="mdiCancel"
                      size="14"
                      color="error"
                      title="Banned"
                      class="ml-1"
                    ></v-icon>
                  </div>
                  <ManaText
                    v-if="suggestion.mana_cost"
                    :text="suggestion.mana_cost"
                    :cost="true"
                  />
                </div>
                <span class="deck-search-suggestion__type">{{ suggestion.type_line }}</span>
              </button>
            </div>
          </div>
          <v-btn
            class="add-card-btn"
            :prepend-icon="mdiPlus"
            :loading="isAddingCard"
            @click="handleAddCard"
          >
            Add Card
          </v-btn>
          <v-btn
            class="package-btn"
            variant="outlined"
            :prepend-icon="mdiPackageVariantClosedPlus"
            :loading="isLoadingPackages"
            @click="openAddPackageToDeckDialog"
          >
            Add Package
          </v-btn>
          <v-btn
            class="import-btn"
            variant="outlined"
            :prepend-icon="mdiDownload"
            @click="openImportDialog"
          >
            Import Deck
          </v-btn>
          <v-btn
            class="export-btn"
            variant="outlined"
            :prepend-icon="mdiExport"
            :disabled="!deck || deckCardTotal === 0"
            @click="exportDeckAsText"
          >
            Export Deck
          </v-btn>
        </div>
      </div>
    </section>

    <div v-if="loadError" class="feedback feedback--error">
      <v-icon :icon="mdiAlertCircleOutline" size="18"></v-icon>
      <span>{{ loadError }}</span>
    </div>

    <template v-else-if="deck">
      <section class="deck-metrics">
        <article class="metric-card">
          <span class="metric-label">Cards</span>
          <strong>{{ deckCardTotal }}</strong>
        </article>
        <article class="metric-card">
          <span class="metric-label">Average MV</span>
          <strong>{{ avgManaValue }}</strong>
        </article>
        <article class="metric-card">
          <span class="metric-label">Bracket</span>
          <strong>{{ deck.bracket ?? 2 }}</strong>
        </article>
        <article class="metric-card">
          <div class="d-flex align-center">
            <span class="metric-label">Game changers</span>
            <v-icon
              v-if="deck.game_changer_count > 0"
              :icon="mdiGaugeFull"
              size="16"
              color="amber-darken-2"
              class="ml-1"
            ></v-icon>
          </div>
          <strong>{{ deck.game_changer_count ?? 0 }}</strong>
        </article>
        <article v-if="deck.illegal_count > 0" class="metric-card metric-card--illegal">
          <div class="d-flex align-center">
            <span class="metric-label">Illegal</span>
            <v-icon
              :icon="mdiCancel"
              size="16"
              color="error"
              class="ml-1"
            ></v-icon>
          </div>
          <strong>{{ deck.illegal_count }}</strong>
        </article>
        <article class="metric-card">
          <ManaText class="metric-label metric-label--symbol" text="{W}" :cost="true" />
          <strong>{{ pipCounts.W }}</strong>
        </article>
        <article class="metric-card">
          <ManaText class="metric-label metric-label--symbol" text="{U}" :cost="true" />
          <strong>{{ pipCounts.U }}</strong>
        </article>
        <article class="metric-card">
          <ManaText class="metric-label metric-label--symbol" text="{B}" :cost="true" />
          <strong>{{ pipCounts.B }}</strong>
        </article>
        <article class="metric-card">
          <ManaText class="metric-label metric-label--symbol" text="{R}" :cost="true" />
          <strong>{{ pipCounts.R }}</strong>
        </article>
        <article class="metric-card">
          <ManaText class="metric-label metric-label--symbol" text="{G}" :cost="true" />
          <strong>{{ pipCounts.G }}</strong>
        </article>
      </section>

      <section class="deck-charts mb-6">
        <v-row>
          <v-col cols="12" md="8">
            <v-card class="mana-curve-card pa-4 fill-height" variant="flat" border>
              <div class="d-flex align-center justify-space-between mb-4">
                <h3 class="text-subtitle-1 font-weight-bold">Mana Curve</h3>
                <v-btn-toggle v-model="manaCurveMode" mandatory density="compact" color="primary" variant="outlined">
                  <v-btn value="bar" size="x-small">Bar</v-btn>
                  <v-btn value="color" size="x-small">Color</v-btn>
                  <v-btn value="line" size="x-small">Line</v-btn>
                </v-btn-toggle>
              </div>
              <div class="mana-curve-chart" style="height: 200px;">
                <svg width="100%" height="100%" viewBox="0 0 800 200" preserveAspectRatio="none">
                  <defs>
                    <clipPath v-for="(count, i) in manaCurveDistribution" :key="'clip'+i" :id="'clip-bar-'+i">
                      <rect 
                        :x="i * 100 + 10"
                        :y="200 - (count / maxCurveCount * 160) - 25"
                        width="80"
                        :height="(count / maxCurveCount * 160)"
                        rx="4"
                      />
                    </clipPath>
                  </defs>

                  <!-- Bar Chart (Single Color) -->
                  <template v-if="manaCurveMode === 'bar'">
                    <rect 
                      v-for="(count, i) in manaCurveDistribution" 
                      :key="'b'+i"
                      :x="i * 100 + 10"
                      :y="200 - (count / maxCurveCount * 160) - 25"
                      width="80"
                      :height="(count / maxCurveCount * 160)"
                      :fill="MANA_COLORS.grey"
                      rx="4"
                    />
                  </template>

                  <!-- Bar Chart (Segmented Colors) -->
                  <template v-else-if="manaCurveMode === 'color'">
                    <g 
                      v-for="(dist, i) in manaCurveColoredDistribution" 
                      :key="'c'+i"
                      :clip-path="dist.total > 0 ? `url(#clip-bar-${i})` : null"
                    >
                      <rect 
                        v-for="(segment, si) in dist.segments"
                        :key="'s'+i+'-'+si"
                        :x="i * 100 + 10"
                        :y="200 - (dist.segments.slice(0, si + 1).reduce((sum, s) => sum + s.value, 0) / maxCurveCount * 160) - 25"
                        width="80"
                        :height="(segment.value / maxCurveCount * 160) + 0.5"
                        :fill="segment.color"
                        rx="0"
                      />
                    </g>
                  </template>
                  
                  <!-- Line Chart -->
                  <template v-else>
                    <path 
                      :d="manaCurvePath" 
                      fill="none" 
                      :stroke="MANA_COLORS.grey" 
                      stroke-width="3" 
                      stroke-linecap="round" 
                      stroke-linejoin="round"
                    />
                    <circle 
                      v-for="(p, i) in manaCurvePoints" 
                      :key="'p'+i"
                      :cx="p.x"
                      :cy="p.y"
                      r="5"
                      :fill="MANA_COLORS.grey"
                      stroke="currentColor"
                      stroke-width="1"
                    />
                  </template>

                  <text 
                    v-for="(_, i) in manaCurveDistribution" 
                    :key="'t'+i"
                    :x="i * 100 + 50"
                    y="195"
                    text-anchor="middle"
                    fill="currentColor"
                    font-size="14"
                    class="font-weight-bold"
                  >{{ i === 7 ? '7+' : i }}</text>
                  <text 
                    v-for="(count, i) in manaCurveDistribution" 
                    v-show="count > 0"
                    :key="'v'+i"
                    :x="i * 100 + 50"
                    :y="200 - (count / maxCurveCount * 160) - 30"
                    text-anchor="middle"
                    fill="currentColor"
                    font-size="12"
                  >{{ count }}</text>
                </svg>
              </div>

              <div v-if="manaCurveMode === 'color'" class="d-flex justify-center flex-wrap mt-4" style="gap: 16px;">
                <div v-for="item in manaCurveLegendItems" :key="item.label" class="d-flex align-center">
                  <div 
                    class="mr-2" 
                    :style="{ 
                      width: '12px', 
                      height: '12px', 
                      backgroundColor: item.color, 
                      borderRadius: '2px',
                      border: '1px solid rgba(var(--v-border-color), var(--v-border-opacity))'
                    }"
                  ></div>
                  <ManaText :text="item.pip" :cost="true" style="font-size: 0.75rem;" class="mr-1" />
                  <span class="text-caption font-weight-medium">{{ item.label }}</span>
                </div>
              </div>
            </v-card>
          </v-col>
          <v-col cols="12" md="4">
            <v-card class="mana-stats-card pa-4 fill-height" variant="flat" border>
              <div class="d-flex align-center justify-space-between mb-4">
                <h3 class="text-subtitle-1 font-weight-bold">Distribution</h3>
                <v-btn-toggle v-model="pieChartMode" mandatory density="compact" color="primary" variant="outlined">
                  <v-btn value="pips-grey" size="x-small">Pips</v-btn>
                  <v-btn value="combined" size="x-small">All</v-btn>
                </v-btn-toggle>
              </div>
              <div class="d-flex align-center justify-center" style="height: 180px;">
                <svg width="180" height="180" viewBox="0 0 200 200">
                  <path 
                    v-for="(slice, i) in pieSlices" 
                    :key="i"
                    :d="slice.path"
                    :fill="slice.color"
                    stroke="rgba(var(--v-theme-surface), 0.8)"
                    stroke-width="1"
                  >
                    <title>{{ slice.label }}: {{ slice.value }}</title>
                  </path>
                </svg>
                <div class="ml-4 pie-chart-legend" style="font-size: 11px; max-width: 130px; overflow: hidden;">
                  <template v-for="slice in pieSlices" :key="slice.label">
                    <div class="pie-chart-legend-icon">
                      <ManaText :text="slice.pip" :cost="true" style="font-size: 0.85rem;" />
                    </div>
                    <div class="pie-chart-legend-value" :title="`${slice.label}: ${slice.value}`">
                      {{ slice.value }}
                    </div>
                  </template>
                </div>
              </div>
            </v-card>
          </v-col>
        </v-row>
      </section>

      <section v-if="testHand.length > 0" class="test-hand-simulation mb-6">
        <v-card class="pa-4" variant="flat" border>
          <div class="d-flex align-center justify-space-between mb-4">
            <h3 class="text-subtitle-1 font-weight-bold">Test Hand</h3>
            <v-btn size="small" variant="outlined" color="primary" @click="refreshTestHand">
              New Hand
            </v-btn>
          </div>
          <div class="test-hand-container">
            <div v-for="(card, index) in testHand" :key="index" class="test-hand-card">
              <img :src="card.displayImage || getCardImage(card)" :alt="card.name" :title="card.name" />
            </div>
          </div>
        </v-card>
      </section>

      <section class="deck-layout">
        <aside class="deck-sidebar">
          <section class="deck-panel">
            <div class="panel-heading">
              <v-icon :icon="mdiCrownOutline" size="18"></v-icon>
              <h2>Command Zone</h2>
            </div>
            <div v-if="commanderSection.length > 0" class="deck-list">
              <DeckCardRow
                v-for="entry in commanderSection"
                :key="`commander-${entry.card.id}`"
                :card="entry.card"
                :quantity="entry.quantity"
                :editable="true"
                :can-remove-partner="canRemovePartner(entry.card)"
                :can-remove-commander="true"
                :show-add-to-package-action="true"
                @add-copy="handleAddCopy(entry.card.name)"
                @remove-copy="handleDeleteCommander(entry.card.name)"
                @remove-partner="handleRemovePartner(entry.card.id, entry.card.name)"
                @remove-commander="handleRemoveCommander(entry.card.name)"
                @add-to-package="handleAddToPackage(entry.card.name)"
              />
            </div>
            <p v-else class="empty-copy">No commander selected yet.</p>
          </section>
        </aside>

        <main class="deck-main">
          <section class="deck-panel">
            <div class="panel-heading">
              <div class="d-flex align-center flex-grow-1">
                <v-icon :icon="mdiCardsOutline" size="18"></v-icon>
                <h2 class="ml-2">Mainboard</h2>
              </div>
              <div class="d-flex align-center bg-amber-darken-4 px-2 py-1 rounded-pill" v-if="deck.game_changer_count > 0">
                <v-icon :icon="mdiGaugeFull" size="16" color="amber-lighten-2"></v-icon>
                <span class="text-caption font-weight-bold ml-1 text-amber-lighten-4">
                  {{ deck.game_changer_count }} GAME CHANGER{{ deck.game_changer_count > 1 ? 'S' : '' }}
                </span>
              </div>
            </div>

            <div v-if="mainDeckSections.length > 0" class="deck-sections">
              <section
                v-for="section in mainDeckSections"
                :key="section.title"
                class="deck-type-section"
              >
                <header class="deck-type-section__header">
                  <h3>{{ section.title }}</h3>
                  <span>{{ section.entries.reduce((sum, entry) => sum + entry.quantity, 0) }} cards</span>
                </header>
                <div class="deck-list">
                  <DeckCardRow
                    v-for="entry in section.entries"
                    :key="`${section.title}-${entry.card.id}`"
                    :card="entry.card"
                    :quantity="entry.quantity"
                    :editable="true"
                    :can-set-commander="canSetCommander(entry.card)"
                    :can-set-partner="canSetPartner(entry.card)"
                    :show-add-to-package-action="true"
                    @add-copy="handleAddCopy(entry.card.name)"
                    @remove-copy="handleRemoveCopy(entry.card.id, entry.card.name)"
                    @set-commander="handleSetCommander(entry.card.id, entry.card.name)"
                    @set-partner="handleSetPartner(entry.card.id, entry.card.name)"
                    @add-to-package="handleAddToPackage(entry.card.name)"
                  />
                </div>
              </section>
            </div>

            <div v-else class="empty-state">
              <h3>This deck is empty</h3>
              <p>Add cards to the deck and they will appear here in grouped sections.</p>
            </div>
          </section>
        </main>
      </section>
    </template>

    <!-- Import Deck Dialog -->
    <v-dialog v-model="importDialogVisible" max-width="720">
      <v-card class="import-dialog">
        <v-card-title>Import Deck</v-card-title>
        <v-card-text>
          <p class="import-dialog__help">Paste a deck list using the format: "1 Card Name" per line.</p>
          <v-textarea
            v-model="importText"
            label="Deck list"
            rows="10"
            density="comfortable"
            hide-details="auto"
            class="import-textarea"
            @input="importErrors = validateImportLines(importText)"
          />

          <div v-if="importErrors.length > 0" class="import-errors">
            <p>Format errors found on the following lines:</p>
            <ul>
              <li v-for="err in importErrors" :key="`err-${err.line}`">Line {{ err.line }}: "{{ err.text }}"</li>
            </ul>
          </div>

          <div class="import-actions" style="display:flex;gap:8px;margin-top:12px;align-items:center;">
            <v-btn variant="outlined" :loading="isPasting" @click="pasteFromClipboard">Paste from Clipboard</v-btn>
            <v-btn variant="outlined" @click="importText = ''">Clear List</v-btn>
            <v-spacer />
          </div>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="outlined" @click="closeImportDialog">Cancel</v-btn>
          <v-btn
            variant="outlined"
            color="primary"
            :loading="isImporting"
            :disabled="isImporting || importErrors.length > 0"
            @click="handleImport"
          >
            Import
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <v-dialog v-model="packageDialogVisible" max-width="560">
      <v-card class="package-dialog">
        <v-card-title>
          {{ packageDialogMode === "card" ? "Add Card to Package" : "Add Package to Deck" }}
        </v-card-title>
        <v-card-text class="package-dialog__content">
          <p v-if="packageDialogMode === 'card'" class="package-dialog__copy">
            Choose a package for "{{ pendingPackageCardName }}".
          </p>
          <p v-else class="package-dialog__copy">
            Choose a package to add its cards to this deck.
          </p>

          <v-select
            v-model="selectedPackageId"
            :items="packages"
            item-title="name"
            item-value="id"
            label="Package"
            density="comfortable"
            hide-details="auto"
            :loading="isLoadingPackages"
            :disabled="isLoadingPackages"
          />

          <div class="package-dialog__create">
            <v-btn
              variant="outlined"
              :loading="isCreatingPackage"
              @click="handleCreatePackage"
            >
              Create Package
            </v-btn>
            <v-text-field
              v-model="newPackageName"
              label="New package name"
              density="comfortable"
              hide-details="auto"
              @keydown.enter.prevent="handleCreatePackage"
            />
          </div>
        </v-card-text>
        <v-card-actions class="package-dialog__actions">
          <v-spacer />
          <v-btn variant="outlined" @click="closePackageDialog">Cancel</v-btn>
          <v-btn
            variant="outlined"
            color="primary"
            :loading="isSubmittingPackageAction"
            :disabled="packages.length === 0 && !selectedPackageId"
            @click="handleSubmitPackageDialog"
          >
            {{ packageDialogMode === "card" ? "Add to Package" : "Add Package" }}
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <v-snackbar
      v-model="snackbarVisible"
      :color="snackbarColor"
      :timeout="2500"
    >
      {{ snackbarMessage }}
    </v-snackbar>
  </v-container>
</template>

<style scoped>
.deck-editor-page {
  max-width: 1440px;
  padding: 28px;
}

.deck-editor-hero {
  padding: 28px 32px;
  margin-bottom: 20px;
  border-radius: 28px;
  background: rgb(var(--v-theme-surface));
  border: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
}

.hero-content {
  display: grid;
  gap: 16px;
}

.gap-4 {
  gap: 16px;
}

.deck-editor-hero h1 {
  margin: 0;
  font-size: clamp(2rem, 3vw, 3.2rem);
  line-height: 1;
}

.hero-actions {
  display: flex;
  gap: 12px;
  align-items: center;
  flex-wrap: wrap;
}

.deck-search {
  min-width: 0;
}

.deck-search-wrap {
  position: relative;
  min-width: 320px;
  max-width: 520px;
  width: 100%;
}

.deck-search-suggestions {
  position: absolute;
  top: calc(100% + 8px);
  left: 0;
  right: 0;
  z-index: 20;
  display: grid;
  gap: 6px;
  padding: 10px;
  border-radius: 18px;
  background: rgb(var(--v-theme-surface));
  border: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: 0 22px 40px rgba(0, 0, 0, 0.4);
}

.deck-search-suggestion {
  display: grid;
  gap: 4px;
  padding: 10px 12px;
  border: 1px solid transparent;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.05);
  color: inherit;
  text-align: left;
  cursor: pointer;
  transition: all 0.2s ease;
}

.deck-search-suggestion--illegal {
  background: rgba(var(--v-theme-error), 0.1);
  border-color: rgba(var(--v-theme-error), 0.3);
}

.deck-search-suggestion--game-changer {
  background: rgba(255, 193, 7, 0.1);
  border-color: rgba(255, 193, 7, 0.3);
}

.deck-search-suggestion--active,
.deck-search-suggestion:hover {
  background: rgba(var(--v-theme-primary), 0.1);
  border-color: rgba(var(--v-theme-primary), 0.3);
}

.deck-search-suggestion--illegal.deck-search-suggestion--active,
.deck-search-suggestion--illegal:hover {
  background: rgba(var(--v-theme-error), 0.2);
  border-color: rgba(var(--v-theme-error), 0.5);
}

.deck-search-suggestion--game-changer.deck-search-suggestion--active,
.deck-search-suggestion--game-changer:hover {
  background: rgba(255, 193, 7, 0.2);
  border-color: rgba(255, 193, 7, 0.5);
}

.deck-search-suggestion__top {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: center;
  font-weight: 700;
}

.deck-search-suggestion__type {
  opacity: 0.7;
  font-size: 0.85rem;
}

.suggestion-pill {
  font-size: 0.6rem;
  font-weight: 800;
  padding: 1px 4px;
  border-radius: 4px;
  letter-spacing: 0.04em;
  flex: 0 0 auto;
}

.suggestion-pill--illegal {
  background: #991b1b;
  color: #fff;
}

.suggestion-pill--game-changer {
  background: #c2410c;
  color: #fff;
}

.add-card-btn {
  flex: 0 0 auto;
}

.package-btn {
  flex: 0 0 auto;
}

.import-btn {
  flex: 0 0 auto;
}

.import-textarea {
  max-height: 300px;
  overflow-y: auto !important;
}

.feedback {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 16px;
  margin-bottom: 20px;
  border-radius: 16px;
}

.feedback--error {
  background: rgba(var(--v-theme-error), 0.1);
  color: rgb(var(--v-theme-error));
}

.deck-metrics {
  display: grid;
  grid-template-columns: repeat(8, minmax(0, 1fr));
  gap: 14px;
  margin-bottom: 20px;
}

.metric-card {
  padding: 16px 18px;
  border-radius: 20px;
  background: rgb(var(--v-theme-surface));
  border: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: 0 16px 30px rgba(0, 0, 0, 0.2);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  text-align: center;
}

.metric-card--illegal {
  background: rgba(var(--v-theme-error), 0.1);
  border-color: rgba(var(--v-theme-error), 0.3);
}

.metric-label {
  display: block;
  font-size: 0.82rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  opacity: 0.7;
}

.metric-card strong {
  font-size: 1.6rem;
  width: 100%;
}

.metric-label--symbol {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  min-height: 2.3rem;
}

.metric-label--symbol :deep(.ms-cost) {
  width: 2.3rem;
  height: 2.3rem;
  line-height: 2.3rem;
  font-size: 1.4rem;
}

.deck-layout {
  display: grid;
  grid-template-columns: minmax(280px, 340px) minmax(0, 1fr);
  gap: 20px;
  align-items: start;
}

.deck-sidebar,
.deck-main,
.deck-sections {
  display: grid;
  gap: 20px;
}

.deck-panel {
  padding: 20px;
  border-radius: 24px;
  background: rgb(var(--v-theme-surface));
  border: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
}

.deck-charts {
  width: 100%;
}

.mana-curve-chart rect {
  transition: height 0.5s cubic-bezier(0.175, 0.885, 0.32, 1.275), y 0.5s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}

.mana-stats-card svg path {
  transition: d 0.4s ease, fill 0.4s ease;
}

.pie-chart-legend {
  display: grid;
  grid-template-columns: 24px 1fr;
  column-gap: 8px;
  row-gap: 4px;
  align-items: center;
}

.pie-chart-legend-icon {
  width: 24px;
  display: flex;
  justify-content: center;
  align-items: center;
}

.pie-chart-legend-value {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  text-align: left;
  justify-self: start;
}

@media (max-width: 1200px) {
  .deck-metrics {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }
}

@media (max-width: 640px) {
  .deck-metrics {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  .deck-layout {
    grid-template-columns: 1fr;
  }
}

.panel-heading {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 16px;
}

.panel-heading h2,
.deck-type-section__header h3 {
  margin: 0;
}

.deck-list {
  display: grid;
  gap: 10px;
}

.deck-type-section {
  display: grid;
  gap: 12px;
}

.deck-type-section__header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: baseline;
}

.deck-type-section__header span {
  opacity: 0.7;
  font-size: 0.9rem;
}

.empty-copy,
.empty-state p {
  margin: 0;
  opacity: 0.7;
  line-height: 1.5;
}

.empty-state {
  padding: 36px 12px 18px;
  text-align: center;
}

.empty-state h3 {
  margin: 0 0 8px;
}

.package-dialog__content {
  display: grid;
  gap: 16px;
}

.package-dialog__copy {
  margin: 0;
  color: #5f6f86;
}

.package-dialog__create {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 12px;
  align-items: center;
}

.package-dialog__actions {
  padding: 0 24px 20px;
}

@media (max-width: 1080px) {
  .deck-metrics {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  .deck-layout {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 720px) {
  .deck-editor-page {
    padding: 16px;
  }

  .deck-editor-hero {
    padding: 22px;
  }

  .hero-actions {
    width: 100%;
    justify-content: stretch;
  }

  .deck-search {
    width: 100%;
  }

  .deck-search-wrap {
    min-width: 0;
  }

  .deck-metrics {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .package-dialog__create {
    grid-template-columns: 1fr;
  }
}

.test-hand-container {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  justify-content: center;
  min-height: 200px;
  align-items: center;
}

.test-hand-card {
  width: 150px;
  aspect-ratio: 63 / 88;
  border-radius: 10px;
  overflow: hidden;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  background: #f0f2f5;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 0.2s ease-in-out;
}

.test-hand-card:hover {
  transform: translateY(-4px);
}

.test-hand-card img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
</style>
