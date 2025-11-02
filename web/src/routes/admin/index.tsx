import { Title } from "@/lib/storage/header";
import { t } from "@/lib/storage/theme";
import rxSticker from "@assets/imgs/rx.webp";
import { addToast } from "@storage/toast";
import { accountStore } from "@storage/account";
import { Show } from "solid-js";
import { get_self_feed_token, regenerate_self_feed_token } from "@api";
import Button from "@widgets/button";
import Icon from "@widgets/icon";

export default function () {
    const randomTips = [
        t("platform.notImplementedTips"),
        t("platform.notImplementedTips1"),
        t("platform.notImplementedTips2"),
        t("platform.notImplementedTips3"),
    ];
    return (
        <>
            <Title title={`${t("admin.title")} - ${t("platform.name")}`} />
            <section class="flex-1 flex flex-col relative items-center justify-center space-y-8">
                <img class="rounded-xl" src={rxSticker} alt=">ω<" width={256} height={256} />
                <h1 class="font-bold text-3xl space-x-4">
                    <span class="opacity-60">{t("platform.hello")}</span>
                    <span class="text-primary">|</span>
                    <span>{t("platform.notImplemented")}</span>
                </h1>
                <p class="opacity-60">{randomTips[Math.floor(Math.random() * randomTips.length)]}</p>
                <div class="w-full max-w-2xl mt-6 p-4 border rounded-md bg-card/40">
                    <h2 class="font-semibold mb-2 flex items-center space-x-2">
                        <Icon name="settings" class="w-5 h-5" />
                        <span>{t("feed.settings")}</span>
                    </h2>
                    <p class="text-sm text-muted mb-4">{t("feed.settingsIntro")}</p>
                    <div class="flex items-center space-x-2">
                        <Show when={accountStore.user} fallback={<span class="opacity-60">{t("feed.needLogin")}</span>}>
                            <span class="flex-1">{accountStore.user?.name} (id:{accountStore.user?.id})</span>
                            <Button
                                size="sm"
                                level="primary"
                                onClick={async () => {
                                    if (!confirm(t("feed.reset") + "?")) return;
                                    try {
                                        const resp = await regenerate_self_feed_token();
                                        const token = resp?.token;
                                        if (!token) {
                                            addToast({ level: "error", description: t("feed.tokenFetchFailed")!, duration: 5000 });
                                            return;
                                        }
                                        const base = location.origin;
                                        const authorId = accountStore.user?.id;
                                        const url = `${base.replace(/\/$/, "")}/api/${authorId}/feed/?token=${token}`;
                                        await navigator.clipboard.writeText(url);
                                        addToast({ level: "success", description: t("feed.resetSuccess")!, duration: 5000 });
                                    } catch (e) {
                                        addToast({ level: "error", description: t("feed.copyFailed")!, duration: 5000 });
                                    }
                                }}
                            >
                                {t("feed.reset")}
                            </Button>
                        </Show>
                    </div>
                </div>
            </section>
        </>
    );
}
