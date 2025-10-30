/* @refresh reload */
import { render } from "solid-js/web";
import { routes } from "./routes/routes";
import "@fontsource/jetbrains-mono";
import "overlayscrollbars/overlayscrollbars.css";
import "@lib/widgets/styles/base.scss";
import { Router } from "@solidjs/router";
import { OverlayScrollbarsComponent } from "overlayscrollbars-solid";
import { onMount } from "solid-js";
import { fullTheme, initTheme } from "./lib/storage/theme";

function checkEdition() {
    const compact_edition: string = import.meta.env.VITE_COMPAT_EDITION as string;
    const edition = localStorage.getItem("edition");
    const needReload = localStorage.length !== 0 && edition !== compact_edition;
    if (compact_edition && needReload) {
        const systemPrefersLocale = (window.navigator.language || window.navigator.languages[0])
            .replace("-", "_")
            .toLowerCase();
        switch (systemPrefersLocale) {
            case "zh_cn":
                alert("周报系统进行了一项非兼容更新，将重新加载此页面以应用更新。请注意您可能需要重新登录。");
                break;
            case "en_us":
                alert(
                    "Weekly Report has done a major update, we will reload this page to apply it. Please note that you may need re-login."
                );
                break;
        }
        localStorage.clear();
        localStorage.setItem("edition", compact_edition);
        location.reload();
    } else if (compact_edition) {
        localStorage.setItem("edition", compact_edition);
    }
}

render(() => {
    checkEdition();
    initTheme();
    onMount(() => {
        setTimeout(() => {
            document.documentElement.classList.add("transition-colors", "duration-700");
            document.body.classList.add("transition-colors", "duration-700");
        }, 1000);
    });
    return (
        <OverlayScrollbarsComponent
            options={{
                scrollbars: {
                    theme: `os-theme-${fullTheme()}`,
                    autoHide: "scroll",
                },
            }}
            class="relative w-screen h-screen print:h-auto print:overflow-auto"
            defer
        >
            <div class="flex flex-col min-h-full min-w-fit">
                <Router explicitLinks>{routes}</Router>
            </div>
        </OverlayScrollbarsComponent>
    );
}, document.getElementById("root") || document.body);
