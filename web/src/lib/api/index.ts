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
    const res = await api.get(`${api_root}/report`).json<any>();
    // backend returns [users, reports]
    if (Array.isArray(res) && res.length === 2) {
        const users = res[0] as User[];
        const reports = (res[1] as any[]).map(normalizeReport);
        return [users, reports] as [User[], Report[]];
    }
    return res as [User[], Report[]];
}

export async function get_weekly_reports(week: number) {
    const res = await api.get(`${api_root}/report?week=${week}`).json<any[]>();
    return res.map(normalizeReport) as Report[];
}

export async function get_user_reports(user: number) {
    const res = await api.get(`${api_root}/report?user=${user}`).json<any[]>();
    return res.map(normalizeReport) as Report[];
}

export async function get_report(user: number, week: number) {
    const res = await api.get(`${api_root}/report?user=${user}&week=${week}`).json<any>();
    return normalizeReport(res) as Report;
}

export async function get_self() {
    return await api.get(`${api_root}/self`).json<User>();
}

export async function get_self_feed_token() {
    return await api.get(`${api_root}/self/feed_token`).json<{ token: string }>();
}

export async function regenerate_self_feed_token() {
    return await api.post(`${api_root}/self/feed_token`).json<{ token: string }>();
}

export async function get_user_list(hidden: boolean) {
    return await api.get(`${api_root}/user?with_hidden=${hidden}`).json<User[]>();
}

export async function get_user(user: number) {
    return await api.get(`${api_root}/user?id=${user}`).json<User>();
}

export async function submit_report(content: string) {
    const res = await api.post(`${api_root}/report`, { json: { content } }).json<any>();
    return normalizeReport(res) as Report;
}

export async function like_report(reportId: number) {
    return await api.post(`${api_root}/report/${reportId}/like`).json<{ likes: string[] }>();
}

export async function unlike_report(reportId: number) {
    return await api.post(`${api_root}/report/${reportId}/unlike`).json<{ likes: string[] }>();
}

function normalizeReport(r: any): Report {
    if (!r) return r;
    try {
        if (typeof r.likes === 'string') {
            r.likes = JSON.parse(r.likes);
        }
    } catch (e) {
        // if parse fails, fall back to empty array
        r.likes = [] as string[];
    }
    // ensure likes is at least an array
    if (!Array.isArray(r.likes)) {
        r.likes = [] as string[];
    }
    return r as Report;
}
