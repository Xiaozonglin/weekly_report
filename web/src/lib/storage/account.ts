import type { User } from "@models/user";
import { createStore } from "solid-js/store";

export const [accountStore, setAccountStore] = createStore({
    user: null as User | null,
});
