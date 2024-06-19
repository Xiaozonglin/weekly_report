import { get_report, submit_report } from "@/lib/api";
import { accountStore } from "@/lib/storage/account";
import { Title } from "@/lib/storage/header";
import { t } from "@/lib/storage/theme";
import { addToast } from "@/lib/storage/toast";
import { getCurrentWeek } from "@/lib/utils/time";
import Button from "@/lib/widgets/button";
import Card from "@/lib/widgets/card";
import Editor from "@/lib/widgets/editor";
import type { HTTPError } from "ky";
import { createSignal } from "solid-js";

export default function () {
    const [hasError, setHasError] = createSignal(false);
    const [content, setContent] = createSignal("");
    const [loading, setLoading] = createSignal(false);
    if (accountStore.user) {
        setLoading(true);
        get_report(accountStore.user.id, getCurrentWeek())
            .then((report) => {
                if (report) {
                    setContent(report.content!);
                }
            })
            .finally(() => setLoading(false));
    }
    function handleSubmit() {
        if (content().length === 0) {
            setHasError(true);
            return;
        }
        setLoading(true);
        submit_report(content())
            .then(() => {
                addToast({
                    level: "success",
                    description: t("submit.success")!,
                    duration: 5000,
                });
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
            .finally(() => {
                setLoading(false);
            });
    }
    return (
        <>
            <Title title={`${t("submit.title")} - ${t("platform.name")}`} />
            <div class="p-3 lg:p-6 flex flex-col self-center w-full max-w-5xl flex-1 space-y-2">
                <h1 class="h-12 flex flex-row space-x-2 items-center">
                    <span class="font-bold px-2 flex-1 text-start">{t("submit.title")}</span>
                    <Button size="sm" level="primary" onClick={handleSubmit} loading={loading()}>
                        {t("form.submit")}
                    </Button>
                </h1>
                <Card level="info" contentClass="p-2">
                    <p>{t("submit.placeholder")}</p>
                    <a
                        class="inline-flex space-x-2 items-center"
                        href="https://docs.github.com/zh/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax"
                    >
                        <span>GitHub Favored Markdown Tutorial</span>
                        <span class="icon-[fluent--open-20-regular] w-5 h-5 text-primary" />
                    </a>
                </Card>
                <Editor
                    class="flex-1"
                    lineNumbers
                    placeholder="MARKDOWN"
                    error={hasError() ? (t("submit.required") as string) : undefined}
                    onValueChanged={(value) => {
                        setContent(value);
                        if (hasError()) {
                            setHasError(value.length === 0);
                        }
                    }}
                    value={content()}
                    lang="markdown"
                />
            </div>
        </>
    );
}
