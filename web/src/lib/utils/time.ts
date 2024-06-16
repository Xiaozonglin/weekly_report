import { DateTime } from "luxon";

export function getCurrentWeek() {
    const c = DateTime.now();
    if (c.weekday === 7) {
        return Number.parseInt(c.toFormat("yyyyMMdd"));
    }
    return Number.parseInt(c.set({ weekday: 7 }).toFormat("yyyyMMdd"));
}
