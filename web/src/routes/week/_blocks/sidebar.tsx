import { get_weekly_reports } from "@/lib/api";
import type { Report } from "@/lib/models/report";
import { fullTheme, t } from "@/lib/storage/theme";
import { addToast } from "@/lib/storage/toast";
import Link from "@/lib/widgets/link";
import { useNavigate, useParams, useSearchParams } from "@solidjs/router";
import type { HTTPError } from "ky";
import { OverlayScrollbarsComponent } from "overlayscrollbars-solid";
import { For, createEffect, createSignal, untrack } from "solid-js";

export default function () {
    const params = useParams();
    const [week, setWeek] = createSignal(null as number | null);
    const [reports, setReports] = createSignal([] as Report[]);
    const navigate = useNavigate();
    const [searchParams, _] = useSearchParams();
    createEffect(() => {
        if (params.week) {
            untrack(() => setWeek(Number.parseInt(params.week)));
        }
    });
    createEffect(() => {
        if (!week()) navigate("/sigtrap/404");
        untrack(() => {
            get_weekly_reports(week()!)
                .then((data) => {
                    setReports(data);
                })
                .catch((err: HTTPError) => {
                    err.response.text().then((text) => {
                        addToast({
                            level: "error",
                            description: text,
                            duration: 5000,
                        });
                    });
                });
        });
    });
    return (
        <OverlayScrollbarsComponent
            options={{
                scrollbars: {
                    theme: `os-theme-${fullTheme()}`,
                    autoHide: "scroll",
                },
            }}
            class="relative w-full h-full print:h-auto print:overflow-auto"
            defer
        >
            <div class="p-3 lg:p-6 flex flex-col space-y-2">
                <h2 class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-2 px-4">
                    <span class="icon-[fluent--calendar-20-regular] w-5 h-5" />
                    <span>{t("week.title", { week: week() || "" })}</span>
                </h2>
                <For each={reports()}>
                    {(report) => (
                        <>
                            <Link
                                ghost
                                href={`/week/${week()}?user=${report.author_id}`}
                                active={searchParams.user === report.author_id.toString()}
                                justify="start"
                            >
                                <span class="icon-[fluent--person-20-regular] w-5 h-5" />
                                <span>{report.author_name}</span>
                            </Link>
                        </>
                    )}
                </For>
            </div>
        </OverlayScrollbarsComponent>
    );
}
