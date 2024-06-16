import { makePersisted } from "@solid-primitives/storage";
import { createStore } from "solid-js/store";

export const [platformStore, setPlatformStore] = makePersisted(
    createStore({
        version: "UNKNOWN" as string,
        accept_cookies: false,
        under_maintenance: false,
        backend_online: false,
        get isOnline() {
            return this.backend_online && !this.under_maintenance;
        },
    }),
    { name: "platform" }
);
