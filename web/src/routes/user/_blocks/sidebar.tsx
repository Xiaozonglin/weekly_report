import { get_user, get_user_reports } from "@/lib/api";
import type { Report } from "@/lib/models/report";
import type { User } from "@/lib/models/user";
import { fullTheme, t } from "@/lib/storage/theme";
import { addToast } from "@/lib/storage/toast";
import Link from "@/lib/widgets/link";
import { useNavigate, useParams, useSearchParams } from "@solidjs/router";
import type { HTTPError } from "ky";
import { OverlayScrollbarsComponent } from "overlayscrollbars-solid";
import { For, createEffect, createSignal, untrack } from "solid-js";

export default function () {
    const params = useParams();
    const [user, setUser] = createSignal(null as number | null);
    const [userModel, setUserModel] = createSignal(null as User | null);
    const [reports, setReports] = createSignal([] as Report[]);
    const navigate = useNavigate();
    const [searchParams, _] = useSearchParams();
    createEffect(() => {
        if (params.user) {
            untrack(() => setUser(Number.parseInt(params.user)));
        }
    });
    createEffect(() => {
        if (!user()) navigate("/sigtrap/404");
        untrack(() => {
            get_user_reports(user()!)
                .then((data) => {
                    setReports(data.sort((a, b) => b.week - a.week));
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
            get_user(user()!)
                .then((data) => {
                    if (data) setUserModel(data);
                    else navigate("/sigtrap/404");
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
                    <span class="icon-[fluent--person-20-regular] w-5 h-5" />
                    <span>{t("user.title", { user: userModel()?.name || "" })}</span>
                </h2>
                <For each={reports()}>
                    {(report) => (
                        <>
                            <Link
                                ghost
                                href={`/user/${user()}?week=${report.week}`}
                                active={searchParams.week === report.week.toString()}
                                justify="start"
                            >
                                <span class="icon-[fluent--calendar-20-regular] w-5 h-5" />
                                <span>{t("user.week", { week: report.week })}</span>
                            </Link>
                        </>
                    )}
                </For>
            </div>
        </OverlayScrollbarsComponent>
    );
}
