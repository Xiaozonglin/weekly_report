import { Title } from "@/lib/storage/header";
import { t } from "@/lib/storage/theme";
import Button from "@/lib/widgets/button";
import Editor from "@/lib/widgets/editor";
import { createSignal } from "solid-js";

export default function () {
    const [hasError, setHasError] = createSignal(false);
    const [content, setContent] = createSignal("");
    return (
        <>
            <Title title={`${t("submit.title")} - ${t("platform.name")}`} />
            <div class="p-3 lg:p-6 flex flex-col self-center w-full max-w-5xl flex-1">
                <h1 class="h-12 flex flex-row space-x-2 items-center">
                    <span class="font-bold px-2 flex-1 text-start">{t("submit.title")}</span>
                    <Button size="sm" level="primary">
                        {t("form.submit")}
                    </Button>
                </h1>
                <Editor
                    class="flex-1"
                    lineNumbers
                    placeholder={t("submit.placeholder") as string}
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
