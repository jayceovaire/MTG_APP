import { createRouter, createWebHashHistory } from 'vue-router'

import DeckEditor from "../views/DeckEditor.vue";
import HomeView from "../views/HomeView.vue";
import DeckLibrary from "../views/DeckLibrary.vue";
import PackageLibrary from "../views/PackageLibrary.vue";
import PackageEditor from "../views/PackageEditor.vue";
import Collection from "../views/Collection.vue";
import PowerCalculator from "../views/PowerCalculator.vue";
import Roast from "../views/Roast.vue";
import Settings from "../views/Settings.vue";
import InfoView from "../views/InfoView.vue";
import CardViewer from "../views/CardViewer.vue";

const routes = [
    { path: "/", component: HomeView },
    { path: "/deck-editor/:deckId", component: DeckEditor, props: true },
    { path: "/deck-library", component: DeckLibrary },
    { path: "/package-editor/:packageId", component: PackageEditor, props: true },
    { path: "/package-library", component: PackageLibrary },
    {path: "/power-calculator", component: PowerCalculator},
    {path: "/collection", component: Collection},
    {path: "/roast", component: Roast},
    {path: "/settings", component: Settings},
    {path: "/info", component: InfoView},
    { path: "/card-viewer", component: CardViewer },
]

export const router = createRouter({
    history: createWebHashHistory(),
    routes
})
