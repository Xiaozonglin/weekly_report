import { get_report } from "@api";
import type { Report } from "@models/report";
import { t } from "@storage/theme";
import { addToast } from "@storage/toast";
import Article from "@widgets/article";
import LoadingTips from "@widgets/loading-tips";
import { A, useNavigate, useParams, useSearchParams } from "@solidjs/router";
import type { HTTPError } from "ky";
import { Match, Switch, Show, createEffect, createSignal, untrack, createMemo } from "solid-js";
import { accountStore } from "@storage/account";
import { get_self_feed_token, regenerate_self_feed_token, like_report, unlike_report } from "@api";

export default function () {
    const params = useParams();
    const [searchParams, _] = useSearchParams();
    const navigate = useNavigate();
    const [report, setReport] = createSignal(null as Report | null);
    // Maintain likes in a separate signal to avoid re-rendering the article/content when only likes change
    const [likes, setLikes] = createSignal<string[]>([]);
    const [loading, setLoading] = createSignal(false);
    const [processing, setProcessing] = createSignal(false);
    const hasLiked = createMemo(() => {
        const me = accountStore.user?.name || "";
        const l = likes() || [];
        return !!me && l.includes(me);
    });
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
                    .then((r: Report) => {
                        setReport(r);
                        setLikes(Array.isArray(r?.likes) ? r.likes : []);
                    })
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
                            {/* Like/Unlike button (moved to title) */}
                            <Show when={accountStore.user} fallback={null}>
                                <button
                                    class="px-2"
                                    title={t("form.like")}
                                    disabled={processing()}
                                    onClick={async () => {
                                        try {
                                            // Block self-like/unlike at UI and show i18n toast
                                            if (accountStore.user?.id === report()?.author_id) {
                                                addToast({ level: "error", description: t("like.self")!, duration: 5000 });
                                                return;
                                            }
                                            if (!report()) return;
                                            const liked = hasLiked();
                                            setProcessing(true);
                                            try {
                                                if (liked) {
                                                    const resp = await unlike_report(report()!.id);
                                                    setLikes(Array.isArray(resp.likes) ? resp.likes : []);
                                                } else {
                                                    const resp = await like_report(report()!.id);
                                                    setLikes(Array.isArray(resp.likes) ? resp.likes : []);
                                                }
                                            } finally {
                                                setProcessing(false);
                                            }
                                        } catch (e) {
                                            // i18n-aware error handling for like/unlike actions
                                            let desc = t("like.failed")!;
                                            // ky throws HTTPError on non-2xx
                                            const err = e as HTTPError;
                                            if (err && (err as any).response) {
                                                try {
                                                    const status = (err as any).response?.status as number | undefined;
                                                    const textRaw = await (err as any).response?.text?.();
                                                    const text = (textRaw || "").toString().trim().toLowerCase();
                                                    if (text.includes("cannot like your own report")) {
                                                        desc = t("like.self")!;
                                                    } else if (text.includes("cannot unlike your own report")) {
                                                        desc = t("like.unlikeSelf")!;
                                                    } else if (text.includes("already liked")) {
                                                        desc = t("like.already")!;
                                                    } else if (typeof status === "number") {
                                                        // Fallback to generic error messages by status code
                                                        const generic = t(`errors.${status}` as any);
                                                        if (generic) desc = generic as string;
                                                        else desc = t("errors.unknown")!;
                                                    }
                                                } catch (_) {
                                                    // ignore parse failures
                                                }
                                            } else if (e instanceof TypeError) {
                                                // likely network error from fetch/ky
                                                desc = t("like.network")!;
                                            } else {
                                                desc = t("like.unknown")!;
                                            }
                                            addToast({ level: "error", description: desc, duration: 5000 });
                                        }
                                    }}
                                >
                                    <span class={hasLiked() ? "icon-[fluent--heart-20-filled] w-5 h-5 text-red-500" : "icon-[fluent--heart-20-regular] w-5 h-5"} />
                                </button>
                            </Show>
                        </h1>
                        <Article extra headingAnchors content={report()?.content || ""} />
                        <div class="w-full max-w-5xl mt-4 p-3 border rounded-md bg-card/20">
                            <div class="text-sm text-muted mb-2">{(likes() || []).length > 0 ? t("feed.likedBy", { names: (likes() || []).join(", ") }) : ""}</div>
                        </div>
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
