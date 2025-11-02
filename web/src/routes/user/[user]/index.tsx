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
import { get_self_feed_token, regenerate_self_feed_token } from "@api";

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
                const week = Number.parseInt(Array.isArray(searchParams.week) ? searchParams.week[0] : (searchParams.week as string));
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
                                        const base = location.origin;
                                        const authorId = report()?.author_id;
                                        if (!authorId) {
                                            addToast({ level: "error", description: t("feed.invalidAuthor")!, duration: 5000 });
                                            return;
                                        }
                                        let url: string;
                                        if (accountStore.user) {
                                            const resp = await get_self_feed_token();
                                            const token = resp?.token;
                                            if (!token) {
                                                addToast({ level: "error", description: t("feed.tokenFetchFailed")!, duration: 5000 });
                                                return;
                                            }
                                            url = `${base.replace(/\/$/, "")}/api/${authorId}/feed/?token=${token}`;
                                        } else {
                                            const envSub = (import.meta.env.VITE_DEV_SUBSCRIBER as string) || "linlinzzo";
                                            const subscriberName = encodeURIComponent(envSub);
                                            url = `${base.replace(/\/$/, "")}/api/${authorId}/feed/?subscriber_name=${subscriberName}`;
                                            addToast({ level: "info", description: t("feed.devFallback")!, duration: 5000 });
                                        }
                                        await navigator.clipboard.writeText(url);
                                        addToast({ level: "success", description: t("feed.copied")!, duration: 5000 });
                                    } catch (e) {
                                        addToast({ level: "error", description: t("feed.copyFailed")!, duration: 5000 });
                                    }
                                }}
                            >
                                <span class="icon-[fluent--rss-20-regular] w-5 h-5" />
                            </button>
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
