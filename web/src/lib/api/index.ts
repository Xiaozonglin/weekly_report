import type { Report } from "@models/report";
import type { User } from "@models/user";
import { luxonReplacer, luxonReviver } from "@models/utils";
import ky from "ky";

export const api_root = (import.meta.env.VITE_API_ROOT as string) || "/api";

const api = ky.extend({
    parseJson: (text) => JSON.parse(text, luxonReviver) as unknown,
    stringifyJson: (data) => JSON.stringify(data, luxonReplacer),
});

export default api;

export async function get_reports() {
    return await api.get(`${api_root}/report`).json<[User[], Report[]]>();
}

export async function get_weekly_reports(week: number) {
    return await api.get(`${api_root}/report?week=${week}`).json<Report[]>();
}

export async function get_user_reports(user: number) {
    return await api.get(`${api_root}/report?user=${user}`).json<Report[]>();
}

export async function get_report(user: number, week: number) {
    return await api.get(`${api_root}/report?user=${user}&week=${week}`).json<Report>();
}

export async function get_self() {
    return await api.get(`${api_root}/self`).json<User>();
}

export async function get_user_list(hidden: boolean) {
    return await api.get(`${api_root}/user?with_hidden=${hidden}`).json<User[]>();
}

export async function get_user(user: number) {
    return await api.get(`${api_root}/user?id=${user}`).json<User>();
}
