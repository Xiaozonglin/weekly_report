import { get_report } from "@/lib/api";
import type { Report } from "@/lib/models/report";
import { t } from "@/lib/storage/theme";
import { addToast } from "@/lib/storage/toast";
import Article from "@/lib/widgets/article";
import LoadingTips from "@/lib/widgets/loading-tips";
import { useNavigate, useParams, useSearchParams } from "@solidjs/router";
import type { HTTPError } from "ky";
import { Match, Switch, createEffect, createSignal, untrack } from "solid-js";

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
                    <div class="flex-1 flex flex-col items-center p-3 lg:p-6 pb-0 lg:pb-0">
                        <h1 class="text-3xl font-bold">
                            {t("report.title", { user: report()?.author_name!, week: report()?.week! })}
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
