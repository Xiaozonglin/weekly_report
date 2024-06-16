import type { User } from "@models/user";
import { createStore } from "solid-js/store";

export const [routeStore, setRouteStore] = createStore({
    user: null as User | null,
    week: null as number | null,
});
