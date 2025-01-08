import { get_reports } from "@api";
import type { User } from "@models/user";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import { addToast } from "@storage/toast";
import { getCurrentWeek } from "@utils/time";
import Link from "@widgets/link";
import LoadingTips from "@widgets/loading-tips";
import type { HTTPError } from "ky";
import { For, Show, createSignal } from "solid-js";

export default function () {
    const [userStates, setUserStates] = createSignal([] as User[]);
    const [weeks, setWeeks] = createSignal([] as number[]);
    const [loading, setLoading] = createSignal(true);
    const week_set = new Set<number>();
    week_set.add(getCurrentWeek());
    get_reports()
        .then(([users, reports]) => {
            for (const user of users) {
                user.recent_reports = reports
                    .filter((report) => report.author_id === user.id)
                    .map((report) => {
                        week_set.add(report.week);
                        return report.week;
                    });
            }
            setUserStates(
                users.sort((a, b) => {
                    if (a.direction !== b.direction) return a.direction!.localeCompare(b.direction!);
                    return a.level - b.level;
                })
            );
            setWeeks(Array.from(week_set).sort((a, b) => a - b));
        })
        .catch((err: HTTPError) => {
            err.response.text().then((text) => {
                addToast({
                    level: "error",
                    description: text,
                    duration: 5000,
                });
            });
        })
        .finally(() => setLoading(false));
    return (
        <>
            <Title title={t("platform.name")!} />
            <div class="p-3 lg:p-6 flex flex-col flex-1 relative">
                <table>
                    <thead class="border-b-2 border-b-layer-content/15 bg-layer h-12 align-middle sticky top-16 z-20">
                        <tr>
                            <th class="font-bold px-3 text-start sticky left-0 bg-layer transition-colors duration-500 border-r border-r-layer-content/5 z-10" />
                            <th class="font-bold px-3 text-start">{t("table.level")}</th>
                            <th class="font-bold px-3 text-start">{t("table.direction")}</th>
                            <For each={weeks()}>
                                {(week) => (
                                    <th class="font-bold px-1">
                                        <Link ghost size="sm" class="w-full" href={`/week/${week}`}>
                                            {week}
                                        </Link>
                                    </th>
                                )}
                            </For>
                        </tr>
                    </thead>
                    <tbody>
                        <For each={userStates()}>
                            {(user) => (
                                <tr class="h-12 align-middle border-b border-b-layer-content/10">
                                    <td class="px-1 text-nowrap sticky left-0 bg-layer transition-colors duration-500 border-r border-r-layer-content/5 z-10">
                                        <Link ghost size="sm" class="w-full overflow-hidden" href={`/user/${user.id}`}>
                                            <span class="text-start flex-1 truncate">{user.name}</span>
                                        </Link>
                                    </td>
                                    <td class="px-3 truncate">{user.level}</td>
                                    <td class="px-3 truncate">{user.direction}</td>
                                    <For each={weeks()}>
                                        {(week) => (
                                            <td class="px-1 align-middle min-w-36">
                                                <Show when={user.recent_reports?.includes(week)}>
                                                    <Link
                                                        href={`/user/${user.id}?week=${week}`}
                                                        class="w-full"
                                                        size="sm"
                                                        ghost
                                                    >
                                                        <span class="icon-[fluent--checkmark-circle-20-filled] w-5 h-5 text-success" />
                                                    </Link>
                                                </Show>
                                            </td>
                                        )}
                                    </For>
                                </tr>
                            )}
                        </For>
                    </tbody>
                </table>
                <Show when={loading()}>
                    <div class="flex-1 flex flex-col items-center justify-center space-y-8">
                        <LoadingTips />
                    </div>
                </Show>
                <Show when={userStates().length === 0 && !loading()}>
                    <div class="flex-1 flex flex-col items-center justify-center space-y-8">
                        <span class="icon-[fluent--archive-20-regular] w-12 h-12 opacity-60" />
                        <h2 class="font-bold text-2xl">{t("table.empty")}</h2>
                        <p class="opacity-60">{t("errors.404Tip")}</p>
                    </div>
                </Show>
            </div>
        </>
    );
}
