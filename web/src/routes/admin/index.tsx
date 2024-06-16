import { Title } from "@/lib/storage/header";
import { t } from "@/lib/storage/theme";

export default function () {
    return (
        <>
            <Title title={`${t("admin.title")} - ${t("platform.name")}`} />
        </>
    );
}
