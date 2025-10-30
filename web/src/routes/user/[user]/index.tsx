import { get_report } from "@api";
import type { Report } from "@models/report";
import { t } from "@storage/theme";
import { addToast } from "@storage/toast";
import Article from "@widgets/article";
import LoadingTips from "@widgets/loading-tips";
import { A, useNavigate, useParams, useSearchParams } from "@solidjs/router";
import type { HTTPError } from "ky";
import { Match, Switch, createEffect, createSignal, untrack } from "solid-js";
import { accountStore } from "@storage/account";
import { get_self_feed_token } from "@api";

export default function () {
    const params = useParams();
    const [searchParams, _] = useSearchParams();
    const navigate = useNavigate();
    const [report, setReport] = createSignal(null as Report | null);
    const [loading, setLoading] = createSignal(false);
    createEffect(() => {
        if (params.user && searchParams.week) {
            untrack(() => {
                const user = Number.parseInt(params.user);
                const week = Number.parseInt(searchParams.week!);
                if (!user || !week) {
                    navigate("/sigtrap/404");
                }
                setLoading(true);
                get_report(user, week)
                    .then(setReport)
                    .catch((err: HTTPError) => {
                        err.response.text().then((text) => {
                            addToast({
                                level: "error",
                                description: text,
                                duration: 5000,
                            });
                            navigate("/sigtrap/502");
                        });
                    })
                    .finally(() => setLoading(false));
            });
        }
    });
    return (
        <>
            <Switch>
                <Match when={loading()}>
                    <div class="flex-1 flex flex-col items-center justify-center">
                        <LoadingTips />
                    </div>
                </Match>
                <Match when={report()}>
                    <div class="flex-1 flex flex-col items-center p-3 lg:p-6">
                        <h1 class="font-bold h-12 flex items-center px-2 space-x-2 border-b-2 border-b-layer-content/10 w-full max-w-5xl">
                            <span class="flex-1 text-start">
                                {t("report.title", { user: report()?.author_name!, week: report()?.week! })}
                            </span>
                            <A class="px-2" href={`/user/${report()?.author_id}`}>
                                <span class="icon-[fluent--person-20-regular]" />
                            </A>
                            <A class="px-2" href={`/week/${report()?.week}`}>
                                <span class="icon-[fluent--calendar-20-regular]" />
                            </A>
                            {/* Copy feed link button */}
                            <button
                                class="px-2"
                                title={t("form.copy")}
                                onClick={async () => {
                                    try {
                                        if (!accountStore.user) {
                                            addToast({ level: "error", description: "请先登录以获取订阅链接", duration: 5000, });
                                            return;
                                        }
                                        const resp = await get_self_feed_token();
                                        const token = resp?.token;
                                        if (!token) {
                                            addToast({ level: "error", description: "无法获取订阅 token", duration: 5000, });
                                            return;
                                        }
                                        const base = (window as any).WR_PUBLIC_URL || window.location.origin;
                                        const url = `${base.replace(/\/$/, "")}/api/${report()?.author_id}/feed/?token=${token}`;
                                        await navigator.clipboard.writeText(url);
                                        addToast({ level: "success", description: "已复制订阅链接到剪贴板", duration: 5000, });
                                    } catch (e) {
                                        addToast({ level: "error", description: "复制失败", duration: 5000, });
                                    }
                                }}
                            >
                                <span class="icon-[fluent--rss-20-regular] w-5 h-5" />
                            </button>
                            
                            <A class="px-2" href={`/week/${report()?.week}`}>
                                <span class="icon-[fluent--calendar-20-regular]" />
                            </A>
                        </h1>
                        <Article extra headingAnchors content={report()?.content || ""} />
                    </div>
                </Match>
                <Match when={true}>
                    <div class="flex-1 flex flex-col items-center justify-center space-y-8">
                        <span class="icon-[fluent--archive-20-regular] w-12 h-12 opacity-60" />
                        <h2 class="opacity-60">{t("form.selectSomething")}</h2>
                    </div>
                </Match>
            </Switch>
        </>
    );
}
