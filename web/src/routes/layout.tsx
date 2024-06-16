import { get_self } from "@/lib/api";
import { setAccountStore } from "@/lib/storage/account";
import { t, themeStore } from "@/lib/storage/theme";
import { addToast } from "@/lib/storage/toast";
import Link from "@/lib/widgets/link";
import xdsecLogo from "@assets/favicon.png";
import { useNavigate, useParams } from "@solidjs/router";
import type { HTTPError } from "ky";
import { type JSX, createEffect } from "solid-js";
import DiyBox from "./_blocks/diy-box";
import NotificationBox from "./_blocks/notification-box";
import Toasts from "./_blocks/toasts";

function TitleBar() {
    const params = useParams();
    const navigate = useNavigate();
    createEffect(() => {
        if (params.week) {
            const week = Number.parseInt(params.week);
            if (!week) {
                navigate("/sigtrap/404");
            }
        }
    });
    get_self()
        .then((user) => {
            setAccountStore({ user });
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
    return (
        <>
            <div id="page-top" class="print:hidden" />
            <div class="h-16 border-b border-b-layer-content/15 w-screen bg-layer/60 backdrop-blur z-50 print:hidden print:static print:h-0 print:max-h-0 print:overflow-hidden sticky top-0 left-0 transition-colors duration-700">
                <div class="bg-layer-content/5 w-full h-full px-2 py-0 flex flex-row space-x-2 items-center relative">
                    <Link href="/" ghost>
                        <img
                            class={`w-8 h-8 mr-2 ${themeStore.colorScheme === "dark" ? "invert" : ""}`}
                            src={xdsecLogo}
                            alt="XDSEC"
                        />
                        <span>{t("platform.name")}</span>
                    </Link>
                    <div class="flex-1" />
                    <div class="hidden xl:flex flex-row space-x-2">
                        <Link href="/admin" square ghost>
                            <span class="icon-[fluent--settings-20-regular] w-5 h-5" />
                        </Link>
                        <NotificationBox />
                        <DiyBox />
                    </div>
                    <Link ghost href="/submit">
                        <span class="icon-[fluent--send-20-regular] w-5 h-5 flex-shrink-0" />
                        <span>{t("form.createReport")}</span>
                    </Link>
                </div>
            </div>
        </>
    );
}

export default function (props: { children?: JSX.Element }) {
    return (
        <>
            <TitleBar />
            {props.children}
            <Toasts />
        </>
    );
}
